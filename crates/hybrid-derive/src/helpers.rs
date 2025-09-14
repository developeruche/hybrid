//! Helper functions and types for the hybrid-derive procedural macros.
//!
//! This module contains the core implementation logic that powers the various
//! derive macros and attributes provided by the hybrid-derive crate. It handles
//! type conversions, function selector generation, interface creation, and
//! code generation for smart contract functionality.

use alloy_core::primitives::keccak256;
use alloy_dyn_abi::DynSolType;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    FnArg, Ident, ImplItemMethod, LitStr, PathArguments, ReturnType, TraitItemMethod, Type,
};

/// Unified method information extracted from both `ImplItemMethod` and `TraitItemMethod`.
///
/// This struct provides a common representation of method signatures that can be used
/// with both trait methods and implementation methods, enabling shared code generation
/// logic across different contexts.
///
/// # Fields
///
/// * `name` - The method identifier
/// * `args` - All function arguments including `self`
/// * `return_type` - The method's return type specification
#[derive(Clone)]
pub struct MethodInfo<'a> {
    name: &'a Ident,
    args: Vec<syn::FnArg>,
    return_type: &'a ReturnType,
}

impl<'a> From<&'a ImplItemMethod> for MethodInfo<'a> {
    fn from(method: &'a ImplItemMethod) -> Self {
        Self {
            name: &method.sig.ident,
            args: method.sig.inputs.iter().cloned().collect(),
            return_type: &method.sig.output,
        }
    }
}

impl<'a> From<&'a TraitItemMethod> for MethodInfo<'a> {
    fn from(method: &'a TraitItemMethod) -> Self {
        Self {
            name: &method.sig.ident,
            args: method.sig.inputs.iter().cloned().collect(),
            return_type: &method.sig.output,
        }
    }
}

impl<'a> MethodInfo<'a> {
    /// Determines if this method requires mutable access to `self`.
    ///
    /// This method examines the first argument to determine if it's `&mut self`
    /// or just `&self`. This information is used to separate methods into
    /// read-only and state-changing categories for interface generation.
    ///
    /// # Returns
    ///
    /// * `true` - If the method takes `&mut self` (can modify state)
    /// * `false` - If the method takes `&self` (read-only)
    ///
    /// # Panics
    ///
    /// Panics if the method doesn't have `self` as the first parameter, which
    /// violates the expected contract method signature format.
    pub fn is_mutable(&self) -> bool {
        match self.args.first() {
            Some(FnArg::Receiver(receiver)) => receiver.mutability.is_some(),
            Some(FnArg::Typed(_)) => panic!("First argument must be self"),
            None => panic!("Expected `self` as the first arg"),
        }
    }
}

/// Internal helper function to extract argument names and types from a method.
///
/// This function processes the method's argument list and extracts both the
/// generated parameter names and their corresponding types. It can optionally
/// skip the first argument (typically `self`) when processing contract methods.
///
/// # Parameters
///
/// * `skip_first_arg` - Whether to skip the first argument (usually `self`)
/// * `method` - The method to extract arguments from
///
/// # Returns
///
/// A tuple containing:
/// * `Vec<Ident>` - Generated parameter names (arg0, arg1, etc.)
/// * `Vec<&Type>` - References to the parameter types
fn get_arg_props<'a>(
    skip_first_arg: bool,
    method: &'a MethodInfo<'a>,
) -> (Vec<Ident>, Vec<&'a syn::Type>) {
    method
        .args
        .iter()
        .skip(if skip_first_arg { 1 } else { 0 })
        .enumerate()
        .map(|(i, arg)| {
            if let FnArg::Typed(pat_type) = arg {
                (format_ident!("arg{}", i), &*pat_type.ty)
            } else {
                panic!("Expected typed arguments");
            }
        })
        .unzip()
}

/// Extracts argument names and types from a method, excluding the first argument.
///
/// This is the standard function used for contract methods where the first
/// argument is always `self` and should be excluded from ABI encoding.
///
/// # Parameters
///
/// * `method` - The method to extract arguments from
///
/// # Returns
///
/// A tuple containing generated parameter names and their types, excluding `self`.
///
/// # Example
///
/// For a method `fn transfer(&mut self, to: Address, amount: U256)`:
/// - Returns: `([arg0, arg1], [&Address, &U256])`
pub fn get_arg_props_skip_first<'a>(
    method: &'a MethodInfo<'a>,
) -> (Vec<Ident>, Vec<&'a syn::Type>) {
    get_arg_props(true, method)
}

/// Extracts argument names and types from a method, including all arguments.
///
/// This function is used when all arguments need to be processed, such as
/// for constructor methods or standalone functions.
///
/// # Parameters
///
/// * `method` - The method to extract arguments from
///
/// # Returns
///
/// A tuple containing generated parameter names and their types for all arguments.
pub fn get_arg_props_all<'a>(method: &'a MethodInfo<'a>) -> (Vec<Ident>, Vec<&'a syn::Type>) {
    get_arg_props(false, method)
}

/// Supported naming style conversions for interface generation.
///
/// These styles enable automatic conversion between Rust naming conventions
/// (snake_case) and other language conventions (camelCase) for cross-language
/// contract compatibility.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterfaceNamingStyle {
    /// Convert snake_case method names to camelCase.
    ///
    /// Examples:
    /// - `get_balance` → `getBalance`
    /// - `transfer_from` → `transferFrom`
    /// - `set_approval_for_all` → `setApprovalForAll`
    CamelCase,
}

/// Arguments parsed from the `#[interface]` attribute.
///
/// This struct holds configuration options that control how interfaces
/// are generated, including naming style conversions.
pub struct InterfaceArgs {
    /// Optional naming style conversion to apply to method names.
    pub rename: Option<InterfaceNamingStyle>,
}

impl Parse for InterfaceArgs {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let rename_style = if !input.is_empty() {
            let value = if input.peek(LitStr) {
                input.parse::<LitStr>()?.value()
            } else {
                input.parse::<Ident>()?.to_string()
            };

            match value.as_str() {
                "camelCase" => Some(InterfaceNamingStyle::CamelCase),
                invalid => {
                    return Err(syn::Error::new(
                        input.span(),
                        format!(
                            "unsupported style: {}. Only 'camelCase' is supported",
                            invalid
                        ),
                    ))
                }
            }
        } else {
            None
        };

        Ok(InterfaceArgs {
            rename: rename_style,
        })
    }
}

/// Generates a complete interface implementation from a collection of methods.
///
/// This function creates a type-safe contract interface that can be used to
/// call external contracts. The generated interface includes:
/// - Address management and context types
/// - Separate method implementations for read-only and mutable operations
/// - Automatic ABI encoding/decoding for all method calls
/// - Builder pattern for interface instantiation
///
/// # Type Parameters
///
/// * `T` - The method type (either `ImplItemMethod` or `TraitItemMethod`)
///
/// # Parameters
///
/// * `methods` - Collection of methods to include in the interface
/// * `interface_name` - Name for the generated interface struct
/// * `interface_style` - Optional naming style conversion for method names
///
/// # Returns
///
/// A `TokenStream` containing the complete interface implementation.
///
/// # Generated Structure
///
/// The generated interface includes:
/// - `InterfaceName<C: CallCtx>` - Main interface struct
/// - `InitInterface` implementation for creation
/// - Context type conversions (`ReadOnly`, `StaticCtx`, `MutableCtx`)
/// - Method implementations separated by mutability
pub fn generate_interface<T>(
    methods: &[&T],
    interface_name: &Ident,
    interface_style: Option<InterfaceNamingStyle>,
) -> quote::__private::TokenStream
where
    for<'a> MethodInfo<'a>: From<&'a T>,
{
    let methods: Vec<MethodInfo> = methods.iter().map(|&m| MethodInfo::from(m)).collect();
    let (mut_methods, immut_methods): (Vec<MethodInfo>, Vec<MethodInfo>) =
        methods.into_iter().partition(|m| m.is_mutable());

    // Generate implementations
    let mut_method_impls = mut_methods
        .iter()
        .map(|method| generate_method_impl(method, interface_style, true));
    let immut_method_impls = immut_methods
        .iter()
        .map(|method| generate_method_impl(method, interface_style, false));

    quote! {
        use core::marker::PhantomData;
        pub struct #interface_name<C: CallCtx> {
            address: Address,
            _ctx: PhantomData<C>
        }

        impl InitInterface for #interface_name<ReadOnly> {
            fn new(address: Address) -> InterfaceBuilder<Self> {
                InterfaceBuilder {
                    address,
                    _phantom: PhantomData
                }
            }
        }

        // Implement conversion between interface types
        impl<C: CallCtx> IntoInterface<#interface_name<C>> for #interface_name<ReadOnly> {
            fn into_interface(self) -> #interface_name<C> {
                #interface_name {
                    address: self.address,
                    _ctx: PhantomData
                }
            }
        }

        impl<C: CallCtx> FromBuilder for #interface_name<C> {
            type Context = C;

            fn from_builder(builder: InterfaceBuilder<Self>) -> Self {
                Self {
                    address: builder.address,
                    _ctx: PhantomData
                }
            }
        }

        impl <C: CallCtx> #interface_name<C> {
            pub fn address(&self) -> Address {
                self.address
            }
        }

        impl<C: StaticCtx> #interface_name<C> {
            #(#immut_method_impls)*
        }

        impl<C: MutableCtx> #interface_name<C> {
            #(#mut_method_impls)*
        }
    }
}

/// Generates the implementation for a single interface method.
///
/// This function creates the Rust code for an interface method that:
/// - Computes the method's function selector
/// - Encodes method parameters into calldata
/// - Makes the appropriate contract call (call vs staticcall)
/// - Decodes and returns the result
///
/// # Parameters
///
/// * `method` - Method information to generate implementation for
/// * `interface_style` - Optional naming style for the method name
/// * `is_mutable` - Whether this method can modify contract state
///
/// # Returns
///
/// A `TokenStream` containing the method implementation.
///
/// # Generated Method Structure
///
/// The generated method:
/// 1. Encodes parameters into ABI calldata
/// 2. Prepends the 4-byte function selector
/// 3. Makes contract call using appropriate method (call/staticcall)
/// 4. Decodes return data based on expected return type
/// 5. Handles errors by returning `None` or propagating custom errors
fn generate_method_impl(
    method: &MethodInfo,
    interface_style: Option<InterfaceNamingStyle>,
    is_mutable: bool,
) -> TokenStream {
    let name = method.name;
    let return_type = method.return_type;
    let method_selector = u32::from_be_bytes(
        generate_fn_selector(method, interface_style).expect("Unable to generate fn selector"),
    );

    let (arg_names, arg_types) = get_arg_props_skip_first(method);

    let calldata = if arg_names.is_empty() {
        quote! {
            let mut complete_calldata = Vec::with_capacity(4);
            complete_calldata.extend_from_slice(&[
                #method_selector.to_be_bytes()[0],
                #method_selector.to_be_bytes()[1],
                #method_selector.to_be_bytes()[2],
                #method_selector.to_be_bytes()[3],
            ]);
        }
    } else {
        quote! {
            let mut args_calldata = (#(#arg_names),*).abi_encode();
            let mut complete_calldata = Vec::with_capacity(4 + args_calldata.len());
            complete_calldata.extend_from_slice(&[
                #method_selector.to_be_bytes()[0],
                #method_selector.to_be_bytes()[1],
                #method_selector.to_be_bytes()[2],
                #method_selector.to_be_bytes()[3],
            ]);
            complete_calldata.append(&mut args_calldata);
        }
    };

    let (call_fn, self_param) = if is_mutable {
        (
            quote! { hybrid_contract::call_contract },
            quote! { &mut self },
        )
    } else {
        (
            quote! { hybrid_contract::staticcall_contract },
            quote! { &self},
        )
    };

    // Generate different implementations based on return type
    match extract_wrapper_types(&method.return_type) {
        // If `Result<T, E>` handle each individual type
        WrapperType::Result(ok_type, err_type) => quote! {
            pub fn #name(#self_param, #(#arg_names: #arg_types),*) -> Result<#ok_type, #err_type>  {
                use alloy_sol_types::SolValue;
                use alloc::vec::Vec;

                #calldata

                let result = #call_fn(
                    self.address,
                    0_u64,
                    &complete_calldata,
                    None
                );

                match <#ok_type>::abi_decode(&result, true) {
                    Ok(decoded) => Ok(decoded),
                    Err(_) => Err(<#err_type>::abi_decode(&result, true))
                }
            }
        },
        // If `Option<T>` unwrap the type to decode, and wrap it back
        WrapperType::Option(return_ty) => {
            quote! {
                pub fn #name(#self_param, #(#arg_names: #arg_types),*) -> Option<#return_ty> {
                    use alloy_sol_types::SolValue;
                    use alloc::vec::Vec;

                    #calldata

                    let result = #call_fn(
                        self.address,
                        0_u64,
                        &complete_calldata,
                        None
                    );

                    match <#return_ty>::abi_decode(&result, true) {
                        Ok(decoded) => Some(decoded),
                        Err(_) => None
                    }
                }
            }
        }
        // Otherwise, simply decode the value + wrap it in an `Option` to force error-handling
        WrapperType::None => {
            let return_ty = match return_type {
                ReturnType::Default => quote! { () },
                ReturnType::Type(_, ty) => quote! { #ty },
            };
            quote! {
                pub fn #name(#self_param, #(#arg_names: #arg_types),*) -> Option<#return_ty> {
                    use alloy_sol_types::SolValue;
                    use alloc::vec::Vec;

                    #calldata

                    let result = #call_fn(
                        self.address,
                        0_u64,
                        &complete_calldata,
                        None
                    );

                    match <#return_ty>::abi_decode(&result, true) {
                        Ok(decoded) => Some(decoded),
                        Err(_) => None
                    }
                }
            }
        }
    }
}

/// Represents different wrapper types that can be used for method return values.
///
/// This enum categorizes return types to enable proper error handling and
/// result encoding in generated contract methods.
pub enum WrapperType {
    /// `Result<T, E>` return type with success and error types as TokenStreams.
    Result(TokenStream, TokenStream),
    /// `Option<T>` return type with inner type as TokenStream.
    Option(TokenStream),
    /// Direct return type with no wrapper.
    None,
}

/// Analyzes a return type to determine if it's wrapped in `Result`, `Option`, or neither.
///
/// This function examines method return types to understand how to handle
/// success/failure cases in the generated code. Different wrapper types
/// require different error handling strategies:
///
/// - `Result<T, E>` - Returns `Ok(value)` on success, `Err(error)` on failure
/// - `Option<T>` - Returns `Some(value)` on success, `None` on failure
/// - `T` - Direct return, wrapped in `Option` automatically
///
/// # Parameters
///
/// * `return_type` - The return type to analyze
///
/// # Returns
///
/// A `WrapperType` indicating how the return value should be handled.
pub fn extract_wrapper_types(return_type: &ReturnType) -> WrapperType {
    let type_path = match return_type {
        ReturnType::Default => return WrapperType::None,
        ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Path(type_path) => type_path,
            _ => return WrapperType::None,
        },
    };

    let last_segment = match type_path.path.segments.last() {
        Some(segment) => segment,
        None => return WrapperType::None,
    };

    match last_segment.ident.to_string().as_str() {
        "Result" => {
            let PathArguments::AngleBracketed(args) = &last_segment.arguments else {
                return WrapperType::None;
            };

            let type_args: Vec<_> = args.args.iter().collect();
            if type_args.len() != 2 {
                return WrapperType::None;
            }

            // Convert the generic arguments to TokenStreams directly
            let ok_type = match &type_args[0] {
                syn::GenericArgument::Type(t) => quote!(#t),
                _ => return WrapperType::None,
            };

            let err_type = match &type_args[1] {
                syn::GenericArgument::Type(t) => quote!(#t),
                _ => return WrapperType::None,
            };

            WrapperType::Result(ok_type, err_type)
        }
        "Option" => {
            let PathArguments::AngleBracketed(args) = &last_segment.arguments else {
                return WrapperType::None;
            };

            let type_args: Vec<_> = args.args.iter().collect();
            if type_args.len() != 1 {
                return WrapperType::None;
            }

            // Convert the generic argument to TokenStream
            let inner_type = match &type_args[0] {
                syn::GenericArgument::Type(t) => quote!(#t),
                _ => return WrapperType::None,
            };

            WrapperType::Option(inner_type)
        }
        _ => WrapperType::None,
    }
}

/// Generates a 4-byte function selector for a method following Ethereum ABI standards.
///
/// This function computes the function selector by:
/// 1. Converting the method name according to the specified style
/// 2. Converting Rust parameter types to Solidity ABI type names
/// 3. Creating the canonical function signature string
/// 4. Computing the Keccak-256 hash of the signature
/// 5. Returning the first 4 bytes as the selector
///
/// # Parameters
///
/// * `method` - The method to generate a selector for
/// * `style` - Optional naming style conversion (e.g., snake_case to camelCase)
///
/// # Returns
///
/// The 4-byte function selector, or `None` if generation fails.
///
/// # Examples
///
/// ```ignore
/// // transfer(address,uint256) -> 0xa9059cbb
/// // balanceOf(address) -> 0x70a08231
/// // approve(address,uint256) -> 0x095ea7b3
/// ```
///
/// # Function Signature Format
///
/// The signature follows Solidity's canonical format:
/// `functionName(type1,type2,...)` with no spaces and exact type names.
pub fn generate_fn_selector(
    method: &MethodInfo,
    style: Option<InterfaceNamingStyle>,
) -> Option<[u8; 4]> {
    let name = match style {
        None => method.name.to_string(),
        Some(style) => match style {
            InterfaceNamingStyle::CamelCase => to_camel_case(method.name.to_string()),
        },
    };

    let (_, arg_types) = get_arg_props_skip_first(method);
    let args = arg_types
        .iter()
        .map(|ty| rust_type_to_sol_type(ty))
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let args_str = args
        .iter()
        .map(|ty| ty.sol_type_name().into_owned())
        .collect::<Vec<_>>()
        .join(",");

    let selector = format!("{}({})", name, args_str);
    let selector_bytes = keccak256(selector.as_bytes())[..4].try_into().ok()?;
    Some(selector_bytes)
}

/// Converts Rust types to their Solidity ABI equivalent types.
///
/// This function provides the core type mapping between Rust's type system
/// and Solidity's ABI type system. It handles primitive types, collections,
/// and complex nested structures to enable seamless interoperability.
///
/// # Supported Type Mappings
///
/// | Rust Type | Solidity Type | Notes |
/// |-----------|---------------|-------|
/// | `Address` | `address` | 20-byte Ethereum address |
/// | `Function` | `function` | Function pointer (24 bytes) |
/// | `bool`, `Bool` | `bool` | Boolean value |
/// | `String`, `str` | `string` | UTF-8 string |
/// | `Bytes` | `bytes` | Dynamic byte array |
/// | `B1`-`B32` | `bytes1`-`bytes32` | Fixed-size byte arrays |
/// | `U8`-`U256` | `uint8`-`uint256` | Unsigned integers |
/// | `I8`-`I256` | `int8`-`int256` | Signed integers |
/// | `Vec<T>` | `T[]` | Dynamic array |
/// | `[T; N]` | `T[N]` | Fixed-size array |
/// | `(T1, T2, ...)` | `(T1, T2, ...)` | Tuple |
///
/// # Parameters
///
/// * `ty` - The Rust type to convert
///
/// # Returns
///
/// * `Ok(DynSolType)` - The corresponding Solidity ABI type
/// * `Err(&str)` - Error message if conversion is not supported
///
/// # Examples
///
/// ```ignore
/// let addr_type = parse_quote!(Address);
/// assert_eq!(rust_type_to_sol_type(&addr_type)?, DynSolType::Address);
///
/// let array_type = parse_quote!(Vec<U256>);
/// assert_eq!(rust_type_to_sol_type(&array_type)?,
///            DynSolType::Array(Box::new(DynSolType::Uint(256))));
/// ```
///
/// # Implementation Notes
///
/// - Integer types must be multiples of 8 bits and ≤ 256 bits
/// - Fixed bytes must be between 1 and 32 bytes
/// - Nested types (arrays of arrays, etc.) are fully supported
/// - Custom struct types are not yet supported (planned enhancement)
pub fn rust_type_to_sol_type(ty: &Type) -> Result<DynSolType, &'static str> {
    match ty {
        Type::Path(type_path) => {
            let path = &type_path.path;
            let segment = path.segments.last().ok_or("Empty type path")?;
            let ident = &segment.ident;
            let type_name = ident.to_string();

            match type_name.as_str() {
                // Fixed-size types
                "Address" => Ok(DynSolType::Address),
                "Function" => Ok(DynSolType::Function),
                "bool" | "Bool" => Ok(DynSolType::Bool),
                "String" | "str" => Ok(DynSolType::String),
                "Bytes" => Ok(DynSolType::Bytes),
                // Fixed-size bytes
                b if b.starts_with('B') => {
                    let size: usize = b
                        .trim_start_matches('B')
                        .parse()
                        .map_err(|_| "Invalid fixed bytes size")?;
                    if size > 0 && size <= 32 {
                        Ok(DynSolType::FixedBytes(size))
                    } else {
                        Err("Invalid fixed bytes size (between 1-32)")
                    }
                }
                // Fixed-size unsigned integers
                u if u.starts_with('U') => {
                    let size: usize = u
                        .trim_start_matches('U')
                        .parse()
                        .map_err(|_| "Invalid uint size")?;
                    if size > 0 && size <= 256 && size % 8 == 0 {
                        Ok(DynSolType::Uint(size))
                    } else {
                        Err("Invalid uint size (multiple of 8 + leq 256)")
                    }
                }
                // Fixed-size signed integers
                i if i.starts_with('I') => {
                    let size: usize = i
                        .trim_start_matches('I')
                        .parse()
                        .map_err(|_| "Invalid int size")?;
                    if size > 0 && size <= 256 && size % 8 == 0 {
                        Ok(DynSolType::Int(size))
                    } else {
                        Err("Invalid int size (must be multiple of 8, max 256)")
                    }
                }
                // Handle vecs
                _ => {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        match type_name.as_str() {
                            "Vec" => {
                                let inner = args.args.first().ok_or("Empty Vec type argument")?;
                                if let syn::GenericArgument::Type(inner_ty) = inner {
                                    let inner_sol_type = rust_type_to_sol_type(inner_ty)?;
                                    Ok(DynSolType::Array(Box::new(inner_sol_type)))
                                } else {
                                    Err("Invalid Vec type argument")
                                }
                            }
                            _ => Err("Unsupported generic type"),
                        }
                    } else {
                        Err("Unsupported type")
                    }
                }
            }
        }
        Type::Array(array) => {
            let inner_sol_type = rust_type_to_sol_type(&array.elem)?;
            if let syn::Expr::Lit(lit) = &array.len {
                if let syn::Lit::Int(size) = &lit.lit {
                    let size: usize = size
                        .base10_digits()
                        .parse()
                        .map_err(|_| "Invalid array size")?;
                    Ok(DynSolType::FixedArray(Box::new(inner_sol_type), size))
                } else {
                    Err("Invalid array size literal")
                }
            } else {
                Err("Invalid array size expression")
            }
        }
        Type::Tuple(tuple) => {
            let inner_types = tuple
                .elems
                .iter()
                .map(rust_type_to_sol_type)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(DynSolType::Tuple(inner_types))
        }
        _ => Err("Unsupported type"),
    }
}

/// Converts a snake_case string to camelCase format.
///
/// This function transforms Rust's conventional snake_case identifiers into
/// JavaScript/Solidity-style camelCase identifiers for cross-language
/// compatibility in contract interfaces.
///
/// # Parameters
///
/// * `s` - The snake_case string to convert
///
/// # Returns
///
/// The string converted to camelCase format.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(to_camel_case("get_balance".to_string()), "getBalance");
/// assert_eq!(to_camel_case("transfer_from".to_string()), "transferFrom");
/// assert_eq!(to_camel_case("set_approval_for_all".to_string()), "setApprovalForAll");
/// ```
///
/// # Algorithm
///
/// 1. Iterate through characters in the input string
/// 2. Keep the first character lowercase
/// 3. When encountering non-alphanumeric characters (like `_`):
///    - Skip the separator
///    - Capitalize the next alphabetic character
/// 4. Preserve alphanumeric characters in their original case otherwise
fn to_camel_case(s: String) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    // Iterate through characters, skipping non-alphabetic separators
    for (i, c) in s.chars().enumerate() {
        if c.is_alphanumeric() {
            if i == 0 {
                result.push(c.to_ascii_lowercase());
            } else if capitalize_next {
                result.push(c.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(c);
            }
        } else {
            // Set flag to capitalize next char  with non-alphanumeric ones
            capitalize_next = true;
        }
    }

    result
}

/// Generates deployment/initialization code for smart contracts.
///
/// This function creates the deployment bytecode that:
/// 1. Decodes constructor arguments from the transaction data
/// 2. Initializes the contract instance using the constructor or default
/// 3. Returns the runtime bytecode that will handle future contract calls
///
/// The generated deployment code becomes the contract's initialization code,
/// which is executed once during deployment and then replaced with the runtime code.
///
/// # Parameters
///
/// * `struct_name` - Name of the contract struct being deployed
/// * `constructor` - Optional constructor method (must be named `new`)
///
/// # Returns
///
/// A `TokenStream` containing the complete deployment code.
///
/// # Generated Deployment Flow
///
/// 1. **Constructor Arguments**: Decode ABI-encoded constructor parameters
/// 2. **Contract Initialization**: Call constructor or use default initialization
/// 3. **Runtime Code Return**: Load and return the runtime bytecode
/// 4. **Exit**: Terminate deployment with the returned runtime code
///
/// # Constructor Requirements
///
/// If a constructor is provided, it must:
/// - Be named `new`
/// - Take ABI-encodable parameters
/// - Return an instance of the contract struct
/// - Be a static method (not take `&self`)
///
/// # Runtime Code Embedding
///
/// The deployment code expects the runtime bytecode to be available at:
/// `../target/riscv64imac-unknown-none-elf/release/runtime`
///
/// This file is included at compile time and returned during deployment.
pub fn generate_deployment_code(
    struct_name: &Ident,
    constructor: Option<&ImplItemMethod>,
) -> quote::__private::TokenStream {
    // Decode constructor args + trigger constructor logic
    let constructor_code = match constructor {
        Some(method) => {
            let method_info = MethodInfo::from(method);
            let (arg_names, arg_types) = get_arg_props_all(&method_info);
            quote! {
                impl #struct_name { #method }

                // Get encoded constructor args
                let calldata = hybrid_contract::msg_data();

                let (#(#arg_names),*) = <(#(#arg_types),*)>::abi_decode(&calldata, true)
                    .expect("Failed to decode constructor args");
                #struct_name::new(#(#arg_names),*);
            }
        }
        None => quote! {
            #struct_name::default();
        },
    };

    quote! {
        use alloc::vec::Vec;
        use alloy_core::primitives::U32;

        #[no_mangle]
        pub extern "C" fn main() -> ! {
            #constructor_code

            // Return runtime code
            let runtime: &[u8] = include_bytes!("../target/riscv64imac-unknown-none-elf/release/runtime");
            let mut prepended_runtime = Vec::with_capacity(1 + runtime.len());
            prepended_runtime.push(0xff);
            prepended_runtime.extend_from_slice(runtime);

            let prepended_runtime_slice: &[u8] = &prepended_runtime;
            let result_ptr = prepended_runtime_slice.as_ptr() as u64;
            let result_len = prepended_runtime_slice.len() as u64;
            hybrid_contract::return_riscv(result_ptr, result_len);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    /// Mock method structure for testing helper functions.
    ///
    /// This struct wraps an `ImplItemMethod` to provide a consistent interface
    /// for testing the various helper functions in this module.
    struct MockMethod {
        method: ImplItemMethod,
    }

    impl MockMethod {
        fn new(name: &str, args: Vec<&str>) -> Self {
            let name_ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            let args_tokens = if args.is_empty() {
                quote!()
            } else {
                let args = args.iter().map(|arg| {
                    let parts: Vec<&str> = arg.split(": ").collect();
                    let arg_name = syn::Ident::new(parts[0], proc_macro2::Span::call_site());
                    let type_str = parts[1];
                    let type_tokens: proc_macro2::TokenStream = type_str.parse().unwrap();
                    quote!(#arg_name: #type_tokens)
                });
                quote!(, #(#args),*)
            };

            let method: ImplItemMethod = parse_quote! {
                fn #name_ident(&self #args_tokens) {}
            };
            Self { method }
        }

        fn info(&self) -> MethodInfo {
            MethodInfo::from(self)
        }
    }

    impl<'a> From<&'a MockMethod> for MethodInfo<'a> {
        fn from(test_method: &'a MockMethod) -> Self {
            MethodInfo::from(&test_method.method)
        }
    }

    /// Helper function to compute a function selector from a signature string.
    ///
    /// This function is used in tests to verify that generated selectors match
    /// the expected values for known function signatures.
    ///
    /// # Parameters
    ///
    /// * `sig` - The function signature string (e.g., "transfer(address,uint256)")
    ///
    /// # Returns
    ///
    /// The 4-byte function selector computed from the signature.
    pub fn get_selector_from_sig(sig: &str) -> [u8; 4] {
        keccak256(sig.as_bytes())[0..4]
            .try_into()
            .expect("Selector should have exactly 4 bytes")
    }

    #[test]
    fn test_rust_to_sol_basic_types() {
        let test_cases = vec![
            (parse_quote!(Address), DynSolType::Address),
            (parse_quote!(Function), DynSolType::Function),
            (parse_quote!(bool), DynSolType::Bool),
            (parse_quote!(Bool), DynSolType::Bool),
            (parse_quote!(String), DynSolType::String),
            (parse_quote!(str), DynSolType::String),
            (parse_quote!(Bytes), DynSolType::Bytes),
        ];

        for (rust_type, expected_sol_type) in test_cases {
            assert_eq!(
                rust_type_to_sol_type(&rust_type).unwrap(),
                expected_sol_type
            );
        }
    }

    #[test]
    fn test_rust_to_sol_fixed_bytes() {
        let test_cases = vec![
            (parse_quote!(B1), DynSolType::FixedBytes(1)),
            (parse_quote!(B16), DynSolType::FixedBytes(16)),
            (parse_quote!(B32), DynSolType::FixedBytes(32)),
        ];

        for (rust_type, expected_sol_type) in test_cases {
            assert_eq!(
                rust_type_to_sol_type(&rust_type).unwrap(),
                expected_sol_type
            );
        }

        // Invalid cases
        assert!(rust_type_to_sol_type(&parse_quote!(B0)).is_err());
        assert!(rust_type_to_sol_type(&parse_quote!(B33)).is_err());
    }

    #[test]
    fn test_rust_to_sol_integers() {
        let test_cases = vec![
            (parse_quote!(U8), DynSolType::Uint(8)),
            (parse_quote!(U256), DynSolType::Uint(256)),
            (parse_quote!(I8), DynSolType::Int(8)),
            (parse_quote!(I256), DynSolType::Int(256)),
        ];

        for (rust_type, expected_sol_type) in test_cases {
            assert_eq!(
                rust_type_to_sol_type(&rust_type).unwrap(),
                expected_sol_type
            );
        }

        // Invalid cases
        assert!(rust_type_to_sol_type(&parse_quote!(U0)).is_err());
        assert!(rust_type_to_sol_type(&parse_quote!(U257)).is_err());
        assert!(rust_type_to_sol_type(&parse_quote!(U7)).is_err()); // Not multiple of 8
        assert!(rust_type_to_sol_type(&parse_quote!(I0)).is_err());
        assert!(rust_type_to_sol_type(&parse_quote!(I257)).is_err());
        assert!(rust_type_to_sol_type(&parse_quote!(I7)).is_err()); // Not multiple of 8
    }

    #[test]
    fn test_rust_to_sol_arrays() {
        // Dynamic arrays (Vec)
        assert_eq!(
            rust_type_to_sol_type(&parse_quote!(Vec<U256>)).unwrap(),
            DynSolType::Array(Box::new(DynSolType::Uint(256)))
        );

        assert_eq!(
            rust_type_to_sol_type(&parse_quote!(Vec<Bool>)).unwrap(),
            DynSolType::Array(Box::new(DynSolType::Bool))
        );

        // Fixed-size arrays
        assert_eq!(
            rust_type_to_sol_type(&parse_quote!([U256; 5])).unwrap(),
            DynSolType::FixedArray(Box::new(DynSolType::Uint(256)), 5)
        );

        assert_eq!(
            rust_type_to_sol_type(&parse_quote!([Bool; 3])).unwrap(),
            DynSolType::FixedArray(Box::new(DynSolType::Bool), 3)
        );
    }

    #[test]
    fn test_rust_to_sol_tuples() {
        assert_eq!(
            rust_type_to_sol_type(&parse_quote!((U256, Bool))).unwrap(),
            DynSolType::Tuple(vec![DynSolType::Uint(256), DynSolType::Bool])
        );

        assert_eq!(
            rust_type_to_sol_type(&parse_quote!((Address, B32, I128))).unwrap(),
            DynSolType::Tuple(vec![
                DynSolType::Address,
                DynSolType::FixedBytes(32),
                DynSolType::Int(128)
            ])
        );
    }

    #[test]
    fn test_rust_to_sol_nested_types() {
        // Nested Vec
        assert_eq!(
            rust_type_to_sol_type(&parse_quote!(Vec<Vec<U256>>)).unwrap(),
            DynSolType::Array(Box::new(DynSolType::Array(Box::new(DynSolType::Uint(256)))))
        );

        // Nested fixed array
        assert_eq!(
            rust_type_to_sol_type(&parse_quote!([[U256; 2]; 3])).unwrap(),
            DynSolType::FixedArray(
                Box::new(DynSolType::FixedArray(Box::new(DynSolType::Uint(256)), 2)),
                3
            )
        );

        // Nested tuple
        assert_eq!(
            rust_type_to_sol_type(&parse_quote!((U256, (Bool, Address)))).unwrap(),
            DynSolType::Tuple(vec![
                DynSolType::Uint(256),
                DynSolType::Tuple(vec![DynSolType::Bool, DynSolType::Address])
            ])
        );
    }

    #[test]
    fn test_rust_to_sol_invalid_types() {
        // Invalid type names
        assert!(rust_type_to_sol_type(&parse_quote!(InvalidType)).is_err());

        // Invalid generic types
        assert!(rust_type_to_sol_type(&parse_quote!(Option<U256>)).is_err());
        assert!(rust_type_to_sol_type(&parse_quote!(Result<U256>)).is_err());
    }

    #[test]
    fn test_fn_selector() {
        // No arguments
        let method = MockMethod::new("balance", vec![]);
        assert_eq!(
            generate_fn_selector(&method.info(), None).unwrap(),
            get_selector_from_sig("balance()"),
        );

        // Single argument
        let method = MockMethod::new("transfer", vec!["to: Address"]);
        assert_eq!(
            generate_fn_selector(&method.info(), None).unwrap(),
            get_selector_from_sig("transfer(address)"),
        );

        // Multiple arguments
        let method = MockMethod::new(
            "transfer_from",
            vec!["from: Address", "to: Address", "amount: U256"],
        );
        assert_eq!(
            generate_fn_selector(&method.info(), None).unwrap(),
            get_selector_from_sig("transfer_from(address,address,uint256)")
        );

        // Dynamic arrays
        let method = MockMethod::new("batch_transfer", vec!["recipients: Vec<Address>"]);
        assert_eq!(
            generate_fn_selector(&method.info(), None).unwrap(),
            get_selector_from_sig("batch_transfer(address[])")
        );

        // Tuples
        let method = MockMethod::new(
            "complex_transfer",
            vec!["data: (Address, U256)", "check: (Vec<Address>, Vec<Bool>)"],
        );
        assert_eq!(
            generate_fn_selector(&method.info(), None).unwrap(),
            get_selector_from_sig("complex_transfer((address,uint256),(address[],bool[]))")
        );

        // Fixed arrays
        let method = MockMethod::new("multi_transfer", vec!["amounts: [U256; 3]"]);
        assert_eq!(
            generate_fn_selector(&method.info(), None).unwrap(),
            get_selector_from_sig("multi_transfer(uint256[3])")
        );
    }

    #[test]
    fn test_fn_selector_rename_camel_case() {
        let method = MockMethod::new("get_balance", vec![]);
        assert_eq!(
            generate_fn_selector(&method.info(), Some(InterfaceNamingStyle::CamelCase)).unwrap(),
            get_selector_from_sig("getBalance()")
        );

        let method = MockMethod::new("transfer_from_account", vec!["to: Address"]);
        assert_eq!(
            generate_fn_selector(&method.info(), Some(InterfaceNamingStyle::CamelCase)).unwrap(),
            get_selector_from_sig("transferFromAccount(address)")
        );
    }

    #[test]
    fn test_fn_selector_erc20() {
        let cases = vec![
            ("totalSupply", vec![], "totalSupply()"),
            ("balanceOf", vec!["account: Address"], "balanceOf(address)"),
            (
                "transfer",
                vec!["recipient: Address", "amount: U256"],
                "transfer(address,uint256)",
            ),
            (
                "allowance",
                vec!["owner: Address", "spender: Address"],
                "allowance(address,address)",
            ),
            (
                "approve",
                vec!["spender: Address", "amount: U256"],
                "approve(address,uint256)",
            ),
            (
                "transferFrom",
                vec!["sender: Address", "recipient: Address", "amount: U256"],
                "transferFrom(address,address,uint256)",
            ),
        ];

        for (name, args, signature) in cases {
            let method = MockMethod::new(name, args);
            assert_eq!(
                generate_fn_selector(&method.info(), None).unwrap(),
                get_selector_from_sig(signature),
                "Selector mismatch for {}",
                signature
            );
        }
    }

    #[test]
    fn test_fn_selector_erc721() {
        let cases = vec![
            (
                "safeTransferFrom",
                vec![
                    "from: Address",
                    "to: Address",
                    "tokenId: U256",
                    "data: Bytes",
                ],
                "safeTransferFrom(address,address,uint256,bytes)",
            ),
            ("name", vec![], "name()"),
            ("symbol", vec![], "symbol()"),
            ("tokenURI", vec!["tokenId: U256"], "tokenURI(uint256)"),
            (
                "approve",
                vec!["to: Address", "tokenId: U256"],
                "approve(address,uint256)",
            ),
            (
                "setApprovalForAll",
                vec!["operator: Address", "approved: bool"],
                "setApprovalForAll(address,bool)",
            ),
        ];

        for (name, args, signature) in cases {
            let method = MockMethod::new(name, args);
            assert_eq!(
                generate_fn_selector(&method.info(), None).unwrap(),
                get_selector_from_sig(signature),
                "Selector mismatch for {}",
                signature
            );
        }
    }
}

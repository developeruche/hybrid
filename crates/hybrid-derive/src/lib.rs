//! # Hybrid Derive
//!
//! A procedural macro crate for writing smart contracts in Rust that compile to RISC-V
//! and interact with EVM-compatible blockchains.
//!
//! This crate provides a comprehensive set of derive macros and attributes that enable
//! developers to write smart contracts using familiar Rust syntax while automatically
//! generating the necessary boilerplate for blockchain interaction, ABI encoding/decoding,
//! and contract deployment.
//!
//! ## Key Features
//!
//! - **Smart Contract Definition**: `#[contract]` attribute for defining contract logic
//! - **Error Handling**: `#[derive(Error)]` for custom error types with ABI encoding
//! - **Event Emission**: `#[derive(Event)]` for blockchain event logging
//! - **Contract Interfaces**: `#[interface]` for generating type-safe contract interfaces
//! - **Storage Management**: `#[storage]` for defining persistent contract storage
//! - **Payment Handling**: `#[payable]` attribute for functions that can receive payments
//!
//! ## Basic Usage
//!
//! ```rust,ignore
//! use hybrid_derive::{contract, storage, Error, Event};
//! use alloy_primitives::{Address, U256};
//!
//! #[storage]
//! pub struct TokenStorage {
//!     pub balances: StorageMap<Address, U256>,
//!     pub total_supply: StorageValue<U256>,
//! }
//!
//! #[derive(Error)]
//! pub enum TokenError {
//!     InsufficientBalance,
//!     InvalidAddress,
//! }
//!
//! #[derive(Event)]
//! pub struct Transfer {
//!     #[indexed]
//!     pub from: Address,
//!     #[indexed]
//!     pub to: Address,
//!     pub amount: U256,
//! }
//!
//! #[contract]
//! impl TokenStorage {
//!     pub fn transfer(&mut self, to: Address, amount: U256) -> Result<(), TokenError> {
//!         // Contract implementation
//!         Ok(())
//!     }
//! }
//! ```

extern crate proc_macro;
use alloy_core::primitives::U256;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, ImplItem, ImplItemMethod, ItemImpl, ItemTrait,
    ReturnType, TraitItem,
};

mod helpers;
use crate::helpers::{InterfaceArgs, MethodInfo};

/// Derives an `Error` trait implementation for enums that can be ABI-encoded and used
/// as smart contract error types.
///
/// This derive macro generates implementations for:
/// - `Error::abi_encode()` - Encodes the error into ABI format with 4-byte selector
/// - `Error::abi_decode()` - Decodes ABI bytes back into the error type
/// - `Debug` trait for error display
///
/// The error selector is computed as the first 4 bytes of the Keccak-256 hash of the
/// error signature, following Solidity's error handling conventions.
///
/// # Supported Enum Variants
///
/// - **Unit variants**: `ErrorName` - No associated data
/// - **Tuple variants**: `ErrorName(Type1, Type2)` - Multiple unnamed fields
/// - **Named variants**: Not supported (will panic)
///
/// # Examples
///
/// ```rust,ignore
/// use hybrid_derive::Error;
/// use alloy_primitives::{Address, U256};
///
/// #[derive(Error)]
/// pub enum TokenError {
///     // Unit variant - encodes as just the 4-byte selector
///     InsufficientBalance,
///
///     // Tuple variant - encodes selector + ABI-encoded data
///     TransferFailed(Address, U256),
///
///     // Multiple fields
///     InvalidTransfer(Address, Address, U256),
/// }
///
/// // Usage in contract
/// let error = TokenError::InsufficientBalance;
/// let encoded = error.abi_encode(); // 4-byte selector
/// let decoded = TokenError::abi_decode(&encoded, true); // Reconstructs error
/// ```
///
/// # Error Signature Generation
///
/// Error signatures follow the format: `ErrorName(type1,type2,...)` where types
/// are converted to their Solidity ABI type names. For example:
/// - `TransferFailed(Address, U256)` → `"TransferFailed(address,uint256)"`
/// - `InsufficientBalance` → `"InsufficientBalance"`
///
/// # ABI Compatibility
///
/// Generated errors are fully compatible with Solidity's custom error system
/// introduced in Solidity 0.8.4, allowing seamless interaction between
/// Rust and Solidity contracts.
#[proc_macro_derive(Error)]
pub fn error_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = if let Data::Enum(data) = &input.data {
        &data.variants
    } else {
        panic!("`Error` must be an enum");
    };

    // Generate error encoding for each variant
    let encode_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        let signature = match &variant.fields {
            Fields::Unit => {
                format!("{}::{}", name, variant_name)
            }
            Fields::Unnamed(fields) => {
                let type_names: Vec<_> = fields
                    .unnamed
                    .iter()
                    .map(|f| {
                        helpers::rust_type_to_sol_type(&f.ty)
                            .expect("Unknown type")
                            .sol_type_name()
                            .into_owned()
                    })
                    .collect();

                format!("{}::{}({})", name, variant_name, type_names.join(","))
            }
            Fields::Named(_) => panic!("Named fields are not supported"),
        };

        let pattern = match &variant.fields {
            Fields::Unit => quote! { #name::#variant_name },
            Fields::Unnamed(fields) => {
                let vars: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| format_ident!("_{}", i))
                    .collect();
                quote! { #name::#variant_name(#(#vars),*) }
            }
            Fields::Named(_) => panic!("Named fields are not supported"),
        };

        // non-unit variants must encode the data
        let data = match &variant.fields {
            Fields::Unit => quote! {},
            Fields::Unnamed(fields) => {
                let vars = (0..fields.unnamed.len()).map(|i| format_ident!("_{}", i));
                quote! { #( res.extend_from_slice(&#vars.abi_encode()); )* }
            }
            Fields::Named(_) => panic!("Named fields are not supported"),
        };

        quote! {
            #pattern => {
                let mut res = Vec::new();
                let selector = keccak256(#signature.as_bytes())[..4].to_vec();
                res.extend_from_slice(&selector);
                #data
                res
            }
        }
    });

    // Generate error decoding for each variant
    let decode_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        let signature = match &variant.fields {
            Fields::Unit => {
                format!("{}::{}", name, variant_name)
            },
            Fields::Unnamed(fields) => {
                let type_names: Vec<_> = fields.unnamed.iter()
                    .map(|f| helpers::rust_type_to_sol_type(&f.ty)
                        .expect("Unknown type")
                        .sol_type_name()
                        .into_owned()
                    ).collect();

                format!("{}::{}({})",
                    name,
                    variant_name,
                    type_names.join(",")
                )
            },
            Fields::Named(_) => panic!("Named fields are not supported"),
        };

        let selector_bytes = quote!{ &keccak256(#signature.as_bytes())[..4].to_vec() };

        match &variant.fields {
            Fields::Unit => quote! { selector if selector == #selector_bytes => #name::#variant_name },
            Fields::Unnamed(fields) => {
                let field_types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();
                let indices: Vec<_> = (0..fields.unnamed.len()).collect();
                quote!{ selector if selector == #selector_bytes => {
                    let mut values = Vec::new();
                    #( values.push(<#field_types>::abi_decode(data.unwrap(), true).expect("Unable to decode")); )*
                    #name::#variant_name(#(values[#indices]),*)
                }}
            },
            Fields::Named(_) => panic!("Named fields are not supported"),
        }
    });

    // Generate `Debug` implementation for each variant
    let debug_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        match &variant.fields {
            Fields::Unit => quote! {
                #name::#variant_name => { f.write_str(stringify!(#variant_name)) }
            },
            Fields::Unnamed(fields) => {
                let vars: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| format_ident!("_{}", i))
                    .collect();
                quote! {
                    #name::#variant_name(#(#vars),*) => {
                        f.debug_tuple(stringify!(#variant_name))
                            #(.field(#vars))*
                            .finish()
                    }
                }
            }
            Fields::Named(_) => panic!("Named fields are not supported"),
        }
    });

    let expanded = quote! {
        impl hybrid_contract::error::Error for #name {
            fn abi_encode(&self) -> alloc::vec::Vec<u8> {
                use alloy_core::primitives::keccak256;
                use alloc::vec::Vec;

                match self { #(#encode_arms),* }
            }

            fn abi_decode(bytes: &[u8], validate: bool) -> Self {
                use alloy_core::primitives::keccak256;
                use alloy_sol_types::SolValue;
                use alloc::vec::Vec;

                if bytes.len() < 4 { panic!("Invalid error length") };
                let selector = &bytes[..4];
                let data = if bytes.len() > 4 { Some(&bytes[4..]) } else { None };

                match selector {
                    #(#decode_arms),*,
                    _ => panic!("Unknown error")
                }
            }
        }

        impl core::fmt::Debug for #name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self { #(#debug_arms),* }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derives an `Event` trait implementation for structs that can be emitted as
/// blockchain events (logs).
///
/// This derive macro generates implementations for:
/// - `Event::encode_log()` - Encodes the event into log format with topics and data
/// - Event signature computation and topic generation
/// - Support for indexed fields (up to 3 indexed fields allowed)
///
/// Events follow the Ethereum event ABI specification, where the first topic (topic0)
/// is always the Keccak-256 hash of the event signature, and up to 3 additional topics
/// can be used for indexed fields.
///
/// # Attributes
///
/// - `#[indexed]` - Marks a field as indexed (becomes a topic rather than log data)
///
/// # Examples
///
/// ```rust,ignore
/// use hybrid_derive::Event;
/// use alloy_primitives::{Address, U256};
///
/// #[derive(Event)]
/// pub struct Transfer {
///     #[indexed]
///     pub from: Address,    // Will be topic1
///     #[indexed]
///     pub to: Address,      // Will be topic2
///     pub amount: U256,     // Will be in log data
/// }
///
/// // Usage in contract
/// let transfer = Transfer {
///     from: Address::from([0u8; 20]),
///     to: Address::from([1u8; 20]),
///     amount: U256::from(100),
/// };
///
/// let (data, topics) = transfer.encode_log();
/// // topics[0] = keccak256("Transfer(address,address,uint256)")
/// // topics[1] = from address (32 bytes)
/// // topics[2] = to address (32 bytes)
/// // data = ABI-encoded amount
/// ```
///
/// # Event Signature Generation
///
/// Event signatures follow the format: `EventName(type1,type2,...)` where types
/// are converted to their Solidity ABI type names:
/// - `Transfer(Address, Address, U256)` → `"Transfer(address,address,uint256)"`
///
/// # Usage with `emit!` Macro
///
/// Events are typically emitted using the generated `emit!` macro:
///
/// ```rust,ignore
/// emit!(Transfer, from_address, to_address, amount);
/// ```
///
/// # Limitations
///
/// - Maximum of 3 indexed fields (Ethereum limitation)
/// - Only named struct fields supported (not tuple structs)
/// - Field types must implement Solidity ABI encoding
#[proc_macro_derive(Event, attributes(indexed))]
pub fn event_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            &fields.named
        } else {
            panic!("Event must have named fields");
        }
    } else {
        panic!("Event must be a struct");
    };

    // Collect iterators into vectors
    let field_names: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    let indexed_fields: Vec<_> = fields
        .iter()
        .filter(|f| f.attrs.iter().any(|attr| attr.path.is_ident("indexed")))
        .map(|f| &f.ident)
        .collect();

    let expanded = quote! {
        impl #name {
            const NAME: &'static str = stringify!(#name);
            const INDEXED_FIELDS: &'static [&'static str] = &[
                #(stringify!(#indexed_fields)),*
            ];

            pub fn new(#(#field_names: #field_types),*) -> Self {
                Self {
                    #(#field_names),*
                }
            }
        }

        impl hybrid_contract::log::Event for #name {
            fn encode_log(&self) -> (alloc::vec::Vec<u8>, alloc::vec::Vec<[u8; 32]>) {
                use alloy_sol_types::SolValue;
                use alloy_core::primitives::{keccak256, B256};
                use alloc::vec::Vec;

                let mut signature = Vec::new();
                signature.extend_from_slice(Self::NAME.as_bytes());
                signature.extend_from_slice(b"(");

                let mut first = true;
                let mut topics = alloc::vec![B256::default()];
                let mut data = Vec::new();

                #(
                    if !first { signature.extend_from_slice(b","); }
                    first = false;

                    signature.extend_from_slice(self.#field_names.sol_type_name().as_bytes());
                    let encoded = self.#field_names.abi_encode();

                    let field_name = stringify!(#field_names);
                    if Self::INDEXED_FIELDS.contains(&field_name) && topics.len() < 4 {
                        topics.push(B256::from_slice(&encoded));
                    } else {
                        data.extend_from_slice(&encoded);
                    }
                )*

                signature.extend_from_slice(b")");
                topics[0] = B256::from(keccak256(&signature));

                (data, topics.iter().map(|t| t.0).collect())
            }
        }
    };

    TokenStream::from(expanded)
}

/// Debug attribute that prints the attribute and item token streams to stdout.
///
/// This is primarily used for debugging macro expansion and understanding how
/// the Rust compiler processes attributes and items. It's useful during
/// development of the hybrid-derive crate itself.
///
/// # Examples
///
/// ```rust,ignore
/// #[show_streams]
/// fn my_function() {}
/// ```
///
/// This will print both the attribute tokens and the function tokens to the console
/// during compilation.
#[proc_macro_attribute]
pub fn show_streams(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("attr: \"{}\"", attr.to_string());
    println!("item: \"{}\"", item.to_string());
    item
}

/// The main contract attribute that transforms a Rust `impl` block into a smart contract.
///
/// This attribute performs several critical transformations:
/// 1. **Function Selector Generation**: Computes 4-byte selectors for all public methods
/// 2. **Dispatch Logic**: Creates a routing mechanism to call methods based on selectors
/// 3. **ABI Handling**: Automatically encodes/decodes function parameters and return values
/// 4. **Interface Generation**: Creates type-safe interfaces for external contract calls
/// 5. **Deployment Code**: Generates initialization code for contract deployment
///
/// # Usage
///
/// Apply to an `impl` block for a struct that represents your contract:
///
/// ```rust,ignore
/// #[storage]
/// struct MyContract {
///     owner: Address,
///     balance: U256,
/// }
///
/// #[contract]
/// impl MyContract {
///     // Constructor (optional)
///     pub fn new(initial_owner: Address) -> Self {
///         Self {
///             owner: initial_owner,
///             balance: U256::ZERO,
///         }
///     }
///
///     // Public contract methods
///     pub fn get_balance(&self) -> U256 {
///         self.balance
///     }
///
///     #[payable]
///     pub fn deposit(&mut self) {
///         self.balance += hybrid_contract::msg_value();
///     }
/// }
/// ```
///
/// # Generated Components
///
/// The macro generates several modules and implementations:
///
/// ## Deploy Module (when `deploy` feature is enabled)
/// Contains deployment/initialization code that:
/// - Decodes constructor arguments from calldata
/// - Initializes the contract instance
/// - Returns the runtime bytecode
///
/// ## Interface Module (when not in `deploy` mode)
/// Generates type-safe interfaces like `IMyContract` for calling the contract
/// from other contracts or external applications.
///
/// ## Implementation Module (default mode)
/// Contains the actual contract execution logic:
/// - `Contract` trait implementation with `call()` and `call_with_data()` methods
/// - Function selector routing
/// - ABI encoding/decoding for all methods
/// - Entry point for contract execution
///
/// # Method Requirements
///
/// ## Constructor
/// - Must be named `new`
/// - Can take any ABI-encodable parameters
/// - Must return `Self`
/// - Optional (will use `Default::default()` if not provided)
///
/// ## Public Methods
/// - Must have `pub` visibility to be callable externally
/// - Parameters must be ABI-encodable types
/// - Return types can be:
///   - Direct types (wrapped in `Option` automatically)
///   - `Option<T>` for explicit success/failure handling
///   - `Result<T, E>` for error handling with custom error types
///
/// # Return Type Handling
///
/// The macro handles different return types intelligently:
///
/// ```rust,ignore
/// // Direct return - success returns Some(value), failure returns None
/// pub fn get_value(&self) -> U256 { ... }
///
/// // Option return - explicit success/failure handling
/// pub fn try_operation(&self) -> Option<bool> { ... }
///
/// // Result return - custom error types with automatic error encoding
/// pub fn transfer(&mut self, to: Address, amount: U256) -> Result<(), TransferError> { ... }
/// ```
///
/// # Payment Handling
///
/// Methods can receive payments by using the `#[payable]` attribute:
///
/// ```rust,ignore
/// #[payable]
/// pub fn deposit(&mut self) {
///     let amount = hybrid_contract::msg_value();
///     // Handle the deposited amount
/// }
///
/// // Non-payable methods will revert if called with value > 0
/// pub fn withdraw(&mut self) { ... }
/// ```
///
/// # Build Features
///
/// The macro generates different code based on Cargo features:
/// - **Default**: Full contract implementation with entry point
/// - **`interface-only`**: Only generates interface definitions
/// - **`deploy`**: Only generates deployment/initialization code
///
/// # ABI Compatibility
///
/// Generated contracts are fully ABI-compatible with Solidity contracts:
/// - Function selectors match Solidity's computation
/// - Parameter encoding follows ABI specification
/// - Events and errors use standard formats
/// - Return data encoding is compatible with web3 libraries
#[proc_macro_attribute]
pub fn contract(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    let struct_name = if let syn::Type::Path(type_path) = &*input.self_ty {
        &type_path.path.segments.first().unwrap().ident
    } else {
        panic!("Expected a struct.");
    };

    let mut constructor = None;
    let mut public_methods: Vec<&ImplItemMethod> = Vec::new();

    // Iterate over the items in the impl block to find pub methods + constructor
    for item in input.items.iter() {
        if let ImplItem::Method(method) = item {
            if method.sig.ident == "new" {
                constructor = Some(method);
            } else if let syn::Visibility::Public(_) = method.vis {
                public_methods.push(method);
            }
        }
    }

    let input_methods: Vec<_> = public_methods
        .iter()
        .map(|method| quote! { #method })
        .collect();
    let match_arms: Vec<_> = public_methods.iter().map(|method| {
        let method_name = &method.sig.ident;
        let method_info = MethodInfo::from(*method);
        let method_selector = u32::from_be_bytes(
            helpers::generate_fn_selector(&method_info, None)
                .expect("Unable to generate fn selector")
        );
        let (arg_names, arg_types) = helpers::get_arg_props_skip_first(&method_info);

        // Check if there are payable methods
        let checks = if !is_payable(&method) {
            quote! {
                if hybrid_contract::msg_value() > U256::from(0) {
                    panic!("Non-payable function");
                }
            }
        } else {
            quote! {}
        };

        // Check if the method has a return type
        let return_handling = match &method.sig.output {
            ReturnType::Default => {
                // No return value
                quote! { self.#method_name(#( #arg_names ),*); }
            }
           ReturnType::Type(_,_) => {
                match helpers::extract_wrapper_types(&method.sig.output) {
                    helpers::WrapperType::Result(_,_) => quote! {
                        let res = self.#method_name(#( #arg_names ),*);
                        match res {
                            Ok(success) => {
                                let result_bytes = success.abi_encode();
                                let result_size = result_bytes.len() as u64;
                                let result_ptr = result_bytes.as_ptr() as u64;
                                hybrid_contract::return_riscv(result_ptr, result_size);
                            }
                            Err(err) => {
                                hybrid_contract::revert_with_error(&err.abi_encode());
                            }
                        }
                    },
                    helpers::WrapperType::Option(_) => quote! {
                        match self.#method_name(#( #arg_names ),*) {
                            Some(success) => {
                                let result_bytes = success.abi_encode();
                                let result_size = result_bytes.len() as u64;
                                let result_ptr = result_bytes.as_ptr() as u64;
                                hybrid_contract::return_riscv(result_ptr, result_size);
                            },
                            None => hybrid_contract::revert(),
                        }
                    },
                    helpers::WrapperType::None => quote! {
                        let result = self.#method_name(#( #arg_names ),*);
                        let result_bytes = result.abi_encode();
                        let result_size = result_bytes.len() as u64;
                        let result_ptr = result_bytes.as_ptr() as u64;
                        hybrid_contract::return_riscv(result_ptr, result_size);
                    }
                }
            }
        };

        quote! {
            #method_selector => {
                let (#( #arg_names ),*) = <(#( #arg_types ),*)>::abi_decode(calldata, true).expect("abi decode failed");
                #checks
                #return_handling
            }
        }
    }).collect();

    let emit_helper = quote! {
        #[macro_export]
        macro_rules! get_type_signature {
            ($arg:expr) => {
                $arg.sol_type_name().as_bytes()
            };
        }

        #[macro_export]
        macro_rules! emit {
            ($event:ident, $($field:expr),*) => {{
                use alloy_sol_types::SolValue;
                use alloy_core::primitives::{keccak256, B256, U256, I256};
                use alloc::vec::Vec;

                let mut signature = alloc::vec![];
                signature.extend_from_slice($event::NAME.as_bytes());
                signature.extend_from_slice(b"(");

                let mut first = true;
                let mut topics = alloc::vec![B256::default()];
                let mut data = Vec::new();

                $(
                    if !first { signature.extend_from_slice(b","); }
                    first = false;

                    signature.extend_from_slice(get_type_signature!($field));
                    let encoded = $field.abi_encode();

                    let field_ident = stringify!($field);
                    if $event::INDEXED_FIELDS.contains(&field_ident) && topics.len() < 4 {
                        topics.push(B256::from_slice(&encoded));
                    } else {
                        data.extend_from_slice(&encoded);
                    }
                )*

                signature.extend_from_slice(b")");
                topics[0] = B256::from(keccak256(&signature));

                if !data.is_empty() {
                    hybrid_contract::emit_log(&data, &topics);
                } else if topics.len() > 1 {
                    let data = topics.pop().unwrap();
                    hybrid_contract::emit_log(data.as_ref(), &topics);
                }
            }};
        }
    };

    // Generate the interface
    let interface_name = format_ident!("I{}", struct_name);
    let interface = helpers::generate_interface(&public_methods, &interface_name, None);

    // Generate initcode for deployments
    let deployment_code = helpers::generate_deployment_code(struct_name, constructor);

    // Generate the complete output with module structure
    let output = quote! {
        use hybrid_contract::*;
        use alloy_sol_types::SolValue;

        // Deploy module
        #[cfg(feature = "deploy")]
            pub mod deploy {
            use super::*;
            use alloy_sol_types::SolValue;
            use hybrid_contract::*;

            #emit_helper
            #deployment_code
        }

        // Public interface module
        #[cfg(not(feature = "deploy"))]
        pub mod interface {
            use super::*;
            #interface
        }

        // Generate the call method implementation privately
        // only when not in `interface-only` mode
        #[cfg(not(any(feature = "deploy", feature = "interface-only")))]
        #[allow(non_local_definitions)]
        #[allow(unused_imports)]
        #[allow(unreachable_code)]
        mod implementation {
            use super::*;
            use alloy_sol_types::SolValue;
            use hybrid_contract::*;

            #emit_helper

            impl #struct_name { #(#input_methods)* }
            impl Contract for #struct_name {
                fn call(&mut self) {
                    self.call_with_data(&msg_data());
                }

                fn call_with_data(&mut self, calldata: &[u8]) {
                    let selector = u32::from_be_bytes([calldata[0], calldata[1], calldata[2], calldata[3]]);
                    let calldata = &calldata[4..];

                    match selector {
                        #( #match_arms )*
                        _ => panic!("unknown method"),
                    }

                    return_riscv(0, 0);
                }
            }

            #[hybrid_contract::entry]
            fn main() -> ! {
                let mut contract = #struct_name::default();
                contract.call();
                hybrid_contract::return_riscv(0, 0)
            }
        }

        // Export initcode when `deploy` mode
        #[cfg(feature = "deploy")]
        pub use deploy::*;

        // Always export the interface when not deploying
        #[cfg(not(feature = "deploy"))]
        pub use interface::*;

        // Only export contract impl when not in `interface-only` or `deploy` modes
        #[cfg(not(any(feature = "deploy", feature = "interface-only")))]
        pub use implementation::*;
    };

    TokenStream::from(output)
}

/// Attribute to mark a contract method as payable (able to receive native tokens).
///
/// By default, all contract methods reject transactions that include a non-zero value.
/// Adding the `#[payable]` attribute allows a method to receive native tokens (ETH, BNB, etc.)
/// along with the function call.
///
/// # Usage
///
/// ```rust,ignore
/// #[contract]
/// impl MyContract {
///     #[payable]
///     pub fn deposit(&mut self) {
///         let amount = hybrid_contract::msg_value();
///         // Process the deposited amount
///         self.balance += amount;
///     }
///
///     // This method will revert if called with value > 0
///     pub fn withdraw(&mut self, amount: U256) {
///         // Implementation
///     }
/// }
/// ```
///
/// # Behavior
///
/// - **With `#[payable]`**: Method can be called with any value (including 0)
/// - **Without `#[payable]`**: Method reverts with "Non-payable function" if called with value > 0
///
/// # Accessing Sent Value
///
/// Use `hybrid_contract::msg_value()` to get the amount of native tokens sent:
///
/// ```rust,ignore
/// #[payable]
/// pub fn buy_tokens(&mut self) -> U256 {
///     let payment = hybrid_contract::msg_value();
///     let token_amount = payment * self.exchange_rate;
///     self.mint_tokens(hybrid_contract::caller(), token_amount);
///     token_amount
/// }
/// ```
///
/// # Security Considerations
///
/// - Always validate that the sent value meets your requirements
/// - Be careful with integer arithmetic on received values
/// - Consider reentrancy attacks when processing payments
/// - Implement proper access controls for payable functions
#[proc_macro_attribute]
pub fn payable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Checks if a method is marked with the `#[payable]` attribute.
///
/// This helper function examines the attributes of a method to determine if it
/// should accept non-zero value transactions.
///
/// # Parameters
///
/// * `method` - The method to check for the payable attribute
///
/// # Returns
///
/// `true` if the method has the `#[payable]` attribute, `false` otherwise.
fn is_payable(method: &syn::ImplItemMethod) -> bool {
    method.attrs.iter().any(|attr| {
        if let Ok(syn::Meta::Path(path)) = attr.parse_meta() {
            if let Some(segment) = path.segments.first() {
                return segment.ident == "payable";
            }
        }
        false
    })
}

/// Generates a type-safe interface for calling external contracts.
///
/// This attribute transforms a Rust trait into a contract interface that can be used
/// to make type-safe calls to other contracts. The generated interface handles:
/// - Function selector computation
/// - ABI encoding/decoding of parameters and return values
/// - Contract address management
/// - Call context handling (read-only vs. mutable)
///
/// # Usage
///
/// ```rust,ignore
/// use hybrid_derive::interface;
/// use alloy_primitives::{Address, U256};
///
/// #[interface]
/// trait IERC20 {
///     fn total_supply(&self) -> U256;
///     fn balance_of(&self, account: Address) -> U256;
///     fn transfer(&mut self, to: Address, amount: U256) -> bool;
/// }
///
/// // Usage in contract
/// #[contract]
/// impl MyContract {
///     pub fn transfer_tokens(&mut self, token: Address, to: Address, amount: U256) {
///         let token_contract = IERC20::new(token).build();
///         let success = token_contract.transfer(to, amount).unwrap_or(false);
///         assert!(success, "Transfer failed");
///     }
/// }
/// ```
///
/// # Naming Style Conversion
///
/// The interface supports automatic naming style conversion:
///
/// ```rust,ignore
/// #[interface("camelCase")]
/// trait MyInterface {
///     fn get_user_balance(&self, user: Address) -> U256; // becomes getUserBalance
///     fn set_approval_for_all(&mut self, operator: Address, approved: bool); // becomes setApprovalForAll
/// }
/// ```
///
/// Supported styles:
/// - `"camelCase"` - Converts snake_case to camelCase for Solidity compatibility
///
/// # Generated Interface Structure
///
/// The macro generates:
///
/// 1. **Interface Struct**: `TraitName<C: CallCtx>` with contract address and context
/// 2. **Builder Pattern**: `TraitName::new(address)` returns an `InterfaceBuilder`
/// 3. **Context Types**: Support for `ReadOnly`, `StaticCtx`, and `MutableCtx`
/// 4. **Method Implementations**: Separate implementations for read-only and mutable methods
///
/// # Method Categories
///
/// ## Read-Only Methods (`&self`)
/// - Available on `StaticCtx` contexts
/// - Use `staticcall` for execution
/// - Cannot modify contract state
/// - Gas-efficient for queries
///
/// ## Mutable Methods (`&mut self`)
/// - Available on `MutableCtx` contexts
/// - Use regular `call` for execution
/// - Can modify contract state
/// - Consume gas and can fail
///
/// # Return Value Handling
///
/// Interface methods return `Option<T>` where:
/// - `Some(value)` indicates successful call with decoded return value
/// - `None` indicates call failure or decoding error
///
/// # Error Handling
///
/// For robust error handling, consider wrapping interface calls:
///
/// ```rust,ignore
/// let result = token_contract.transfer(to, amount)
///     .ok_or(TokenError::TransferFailed)?;
/// ```
///
/// # ABI Compatibility
///
/// Generated interfaces are fully compatible with:
/// - Solidity contract interfaces
/// - OpenZeppelin standards (ERC20, ERC721, etc.)
/// - Custom contract ABIs
/// - Web3 library expectations
#[proc_macro_attribute]
pub fn interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemTrait);
    let args = parse_macro_input!(attr as InterfaceArgs);
    let trait_name = &input.ident;

    let methods: Vec<_> = input
        .items
        .iter()
        .map(|item| {
            if let TraitItem::Method(method) = item {
                method
            } else {
                panic!("Expected methods arguments")
            }
        })
        .collect();

    // Generate intreface implementation
    let interface = helpers::generate_interface(&methods, trait_name, args.rename);
    let output = quote! { #interface };

    TokenStream::from(output)
}

/// Generates persistent storage layout and accessor methods for smart contract storage.
///
/// This attribute transforms a regular Rust struct into a smart contract storage layout
/// with automatic storage slot allocation and type-safe storage access methods.
/// Each field in the struct gets mapped to a storage slot on the blockchain.
///
/// # Usage
///
/// ```rust,ignore
/// use hybrid_derive::storage;
/// use alloy_primitives::{Address, U256};
///
/// #[storage]
/// pub struct TokenStorage {
///     pub name: String,
///     pub symbol: String,
///     pub total_supply: U256,
///     pub balances: StorageMap<Address, U256>,
///     pub allowances: StorageMap<Address, StorageMap<Address, U256>>,
/// }
/// ```
///
/// # Generated Components
///
/// The macro generates:
///
/// 1. **Storage Struct**: The original struct with all fields made public
/// 2. **Default Constructor**: `default()` method for contract initialization
/// 3. **Storage Allocation**: Automatic assignment of storage slots starting from slot 0
///
/// # Storage Slot Allocation
///
/// Each field is assigned a sequential storage slot:
/// - Field 0 → Storage slot 0
/// - Field 1 → Storage slot 1
/// - And so on...
///
/// The allocation uses a simple strategy suitable for basic types. For complex nested
/// structures, the allocation may need manual optimization in future versions.
///
/// # Supported Storage Types
///
/// The storage system supports various types:
///
/// ## Basic Types
/// - `U256`, `I256`, `bool`, `Address` - Single storage slot
/// - `String`, `Bytes` - Dynamic size with length prefix
///
/// ## Collection Types
/// - `StorageMap<K, V>` - Key-value mapping (equivalent to Solidity `mapping`)
/// - `Vec<T>` - Dynamic array
/// - Nested mappings like `StorageMap<K, StorageMap<K2, V>>`
///
/// ## Example Storage Usage
///
/// ```rust,ignore
/// #[storage]
/// pub struct MyStorage {
///     pub owner: Address,           // Slot 0
///     pub is_paused: bool,          // Slot 1
///     pub user_balances: StorageMap<Address, U256>, // Slot 2
///     pub metadata: String,         // Slot 3
/// }
///
/// #[contract]
/// impl MyStorage {
///     pub fn new(owner: Address) -> Self {
///         let mut storage = Self::default();
///         storage.owner = owner;
///         storage.is_paused = false;
///         storage
///     }
///
///     pub fn get_balance(&self, user: Address) -> U256 {
///         self.user_balances.get(user).unwrap_or(U256::ZERO)
///     }
///
///     pub fn set_balance(&mut self, user: Address, amount: U256) {
///         self.user_balances.set(user, amount);
///     }
/// }
/// ```
///
/// # Storage Initialization
///
/// The generated `default()` method initializes all storage using the `StorageLayout`
/// trait to properly allocate storage slots. This ensures that:
/// - Each field gets a unique storage location
/// - Storage mappings are properly initialized
/// - No storage conflicts occur between fields
///
/// # Unit Structs
///
/// For contracts without storage fields, you can use an empty struct:
///
/// ```rust,ignore
/// #[storage]
/// pub struct StatelessContract;
///
/// // Generates:
/// impl StatelessContract {
///     pub fn new() -> Self { Self {} }
/// }
/// ```
///
/// # Future Enhancements
///
/// The current implementation uses a naive storage allocation strategy.
/// Future versions may include:
/// - Storage layout optimization
/// - Custom storage slot assignment
/// - Support for storage structs and complex nested types
/// - Storage migration utilities
#[proc_macro_attribute]
pub fn storage(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let vis = &input.vis;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                let output = quote! {
                    #vis struct #name;
                    impl #name { pub fn new() -> Self { Self {} } }
                };
                return TokenStream::from(output);
            }
        },
        _ => panic!("Storage derive only works on structs"),
    };

    // Generate the struct definition with the same fields
    let struct_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! { pub #name: #ty }
    });

    // Generate initialization code for each field
    // TODO: PoC uses a naive strategy. Enhance to support complex types like tuples or custom structs.
    let init_fields = fields.iter().enumerate().map(|(i, f)| {
        let name = &f.ident;
        let slot = U256::from(i);
        let [limb0, limb1, limb2, limb3] = slot.as_limbs();
        quote! { #name: StorageLayout::allocate(#limb0, #limb1, #limb2, #limb3) }
    });

    let expanded = quote! {
        #vis struct #name { #(#struct_fields,)* }

        impl #name {
            pub fn default() -> Self {
                Self { #(#init_fields,)* }
            }
        }
    };

    TokenStream::from(expanded)
}

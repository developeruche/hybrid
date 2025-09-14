# API Documentation - Hybrid Derive Helpers

This document provides detailed API documentation for the helper functions and types used internally by the `hybrid-derive` procedural macros.

## Table of Contents

- [Core Types](#core-types)
- [Method Information](#method-information)
- [Type Conversion](#type-conversion)
- [Function Selectors](#function-selectors)
- [Interface Generation](#interface-generation)
- [Code Generation](#code-generation)

## Core Types

### `MethodInfo<'a>`

A unified representation of method information extracted from both `ImplItemMethod` and `TraitItemMethod`.

```rust
pub struct MethodInfo<'a> {
    name: &'a Ident,
    args: Vec<syn::FnArg>,
    return_type: &'a ReturnType,
}
```

#### Methods

- `is_mutable(&self) -> bool` - Returns true if the method takes `&mut self`

#### Trait Implementations

- `From<&'a ImplItemMethod>` - Convert from implementation method
- `From<&'a TraitItemMethod>` - Convert from trait method

### `InterfaceArgs`

Configuration for interface generation attributes.

```rust
pub struct InterfaceArgs {
    pub rename: Option<InterfaceNamingStyle>,
}
```

### `InterfaceNamingStyle`

Enumeration of supported naming conversion styles.

```rust
pub enum InterfaceNamingStyle {
    CamelCase,
}
```

### `WrapperType`

Represents different wrapper types for method return values.

```rust
pub enum WrapperType {
    Result(TokenStream, TokenStream),  // Ok type, Error type
    Option(TokenStream),               // Inner type
    None,                             // No wrapper
}
```

## Method Information

### `get_arg_props_skip_first`

Extracts argument names and types from a method, skipping the first argument (typically `self`).

```rust
pub fn get_arg_props_skip_first<'a>(
    method: &'a MethodInfo<'a>
) -> (Vec<Ident>, Vec<&'a syn::Type>)
```

**Returns:** Tuple of (argument names, argument types)

**Usage:**
```rust
let method_info = MethodInfo::from(method);
let (arg_names, arg_types) = get_arg_props_skip_first(&method_info);
```

### `get_arg_props_all`

Extracts argument names and types from a method, including all arguments.

```rust
pub fn get_arg_props_all<'a>(
    method: &'a MethodInfo<'a>
) -> (Vec<Ident>, Vec<&'a syn::Type>)
```

**Returns:** Tuple of (argument names, argument types)

## Type Conversion

### `rust_type_to_sol_type`

Converts Rust types to their Solidity ABI equivalent types.

```rust
pub fn rust_type_to_sol_type(ty: &Type) -> Result<DynSolType, &'static str>
```

**Supported Conversions:**

| Rust Type | Solidity Type | Notes |
|-----------|---------------|-------|
| `Address` | `address` | 20-byte Ethereum address |
| `Function` | `function` | Function pointer |
| `bool`, `Bool` | `bool` | Boolean value |
| `String`, `str` | `string` | Dynamic string |
| `Bytes` | `bytes` | Dynamic byte array |
| `B1`-`B32` | `bytes1`-`bytes32` | Fixed-size byte arrays |
| `U8`-`U256` | `uint8`-`uint256` | Unsigned integers (multiples of 8) |
| `I8`-`I256` | `int8`-`int256` | Signed integers (multiples of 8) |
| `Vec<T>` | `T[]` | Dynamic array |
| `[T; N]` | `T[N]` | Fixed-size array |
| `(T1, T2, ...)` | `(T1, T2, ...)` | Tuple |

**Examples:**
```rust
use syn::parse_quote;

let rust_type = parse_quote!(Vec<U256>);
let sol_type = rust_type_to_sol_type(&rust_type).unwrap();
assert_eq!(sol_type, DynSolType::Array(Box::new(DynSolType::Uint(256))));
```

### `extract_wrapper_types`

Analyzes a return type to determine if it's wrapped in `Result`, `Option`, or neither.

```rust
pub fn extract_wrapper_types(return_type: &ReturnType) -> WrapperType
```

**Examples:**
```rust
// Result<T, E> -> WrapperType::Result(T_tokens, E_tokens)
// Option<T> -> WrapperType::Option(T_tokens)  
// T -> WrapperType::None
```

## Function Selectors

### `generate_fn_selector`

Generates a 4-byte function selector for a method based on its signature.

```rust
pub fn generate_fn_selector(
    method: &MethodInfo,
    style: Option<InterfaceNamingStyle>
) -> Option<[u8; 4]>
```

**Parameters:**
- `method` - Method information
- `style` - Optional naming style conversion (e.g., snake_case to camelCase)

**Returns:** 4-byte selector or None if generation fails

**Selector Generation Process:**
1. Extract method name and apply naming style if specified
2. Convert argument types to Solidity type names
3. Create signature string: `"methodName(type1,type2,...)"`
4. Compute Keccak-256 hash of signature
5. Return first 4 bytes as selector

**Examples:**
```rust
// transfer(address,uint256) -> 0xa9059cbb
let selector = generate_fn_selector(&method_info, None).unwrap();

// get_balance -> getBalance with camelCase style
let selector = generate_fn_selector(&method_info, Some(InterfaceNamingStyle::CamelCase));
```

## Interface Generation

### `generate_interface`

Generates a complete interface implementation from a collection of methods.

```rust
pub fn generate_interface<T>(
    methods: &[&T],
    interface_name: &Ident,
    interface_style: Option<InterfaceNamingStyle>
) -> TokenStream
where
    for<'a> MethodInfo<'a>: From<&'a T>,
```

**Parameters:**
- `methods` - Collection of methods (can be `ImplItemMethod` or `TraitItemMethod`)
- `interface_name` - Name for the generated interface struct
- `interface_style` - Optional naming style for method conversion

**Generated Components:**
1. Interface struct with address and phantom type parameter
2. `InitInterface` implementation for creating instances
3. Context type conversions (`ReadOnly`, `MutableCtx`, `StaticCtx`)
4. Method implementations separated by mutability
5. Contract call encoding and execution

**Example Generated Interface:**
```rust
pub struct IToken<C: CallCtx> {
    address: Address,
    _ctx: PhantomData<C>
}

impl InitInterface for IToken<ReadOnly> {
    fn new(address: Address) -> InterfaceBuilder<Self> { ... }
}

impl<C: StaticCtx> IToken<C> {
    pub fn balance_of(&self, account: Address) -> Option<U256> { ... }
}

impl<C: MutableCtx> IToken<C> {
    pub fn transfer(&mut self, to: Address, amount: U256) -> Option<bool> { ... }
}
```

## Code Generation

### `generate_deployment_code`

Generates deployment/initialization code for contracts.

```rust
pub fn generate_deployment_code(
    struct_name: &Ident,
    constructor: Option<&ImplItemMethod>
) -> TokenStream
```

**Parameters:**
- `struct_name` - Name of the contract struct
- `constructor` - Optional constructor method (`new` function)

**Generated Code:**
1. Constructor argument decoding (if constructor exists)
2. Contract initialization
3. Runtime code return logic

**Example Output:**
```rust
#[no_mangle]
pub extern "C" fn main() -> ! {
    // Decode constructor arguments
    let calldata = hybrid_contract::msg_data();
    let (initial_supply, owner) = <(U256, Address)>::abi_decode(&calldata, true)
        .expect("Failed to decode constructor args");
    
    // Initialize contract
    Token::new(initial_supply, owner);
    
    // Return runtime code
    let runtime: &[u8] = include_bytes!("../target/riscv64imac-unknown-none-elf/release/runtime");
    let mut prepended_runtime = Vec::with_capacity(1 + runtime.len());
    prepended_runtime.push(0xff);
    prepended_runtime.extend_from_slice(runtime);
    
    let result_ptr = prepended_runtime.as_ptr() as u64;
    let result_len = prepended_runtime.len() as u64;
    hybrid_contract::return_riscv(result_ptr, result_len);
}
```

## Utility Functions

### `to_camel_case`

Converts snake_case identifiers to camelCase.

```rust
fn to_camel_case(s: String) -> String
```

**Examples:**
```rust
assert_eq!(to_camel_case("get_balance".to_string()), "getBalance");
assert_eq!(to_camel_case("transfer_from".to_string()), "transferFrom");
```

## Error Handling

All helper functions use standard Rust error handling patterns:

- Functions that can fail return `Result<T, E>` or `Option<T>`
- String conversion errors include the invalid input for debugging
- Type conversion errors provide descriptive messages about what went wrong

## Testing

The helpers module includes comprehensive tests covering:

- Type conversion for all supported Rust/Solidity type mappings
- Function selector generation for various method signatures
- Naming style conversions
- Edge cases and error conditions
- ERC-20 and ERC-721 standard compatibility

Run tests with:
```bash
cargo test helpers
```

//! # Hybrid Contract Library
//!
//! A comprehensive smart contract development library for the Hybrid VM, providing
//! Rust-based smart contract functionality with Solidity-like abstractions and
//! EVM compatibility.
//!
//! ## Overview
//!
//! This library enables developers to write smart contracts in Rust that run on the
//! Hybrid VM, a RISC-V based execution environment that provides EVM compatibility
//! while leveraging Rust's safety and performance characteristics.
//!
//! ## Key Features
//!
//! - **Storage Abstractions**: Solidity-like storage types (`Slot`, `Mapping`)
//! - **Contract Interactions**: Type-safe contract calling and deployment
//! - **Event System**: Blockchain event emission with ABI encoding
//! - **Environment Access**: Block and transaction information
//! - **Memory Management**: Custom bump allocator for deterministic allocation
//! - **Error Handling**: Contract reversion with structured error data
//!
//! ## Architecture
//!
//! The library is built around several core concepts:
//!
//! ### Storage System (`hstd` module)
//! Provides persistent storage abstractions:
//! - `Slot<T>`: Single storage slots for any ABI-encodable type
//! - `Mapping<K, V>`: Key-value mappings with automatic key derivation
//!
//! ### Contract System
//! - `Contract` trait: Entry point for contract execution
//! - `Deployable` trait: Contract deployment with constructor arguments
//! - Type-safe interfaces with context checking (ReadOnly vs ReadWrite)
//!
//! ### Environment Access
//! - Block information: timestamp, number, base fee, gas limit, chain ID
//! - Transaction information: gas price, origin address
//! - Message context: sender, value, calldata
//!
//! ### System Integration
//! - RISC-V system calls for EVM operations (SLOAD, SSTORE, CALL, etc.)
//! - Custom panic handler with contract reversion
//! - Memory-mapped calldata access
//! - Global bump allocator for deterministic memory management
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! #![no_std]
//! #![no_main]
//!
//! use hybrid_contract::*;
//! use hybrid_contract::hstd::{Slot, Mapping};
//! use alloy_core::primitives::{Address, U256};
//!
//! // Define contract storage
//! struct TokenStorage {
//!     total_supply: Slot<U256>,
//!     balances: Mapping<Address, U256>,
//!     allowances: Mapping<Address, Mapping<Address, U256>>,
//! }
//!
//! // Implement contract logic
//! impl Contract for TokenStorage {
//!     fn call(&mut self) {
//!         let sig = msg_sig();
//!         match sig {
//!             [0xa9, 0x05, 0x9c, 0xbb] => self.balance_of(),
//!             [0x23, 0xb8, 0x72, 0xdd] => self.transfer(),
//!             _ => revert(),
//!         }
//!     }
//!
//!     fn call_with_data(&mut self, calldata: &[u8]) {
//!         // Handle specific calldata
//!     }
//! }
//!
//! // Contract entry point
//! #[entry]
//! fn main() -> ! {
//!     let mut contract = TokenStorage::default();
//!     contract.call();
//! }
//! ```
//!
//! ## Safety and Determinism
//!
//! This library is designed for blockchain environments where determinism and safety
//! are critical:
//! - All operations are deterministic across different machines
//! - Memory allocation uses a bump allocator with no deallocation
//! - Integer arithmetic should be checked to prevent overflow/underflow
//! - All external calls are explicit and trackable
//!
//! ## Compatibility
//!
//! The library maintains compatibility with:
//! - Ethereum ABI encoding/decoding standards
//! - EVM storage layout conventions
//! - Solidity event and error formats
//! - Standard blockchain tooling and interfaces

#![no_std]
#![no_main]
#![feature(alloc_error_handler, maybe_uninit_write_slice, round_char_boundary)]

use alloy_core::primitives::{Address, U256};
use core::{arch::asm, fmt::Write, panic::PanicInfo, slice};
extern crate alloc as ext_alloc;

mod allocator;
pub mod env;
pub mod hstd;
pub mod tx;

pub mod create;
pub use create::Deployable;

pub mod error;
pub use error::{revert, revert_with_error, Error};

pub mod log;
pub use log::{emit_log, Event};

pub mod call;
pub use call::*;

/// Memory address where calldata is mapped in the contract's address space.
/// The first 8 bytes contain the calldata length, followed by the actual calldata.
const CALLDATA_ADDRESS: usize = 0x8000_0000;

pub use riscv_rt::entry;

/// Creates a slice from raw memory address and length.
///
/// This function provides unsafe access to memory at a specific address, which is
/// necessary for accessing memory-mapped regions like calldata in the contract
/// execution environment.
///
/// # Arguments
/// * `address` - The memory address to create a slice from
/// * `length` - The length of the slice in bytes
///
/// # Returns
/// A static slice referencing the memory region
///
/// # Safety
/// This function is unsafe because:
/// - The caller must ensure the memory region is valid and accessible
/// - The memory must remain valid for the 'static lifetime
/// - The caller must ensure the memory contains valid data for the slice type
/// - No other code should mutate the memory while the slice exists
///
/// # Usage
/// This is primarily used internally for accessing memory-mapped calldata:
/// ```rust,no_run
/// let calldata = unsafe { slice_from_raw_parts(CALLDATA_ADDRESS + 8, length) };
/// ```
pub unsafe fn slice_from_raw_parts(address: usize, length: usize) -> &'static [u8] {
    slice::from_raw_parts(address as *const u8, length)
}

/// Global panic handler for the contract execution environment.
///
/// This panic handler is called whenever a panic occurs in the contract code.
/// Instead of terminating the process (as would happen in a normal Rust program),
/// it reverts the contract execution with error information.
///
/// # Behavior
/// 1. Captures the panic message and formats it
/// 2. Reverts the contract with the panic message as error data
/// 3. If the panic handler itself panics, reverts with a generic message
///
/// # Double-Panic Protection
/// The handler includes protection against recursive panics by tracking whether
/// a panic is already being handled. This prevents infinite loops if the panic
/// handling code itself panics.
///
/// # Error Data
/// The panic message is included in the revert data, making it available to:
/// - External callers who can decode the error
/// - Development tools and debuggers
/// - Transaction simulators and tracers
///
/// # Safety
/// This function is marked unsafe because it accesses static mutable state
/// (`IS_PANICKING`) without synchronization. This is acceptable in the
/// single-threaded contract execution environment.
#[panic_handler]
unsafe fn panic(info: &PanicInfo<'_>) -> ! {
    static mut IS_PANICKING: bool = false;

    if !IS_PANICKING {
        IS_PANICKING = true;

        // Capture the panic info msg
        let mut message = ext_alloc::string::String::new();
        let _ = write!(message, "{:?}", info.message());

        // Convert to bytes and revert
        let msg = message.into_bytes();
        revert_with_error(&msg);
    } else {
        revert_with_error("Panic handler has panicked!".as_bytes())
    }
}

use hybrid_syscalls::Syscall;

/// Returns execution control to the caller with specified return data.
///
/// This function terminates the current contract execution and returns control
/// to the caller with the specified return data. This corresponds to the EVM
/// RETURN opcode and is used to complete successful contract execution.
///
/// # Arguments
/// * `addr` - Memory address containing the return data
/// * `offset` - Size of the return data in bytes
///
/// # Behavior
/// - Terminates contract execution immediately
/// - Returns the specified data to the caller
/// - Does not revert the transaction (unlike `revert`)
/// - All state changes made during execution are preserved
///
/// # Usage
/// This is typically used internally by the contract framework when a function
/// completes successfully and needs to return data to the caller.
///
/// # Safety
/// Uses inline RISC-V assembly to make a system call. The caller must ensure
/// that the memory region from `addr` to `addr + offset` contains valid data.
///
/// # Note
/// This function never returns - it uses the `!` return type to indicate divergence.
pub fn return_riscv(addr: u64, offset: u64) -> ! {
    unsafe {
        asm!("ecall", in("a0") addr, in("a1") offset, in("t0") u8::from(Syscall::Return));
    }
    unreachable!()
}

/// Reads a 256-bit word from contract storage.
///
/// This function corresponds to the EVM SLOAD opcode and reads a value from the
/// contract's persistent storage at the specified key. Storage keys are 256-bit
/// values that identify unique storage slots.
///
/// # Arguments
/// * `key` - The storage key to read from (256-bit value)
///
/// # Returns
/// The 256-bit value stored at the specified key, or zero if no value has been stored
///
/// # Usage
/// This is typically used by higher-level storage abstractions like `Slot` and
/// `Mapping` rather than being called directly by contract code.
///
/// # Examples
/// ```rust,no_run
/// let key = U256::from(0x1234);
/// let value = sload(key);
/// ```
///
/// # Gas Cost
/// SLOAD operations have variable gas costs depending on whether the storage
/// slot has been accessed before in the current transaction (EIP-2929).
pub fn sload(key: U256) -> U256 {
    let key = key.as_limbs();
    let (val0, val1, val2, val3): (u64, u64, u64, u64);
    unsafe {
        asm!(
            "ecall",
            lateout("a0") val0, lateout("a1") val1, lateout("a2") val2, lateout("a3") val3,
            in("a0") key[0], in("a1") key[1], in("a2") key[2], in("a3") key[3],
            in("t0") u8::from(Syscall::SLoad));
    }
    U256::from_limbs([val0, val1, val2, val3])
}

/// Writes a 256-bit word to contract storage.
///
/// This function corresponds to the EVM SSTORE opcode and writes a value to the
/// contract's persistent storage at the specified key. The stored value will
/// persist across contract calls and transactions.
///
/// # Arguments
/// * `key` - The storage key to write to (256-bit value)
/// * `value` - The 256-bit value to store
///
/// # Usage
/// This is typically used by higher-level storage abstractions like `Slot` and
/// `Mapping` rather than being called directly by contract code.
///
/// # Examples
/// ```rust,no_run
/// let key = U256::from(0x1234);
/// let value = U256::from(42);
/// sstore(key, value);
/// ```
///
/// # Gas Costs
/// SSTORE has complex gas costs that depend on:
/// - Whether the storage slot is being set for the first time
/// - Whether the value is being changed from non-zero to zero (gas refund)
/// - Whether the slot was accessed before in the current transaction
/// - The current value versus the new value (EIP-2929, EIP-3529)
///
/// # State Changes
/// Storage modifications made by SSTORE are part of the transaction's state
/// changes and will be reverted if the transaction fails or reverts.
pub fn sstore(key: U256, value: U256) {
    let key = key.as_limbs();
    let value = value.as_limbs();

    unsafe {
        asm!(
            "ecall",
            in("a0") key[0], in("a1") key[1], in("a2") key[2], in("a3") key[3],
            in("a4") value[0], in("a5") value[1], in("a6") value[2], in("a7") value[3],
            in("t0") u8::from(Syscall::SStore)
        );
    }
}

/// Computes the Keccak-256 hash of data in memory.
///
/// This function corresponds to the EVM KECCAK256 (formerly SHA3) opcode and
/// computes the cryptographic hash of data stored in memory. Keccak-256 is
/// widely used in Ethereum for generating storage keys, event topics, and
/// other cryptographic operations.
///
/// # Arguments
/// * `offset` - Memory offset where the data to hash begins
/// * `size` - Size of the data to hash in bytes
///
/// # Returns
/// The Keccak-256 hash as a 256-bit value
///
/// # Usage
/// This function is used internally by storage systems (like mappings) and can
/// be used for custom cryptographic operations:
///
/// # Examples
/// ```rust,no_run
/// let data = b"Hello, World!";
/// let hash = keccak256(data.as_ptr() as u64, data.len() as u64);
/// ```
///
/// # Applications
/// - Generating mapping storage keys
/// - Computing event topic hashes
/// - Creating commit-reveal schemes
/// - Verifying data integrity
/// - Generating pseudo-random values (when combined with other inputs)
///
/// # Gas Cost
/// The gas cost is proportional to the amount of data being hashed, with
/// a base cost plus additional cost per word (32 bytes) of data.
pub fn keccak256(offset: u64, size: u64) -> U256 {
    let (first, second, third, fourth): (u64, u64, u64, u64);
    unsafe {
        asm!(
            "ecall",
            in("a0") offset,
            in("a1") size,
            lateout("a0") first,
            lateout("a1") second,
            lateout("a2") third,
            lateout("a3") fourth,
            in("t0") u8::from(Syscall::Keccak256)
        );
    }
    U256::from_limbs([first, second, third, fourth])
}

/// Returns the address of the immediate caller of the current contract.
///
/// This function corresponds to the EVM CALLER opcode and returns the address
/// that directly called the current contract. This could be an externally-owned
/// account (EOA) or another contract address.
///
/// # Returns
/// The caller's address as an `Address` (20 bytes)
///
/// # Usage
/// This is commonly used for access control and authentication:
///
/// ```rust,no_run
/// let caller = msg_sender();
/// if caller != owner_address {
///     revert(); // Only owner can call this function
/// }
/// ```
///
/// # Difference from `tx.origin`
/// - `msg.sender`: The immediate caller (can be a contract)
/// - `tx.origin`: The original transaction initiator (always an EOA)
///
/// For security reasons, prefer using `msg.sender` for authorization checks
/// rather than `tx.origin`, as it provides protection against certain attack
/// vectors involving intermediate contracts.
///
/// # Call Chain Example
/// ```text
/// EOA -> Contract A -> Contract B
///
/// In Contract B:
/// - msg.sender = Contract A
/// - tx.origin = EOA
/// ```
pub fn msg_sender() -> Address {
    let (first, second, third): (u64, u64, u64);
    unsafe {
        asm!("ecall", lateout("a0") first, lateout("a1") second, lateout("a2") third, in("t0") u8::from(Syscall::Caller));
    }
    let mut bytes = [0u8; 20];
    bytes[0..8].copy_from_slice(&first.to_be_bytes());
    bytes[8..16].copy_from_slice(&second.to_be_bytes());
    bytes[16..20].copy_from_slice(&third.to_be_bytes()[..4]);
    Address::from_slice(&bytes)
}

/// Returns the amount of wei sent with the current message call.
///
/// This function corresponds to the EVM CALLVALUE opcode and returns the amount
/// of ether (in wei) that was transferred to the contract with the current call.
/// This value is only non-zero if the contract function is marked as payable.
///
/// # Returns
/// The value sent with the call as a U256 (in wei)
///
/// # Usage
/// Used to implement payable functions and handle ether transfers:
///
/// ```rust,no_run
/// let payment = msg_value();
/// if payment == U256::ZERO {
///     revert(); // Function requires payment
/// }
///
/// // Process payment
/// balances[msg_sender()] += payment;
/// ```
///
/// # Payable Functions
/// For a contract to receive ether, it must explicitly check and handle `msg.value`:
/// - Non-payable functions should revert if `msg.value > 0`
/// - Payable functions should process the received ether appropriately
/// - Fallback/receive functions handle direct ether transfers
///
/// # Security Considerations
/// - Always validate payment amounts against expected values
/// - Be aware of reentrancy attacks when handling payments
/// - Consider integer overflow when adding to balances
/// - Implement proper refund mechanisms for overpayments
pub fn msg_value() -> U256 {
    let (first, second, third, fourth): (u64, u64, u64, u64);
    unsafe {
        asm!("ecall", lateout("a0") first, lateout("a1") second, lateout("a2") third, lateout("a3") fourth, in("t0") u8::from(Syscall::CallValue));
    }
    U256::from_limbs([first, second, third, fourth])
}

/// Returns the function selector (first 4 bytes) of the current call.
///
/// This function extracts the function selector from the calldata, which is used
/// to determine which function is being called. The selector is the first 4 bytes
/// of the keccak256 hash of the function signature.
///
/// # Returns
/// The 4-byte function selector as a byte array
///
/// # Usage
/// Used for function dispatching in contract implementations:
///
/// ```rust,no_run
/// let selector = msg_sig();
/// match selector {
///     [0xa9, 0x05, 0x9c, 0xbb] => self.balance_of(), // balanceOf(address)
///     [0x23, 0xb8, 0x72, 0xdd] => self.transfer(),   // transfer(address,uint256)
///     _ => revert(), // Unknown function
/// }
/// ```
///
/// # Function Signature Calculation
/// Function selectors are calculated as:
/// ```text
/// keccak256("functionName(paramType1,paramType2)")[0:4]
/// ```
///
/// Examples:
/// - `transfer(address,uint256)` -> `0x23b872dd`
/// - `balanceOf(address)` -> `0xa9059cbb`
/// - `approve(address,uint256)` -> `0x095ea7b3`
///
/// # Calldata Layout
/// ```text
/// [0:4]   - Function selector
/// [4:36]  - First parameter (32 bytes)
/// [36:68] - Second parameter (32 bytes)
/// ...
/// ```
pub fn msg_sig() -> [u8; 4] {
    let sig = unsafe { slice_from_raw_parts(CALLDATA_ADDRESS + 8, 4) };
    sig.try_into().unwrap()
}

/// Returns the complete calldata for the current message call.
///
/// This function provides access to the entire calldata sent with the current
/// contract call, including the function selector and all parameters. The data
/// is ABI-encoded according to Ethereum standards.
///
/// # Returns
/// A static slice containing the complete calldata
///
/// # Usage
/// Used for advanced calldata processing or when implementing proxy patterns:
///
/// ```rust,no_run
/// let calldata = msg_data();
///
/// // Extract function selector
/// let selector = &calldata[0..4];
///
/// // Extract parameters
/// let params = &calldata[4..];
///
/// // Forward call to another contract
/// let result = call_contract(target, 0, calldata, None);
/// ```
///
/// # Calldata Structure
/// ```text
/// Offset | Size | Description
/// -------|------|------------
/// 0      | 4    | Function selector
/// 4      | 32   | First parameter (padded to 32 bytes)
/// 36     | 32   | Second parameter (padded to 32 bytes)
/// ...    | ...  | Additional parameters
/// ```
///
/// # Memory Layout
/// The calldata is memory-mapped at a fixed address (`CALLDATA_ADDRESS`):
/// - First 8 bytes: Length of the calldata
/// - Remaining bytes: The actual calldata
///
/// # ABI Encoding
/// Parameters in calldata follow Ethereum ABI encoding rules:
/// - Static types are encoded in-place
/// - Dynamic types use offset-based encoding
/// - All values are padded to 32-byte boundaries
pub fn msg_data() -> &'static [u8] {
    let length = unsafe { slice_from_raw_parts(CALLDATA_ADDRESS, 8) };
    let length = u64::from_le_bytes([
        length[0], length[1], length[2], length[3], length[4], length[5], length[6], length[7],
    ]) as usize;
    unsafe { slice_from_raw_parts(CALLDATA_ADDRESS + 8, length) }
}

/// Default interrupt handler for unhandled RISC-V interrupts.
///
/// This function serves as a catch-all handler for any RISC-V interrupts that
/// are not explicitly handled by the runtime. In the contract execution
/// environment, unexpected interrupts typically indicate programming errors
/// or system issues.
///
/// # Behavior
/// The handler panics with a descriptive message, which will trigger the
/// contract's panic handler and cause the transaction to revert with error data.
///
/// # Usage
/// This function is automatically called by the RISC-V runtime when an
/// unhandled interrupt occurs. It should not be called directly by application code.
///
/// # Debugging
/// If this handler is triggered, it usually indicates:
/// - An unexpected system interrupt
/// - Incorrect interrupt vector configuration
/// - Hardware or emulation issues
/// - Runtime environment problems
#[allow(non_snake_case)]
#[no_mangle]
fn DefaultHandler() {
    panic!("default handler");
}

//! # Contract Calling Interface
//!
//! This module provides functionality for calling other smart contracts in the Hybrid VM environment.
//! It implements both regular calls and static calls, with support for type-safe contract interfaces
//! and context-aware method calls.
//!
//! ## Features
//! - Type-safe contract interfaces with compile-time context checking
//! - Support for both mutable (`call`) and read-only (`staticcall`) operations
//! - Automatic ABI encoding/decoding of call data and return values
//! - Context markers to prevent invalid operations (e.g., state changes in static contexts)
//!
//! ## Usage
//! ```rust,no_run
//! use hybrid_contract::call::*;
//! use alloy_core::primitives::Address;
//!
//! // Call another contract
//! let result = call_contract(
//!     Address::ZERO,
//!     0, // value in wei
//!     &call_data,
//!     Some(32) // expected return size
//! );
//! ```

extern crate alloc;
use alloc::vec::Vec;
use alloy_core::primitives::{Address, Bytes, U256};
use core::{arch::asm, marker::PhantomData};
use hybrid_syscalls::Syscall;

/// Marker type for read-only contract call contexts.
/// Used to enforce that only read operations are performed.
pub struct ReadOnly;

/// Marker type for read-write contract call contexts.
/// Allows both read and write operations.
pub struct ReadWrite;

/// Base trait for all call contexts.
pub trait CallCtx {}

/// Trait for static (read-only) call contexts.
/// Implementations of this trait can only perform read operations.
pub trait StaticCtx: CallCtx {}

/// Trait for mutable call contexts.
/// Implementations can perform both read and write operations.
pub trait MutableCtx: StaticCtx {}

impl CallCtx for ReadOnly {}
impl CallCtx for ReadWrite {}
impl StaticCtx for ReadOnly {}
impl StaticCtx for ReadWrite {}
impl MutableCtx for ReadWrite {}

/// Connects contract method context with call context types.
/// This trait ensures type safety by associating method signatures with their allowed contexts.
pub trait MethodCtx {
    type Allowed: CallCtx;
}
impl<'a, T> MethodCtx for &'a T {
    type Allowed = ReadOnly;
}
impl<'a, T> MethodCtx for &'a mut T {
    type Allowed = ReadWrite;
}

/// Builder for creating type-safe contract interfaces.
/// This struct helps construct contract interfaces with proper context checking.
pub struct InterfaceBuilder<I> {
    /// The contract address this interface will interact with
    pub address: Address,
    pub _phantom: PhantomData<I>,
}

/// Trait for contracts that can initialize their interface.
pub trait InitInterface: Sized {
    /// Creates a new interface builder for the given contract address.
    fn new(address: Address) -> InterfaceBuilder<Self>;
}

/// Trait for converting between different interface types.
/// This enables context-aware interface transformations.
pub trait IntoInterface<T> {
    fn into_interface(self) -> T;
}

impl<I> InterfaceBuilder<I> {
    /// Creates an interface with the specified method context.
    /// This ensures that the interface can only be used in the appropriate context
    /// (e.g., read-only methods in static contexts).
    pub fn with_ctx<M: MethodCtx, T>(self, _: M) -> T
    where
        I: IntoInterface<T>,
        M: MethodCtx<Allowed = T::Context>,
        T: FromBuilder,
    {
        let target_builder = InterfaceBuilder {
            address: self.address,
            _phantom: PhantomData,
        };
        T::from_builder(target_builder)
    }
}

/// Trait for types that can be constructed from an interface builder.
pub trait FromBuilder: Sized {
    /// The call context this type operates in
    type Context: CallCtx;
    /// Constructs the type from an interface builder
    fn from_builder(builder: InterfaceBuilder<Self>) -> Self;
}

/// Trait for contracts that can handle transaction calls.
/// This provides the entry points for contract execution.
pub trait Contract {
    /// Handle a contract call without explicit calldata
    fn call(&mut self);
    /// Handle a contract call with specific calldata
    fn call_with_data(&mut self, calldata: &[u8]);
}

/// Calls another contract and returns the response data.
///
/// This is a high-level wrapper that handles the complete call flow:
/// 1. Executes the contract call
/// 2. Retrieves the return data
/// 3. Returns it as a `Bytes` object
///
/// # Arguments
/// * `addr` - The address of the contract to call
/// * `value` - Amount of wei to send with the call
/// * `data` - The call data (typically ABI-encoded function call)
/// * `ret_size` - Expected size of return data (None to auto-detect)
///
/// # Returns
/// The return data from the called contract
pub fn call_contract(addr: Address, value: u64, data: &[u8], ret_size: Option<u64>) -> Bytes {
    // Perform the call without writing return data into (REVM) memory
    call(addr, value, data.as_ptr() as u64, data.len() as u64);
    // Load call output to memory
    handle_call_output(ret_size)
}

/// Low-level contract call via RISC-V system call.
///
/// This function performs the actual EVM CALL operation through a system call
/// to the Hybrid VM. It does not handle return data - use `call_contract` for
/// a complete solution.
///
/// # Arguments
/// * `addr` - The contract address to call
/// * `value` - Amount of wei to transfer
/// * `data_offset` - Memory offset of the call data
/// * `data_size` - Size of the call data in bytes
///
/// # Safety
/// Uses inline assembly and assumes the caller has prepared valid call data.
pub fn call(addr: Address, value: u64, data_offset: u64, data_size: u64) {
    let addr: U256 = addr.into_word().into();
    let addr = addr.as_limbs();
    unsafe {
        asm!(
            "ecall",
            in("a0") addr[0], in("a1") addr[1], in("a2") addr[2],
            in("a3") value, in("a4") data_offset, in("a5") data_size,
            in("t0") u8::from(Syscall::Call)
        );
    }
}

/// Performs a static call to another contract and returns the response data.
///
/// Static calls are read-only operations that cannot modify state or transfer value.
/// This is equivalent to the EVM STATICCALL opcode.
///
/// # Arguments
/// * `addr` - The address of the contract to call
/// * `value` - Should typically be 0 for static calls
/// * `data` - The call data (typically ABI-encoded function call)
/// * `ret_size` - Expected size of return data (None to auto-detect)
///
/// # Returns
/// The return data from the called contract
pub fn staticcall_contract(addr: Address, value: u64, data: &[u8], ret_size: Option<u64>) -> Bytes {
    // Perform the staticcall without writing return data into (REVM) memory
    staticcall(addr, value, data.as_ptr() as u64, data.len() as u64);
    // Load call output to memory
    handle_call_output(ret_size)
}

/// Handles the retrieval of return data from a contract call.
///
/// This function manages the process of copying return data from the VM's
/// internal buffer into Rust-managed memory, handling both the size detection
/// and efficient chunked copying.
///
/// # Arguments
/// * `ret_size` - Expected return data size, or None to auto-detect
///
/// # Returns
/// The return data as a `Bytes` object
fn handle_call_output(ret_size: Option<u64>) -> Bytes {
    // Figure out return data size + initialize memory location
    let ret_size = match ret_size {
        Some(size) => size,
        None => return_data_size(),
    };

    if ret_size == 0 {
        return Bytes::default();
    };

    let mut ret_data = Vec::with_capacity(ret_size as usize);
    ret_data.resize(ret_size as usize, 0);

    // Copy the return data from the interpreter's buffer
    let (offset, chunks, remainder) = (ret_data.as_ptr() as u64, ret_size / 32, ret_size % 32);

    // handle full chunks
    for i in 0..chunks {
        let step = i * 32;
        return_data_copy(offset + step, step, 32);
    }

    // handle potential last partial-chunk
    if remainder != 0 {
        let step = chunks * 32;
        return_data_copy(offset + step, step, remainder);
    };

    Bytes::from(ret_data)
}

/// Low-level static contract call via RISC-V system call.
///
/// This function performs the actual EVM STATICCALL operation through a system call
/// to the Hybrid VM. Static calls are read-only and cannot modify state.
///
/// # Arguments
/// * `addr` - The contract address to call
/// * `value` - Should typically be 0 for static calls
/// * `data_offset` - Memory offset of the call data
/// * `data_size` - Size of the call data in bytes
///
/// # Safety
/// Uses inline assembly and assumes the caller has prepared valid call data.
pub fn staticcall(addr: Address, value: u64, data_offset: u64, data_size: u64) {
    let addr: U256 = addr.into_word().into();
    let addr = addr.as_limbs();
    unsafe {
        asm!(
            "ecall",
            in("a0") addr[0], in("a1") addr[1], in("a2") addr[2],
            in("a3") value, in("a4") data_offset, in("a5") data_size,
            in("t0") u8::from(Syscall::StaticCall)
        );
    }
}

/// Gets the size of the return data from the last contract call.
///
/// This corresponds to the EVM RETURNDATASIZE opcode and is used to determine
/// how much data is available to copy from the return buffer.
///
/// # Returns
/// The size in bytes of available return data
pub fn return_data_size() -> u64 {
    let size: u64;
    unsafe {
        asm!( "ecall", lateout("a0") size, in("t0") u8::from(Syscall::ReturnDataSize));
    }

    size
}

/// Copies return data from the VM's buffer to a specified memory location.
///
/// This corresponds to the EVM RETURNDATACOPY opcode and is used to retrieve
/// the actual return data after a contract call.
///
/// # Arguments
/// * `dest_offset` - Destination memory offset to copy data to
/// * `res_offset` - Offset within the return data buffer to start copying from
/// * `res_size` - Number of bytes to copy
///
/// # Safety
/// The caller must ensure that the destination memory is properly allocated
/// and that the specified offsets and sizes are valid.
pub fn return_data_copy(dest_offset: u64, res_offset: u64, res_size: u64) {
    unsafe {
        asm!(
            "ecall",
            in("a0") dest_offset, in("a1") res_offset, in("a2") res_size, in("t0")
            u8::from(Syscall::ReturnDataCopy)
        );
    }
}

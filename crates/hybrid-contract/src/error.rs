//! # Error Handling and Revert Functionality
//!
//! This module provides error handling capabilities for smart contracts in the Hybrid VM.
//! It implements contract reversion functionality that allows contracts to abort execution
//! and optionally return error data to the caller.
//!
//! ## Features
//! - Contract reversion with custom error data
//! - ABI encoding/decoding support for structured errors
//! - Integration with Rust's panic system
//! - EVM-compatible revert behavior
//!
//! ## Usage
//! ```rust,no_run
//! use hybrid_contract::error::*;
//!
//! // Simple revert without data
//! revert();
//!
//! // Revert with custom error message
//! let error_data = b"Insufficient balance";
//! revert_with_error(error_data);
//!
//! // Custom error types can implement the Error trait
//! struct InsufficientBalance {
//!     required: u256,
//!     available: u256,
//! }
//! ```

extern crate alloc;
use crate::Syscall;
use alloc::vec::Vec;
use core::arch::asm;

/// Trait for custom error types that can be used with contract reversion.
///
/// This trait allows custom error types to be ABI-encoded for transmission
/// back to the caller when a contract reverts. Error types should typically
/// contain relevant information about what caused the failure.
///
/// # Examples
/// ```rust,no_run
/// use hybrid_contract::error::Error;
///
/// struct InsufficientBalance {
///     required: u256,
///     available: u256,
/// }
///
/// impl Error for InsufficientBalance {
///     fn abi_encode(&self) -> Vec<u8> {
///         // Implement ABI encoding for this error type
///         todo!()
///     }
///
///     fn abi_decode(bytes: &[u8], validate: bool) -> Self {
///         // Implement ABI decoding for this error type
///         todo!()
///     }
/// }
/// ```
pub trait Error {
    /// Encodes the error into ABI format for transmission.
    ///
    /// # Returns
    /// The ABI-encoded error data as a byte vector
    fn abi_encode(&self) -> Vec<u8>;

    /// Decodes error data from ABI format.
    ///
    /// # Arguments
    /// * `bytes` - The ABI-encoded error data
    /// * `validate` - Whether to perform validation during decoding
    ///
    /// # Returns
    /// The decoded error instance
    fn abi_decode(bytes: &[u8], validate: bool) -> Self;
}

/// Reverts the current contract execution without any error data.
///
/// This function immediately terminates the contract execution and reverts all
/// state changes made during the current call. No error data is returned to the caller.
/// This is equivalent to the EVM REVERT opcode with empty data.
///
/// # Examples
/// ```rust,no_run
/// if unauthorized_access {
///     revert(); // Simple revert without explanation
/// }
/// ```
///
/// # Note
/// This function never returns - it uses the `!` return type to indicate divergence.
pub fn revert() -> ! {
    revert_with_error(Vec::new().as_slice())
}
/// Reverts the current contract execution with custom error data.
///
/// This function immediately terminates the contract execution, reverts all state
/// changes made during the current call, and returns the provided error data to the caller.
/// This is equivalent to the EVM REVERT opcode with custom revert data.
///
/// The error data is typically ABI-encoded to provide structured information about
/// the failure reason, which can be decoded by the caller or development tools.
///
/// # Arguments
/// * `data` - The error data to return to the caller (typically ABI-encoded)
///
/// # Examples
/// ```rust,no_run
/// // Revert with a simple error message
/// let error_msg = b"Transfer failed: insufficient balance";
/// revert_with_error(error_msg);
///
/// // Revert with ABI-encoded structured error
/// let error = InsufficientBalance { required: 100, available: 50 };
/// revert_with_error(&error.abi_encode());
/// ```
///
/// # Safety
/// Uses inline RISC-V assembly to make a system call to the Hybrid VM.
/// The function never returns and will terminate contract execution.
///
/// # Note
/// This function never returns - it uses the `!` return type to indicate divergence.
pub fn revert_with_error(data: &[u8]) -> ! {
    let (offset, size) = (data.as_ptr() as u64, data.len() as u64);
    unsafe {
        asm!("ecall",
            in("a0") offset, in("a1") size,
            in("t0") u8::from(Syscall::Revert));
    }
    unreachable!()
}

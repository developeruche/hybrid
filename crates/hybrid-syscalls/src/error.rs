//! Error types for the hybrid-syscalls crate.
//!
//! This module defines error types that can occur during syscall operations,
//! including opcode parsing failures and string conversion errors.

use alloc::borrow::Cow;

/// Errors that can occur during syscall operations.
///
/// This enum represents all possible error conditions when working with
/// syscalls in the Hybrid framework. It provides detailed error information
/// to help with debugging and error handling in syscall processing code.
///
/// # Error Categories
///
/// ## Opcode Errors
/// - [`Error::UnknownOpcode`] - When an opcode doesn't map to any known syscall
///
/// ## String Parsing Errors
/// - [`Error::ParseError`] - When a string doesn't match any syscall name
///
/// # Usage in Error Handling
///
/// ```rust
/// use hybrid_syscalls::{Syscall, Error};
///
/// match Syscall::try_from(0xFF) {
///     Ok(syscall) => println!("Valid syscall: {}", syscall),
///     Err(Error::UnknownOpcode(code)) => {
///         eprintln!("Unknown opcode: 0x{:02x}", code);
///     },
///     Err(e) => eprintln!("Other error: {}", e),
/// }
/// ```
#[derive(Debug, thiserror_no_std::Error)]
pub enum Error {
    /// An unknown or unsupported syscall opcode was encountered.
    ///
    /// This error occurs when attempting to convert a byte value to a [`Syscall`]
    /// enum variant using [`TryFrom<u8>`], but the byte doesn't correspond to any
    /// known syscall opcode.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hybrid_syscalls::{Syscall, Error};
    ///
    /// let result = Syscall::try_from(0xFF); // Invalid opcode
    /// match result {
    ///     Err(Error::UnknownOpcode(code)) => {
    ///         assert_eq!(code, 0xFF);
    ///         println!("Unknown opcode: 0x{:02x}", code);
    ///     },
    ///     _ => unreachable!(),
    /// }
    /// ```
    ///
    /// # Debugging Tips
    ///
    /// When encountering this error:
    /// 1. Verify that the opcode is a valid EVM opcode
    /// 2. Check if the syscall has been implemented in the current version
    /// 3. Ensure the byte value wasn't corrupted during transmission/storage
    ///
    /// [`Syscall`]: crate::Syscall
    /// [`TryFrom<u8>`]: core::convert::TryFrom
    #[error("Unknown syscall opcode: 0x{0:02x}")]
    UnknownOpcode(u8),

    /// A string could not be parsed as a valid syscall name.
    ///
    /// This error occurs when using [`FromStr`] to parse a string into a
    /// [`Syscall`] enum variant, but the string doesn't match any of the
    /// canonical syscall names.
    ///
    /// # Field Details
    ///
    /// * `input` - The invalid input string that caused the parsing failure.
    ///   Uses [`Cow<'static, str>`] to efficiently handle both static and
    ///   owned strings without unnecessary allocations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hybrid_syscalls::{Syscall, Error};
    /// use std::str::FromStr;
    ///
    /// let result = Syscall::from_str("invalid_syscall");
    /// match result {
    ///     Err(Error::ParseError { input }) => {
    ///         println!("Invalid syscall name: {}", input);
    ///         assert_eq!(input, "invalid_syscall");
    ///     },
    ///     _ => unreachable!(),
    /// }
    /// ```
    ///
    /// # Common Causes
    ///
    /// - Typos in syscall names (`"caler"` instead of `"caller"`)
    /// - Case sensitivity (`"CALLER"` instead of `"caller"`)
    /// - Using alternative names (`"sha3"` instead of `"keccak256"`)
    /// - Including extra whitespace or characters
    ///
    /// # Valid Syscall Names
    ///
    /// For a complete list of valid syscall names, see the [`Syscall`] enum
    /// documentation or use the [`Display`] implementation to see canonical names.
    ///
    /// [`FromStr`]: core::str::FromStr
    /// [`Syscall`]: crate::Syscall
    /// [`Cow<'static, str>`]: alloc::borrow::Cow
    /// [`Display`]: core::fmt::Display
    #[error("Parse error for syscall string. Input: '{input}'")]
    ParseError { input: Cow<'static, str> },
}

//! # Event Logging Module
//!
//! This module provides functionality for emitting events from smart contracts in the Hybrid VM.
//! Events are a way for contracts to communicate that something happened on the blockchain to
//! the outside world, and they can be efficiently searched and filtered by external applications.
//!
//! ## Overview
//!
//! The event system in Hybrid VM follows the Ethereum event model:
//! - Events have topics (indexed parameters) for efficient filtering
//! - Events have data (non-indexed parameters) for additional information
//! - Events are stored in transaction logs and can be queried by external tools
//!
//! ## Features
//!
//! - Type-safe event emission with automatic ABI encoding
//! - Support for up to 3 indexed topics (following EVM limitations)
//! - Integration with Solidity-compatible event definitions
//! - Efficient topic and data encoding
//!
//! ## Usage
//!
//! ```rust,no_run
//! use hybrid_contract::log::{emit, Event};
//! use alloy_core::primitives::{Address, U256};
//!
//! struct Transfer {
//!     from: Address,
//!     to: Address,
//!     amount: U256,
//! }
//!
//! impl Event for Transfer {
//!     fn encode_log(&self) -> (Vec<u8>, Vec<[u8; 32]>) {
//!         // Implementation would encode topics and data
//!         todo!()
//!     }
//! }
//!
//! // Emit the event
//! let transfer_event = Transfer {
//!     from: Address::ZERO,
//!     to: user_address,
//!     amount: U256::from(1000),
//! };
//! emit(transfer_event);
//! ```

extern crate alloc;
use crate::Syscall;
use alloc::vec::Vec;
use alloy_core::primitives::B256;
use core::arch::asm;

/// Trait for types that can be emitted as blockchain events.
///
/// This trait defines the interface for custom event types that can be logged to the blockchain.
/// Events consist of topics (indexed parameters for filtering) and data (non-indexed parameters
/// for additional information).
///
/// # Event Structure
///
/// In the EVM event model:
/// - **Topics**: Up to 3 indexed parameters (32 bytes each) used for efficient filtering
/// - **Data**: ABI-encoded non-indexed parameters containing additional event information
///
/// # Implementation Guidelines
///
/// When implementing this trait:
/// - The first topic should typically be the event signature hash
/// - Subsequent topics should contain indexed parameters
/// - Data should contain ABI-encoded non-indexed parameters
/// - Follow Solidity event encoding conventions for compatibility
///
/// # Examples
/// ```rust,no_run
/// use hybrid_contract::log::Event;
/// use alloy_core::primitives::{Address, U256};
///
/// struct Transfer {
///     from: Address,    // Will be indexed (topic)
///     to: Address,      // Will be indexed (topic)
///     amount: U256,     // Will be in data (non-indexed)
/// }
///
/// impl Event for Transfer {
///     fn encode_log(&self) -> (Vec<u8>, Vec<[u8; 32]>) {
///         let data = self.amount.abi_encode();
///         let topics = vec![
///             keccak256("Transfer(address,address,uint256)").0,  // Event signature
///             self.from.into_word().0,                           // Indexed 'from'
///             self.to.into_word().0,                             // Indexed 'to'
///         ];
///         (data, topics)
///     }
/// }
/// ```
pub trait Event {
    /// Encodes the event into log data and topics.
    ///
    /// This method should return the event data and topics in a format suitable
    /// for blockchain logging. The encoding should follow Solidity event conventions
    /// for maximum compatibility with external tools.
    ///
    /// # Returns
    /// A tuple containing:
    /// - `Vec<u8>`: ABI-encoded event data (non-indexed parameters)
    /// - `Vec<[u8; 32]>`: Event topics (event signature + indexed parameters)
    fn encode_log(&self) -> (Vec<u8>, Vec<[u8; 32]>);
}

/// Emits an event to the blockchain log.
///
/// This is a high-level convenience function that takes any type implementing the `Event` trait,
/// encodes it into the appropriate log format, and emits it to the blockchain. This is the
/// recommended way to emit events from contracts.
///
/// # Arguments
/// * `event` - The event instance to emit, must implement the `Event` trait
///
/// # Examples
/// ```rust,no_run
/// use hybrid_contract::log::{emit, Event};
///
/// struct Approval {
///     owner: Address,
///     spender: Address,
///     amount: U256,
/// }
///
/// // Emit an approval event
/// let approval_event = Approval {
///     owner: msg_sender(),
///     spender: spender_address,
///     amount: U256::from(1000),
/// };
/// emit(approval_event);
/// ```
pub fn emit<T: Event>(event: T) {
    let (data, topics) = event.encode_log();
    emit_log(
        &data,
        &topics
            .iter()
            .map(|t| B256::from_slice(t))
            .collect::<Vec<_>>(),
    );
}

/// Emits a log entry with the specified data and topics.
///
/// This is a mid-level function that directly emits log data and topics to the blockchain.
/// It handles the formatting of topics into the expected array format and ensures
/// compliance with EVM topic limits (maximum 3 topics beyond the event signature).
///
/// # Arguments
/// * `data` - The event data as raw bytes (typically ABI-encoded)
/// * `topics` - A slice of 32-byte topics for event filtering
///
/// # Topic Limitations
/// The EVM supports a maximum of 4 topics per log entry:
/// - Topic 0: Usually the event signature hash (automatically added by most implementations)
/// - Topics 1-3: Indexed event parameters
///
/// This function enforces the 3-topic limit by truncating longer topic arrays.
///
/// # Examples
/// ```rust,no_run
/// use hybrid_contract::log::emit_log;
/// use alloy_core::primitives::B256;
///
/// let data = amount.abi_encode();
/// let topics = vec![
///     B256::from(keccak256("Transfer(address,address,uint256)")),
///     B256::from(from_address.into_word()),
///     B256::from(to_address.into_word()),
/// ];
/// emit_log(&data, &topics);
/// ```
pub fn emit_log(data: &[u8], topics: &[B256]) {
    let mut all_topics = [0u8; 96];
    let topics = &topics[..topics.len().min(3)];
    for (i, topic) in topics.iter().enumerate() {
        let start = i * 32;
        all_topics[start..start + 32].copy_from_slice(topic.as_ref());
    }

    log(
        data.as_ptr() as u64,
        data.len() as u64,
        all_topics.as_ptr() as u64,
        topics.len() as u64,
    );
}

/// Low-level function to emit a log entry via RISC-V system call.
///
/// This function performs the actual EVM LOG operation through a system call to the Hybrid VM.
/// It corresponds to the EVM LOG0, LOG1, LOG2, or LOG3 opcodes depending on the number of topics.
///
/// # Arguments
/// * `data_ptr` - Memory pointer to the event data
/// * `data_size` - Size of the event data in bytes
/// * `topics_ptr` - Memory pointer to the topics array (up to 3 topics, 32 bytes each)
/// * `topics_size` - Number of topics (0-3)
///
/// # Safety
/// This function uses inline RISC-V assembly to make a system call. The caller must ensure:
/// - `data_ptr` points to valid memory containing `data_size` bytes
/// - `topics_ptr` points to valid memory containing `topics_size * 32` bytes
/// - Memory regions remain valid during the system call
///
/// # EVM Compatibility
/// The log entry created by this function is fully compatible with EVM logs and can be:
/// - Queried using standard Ethereum JSON-RPC methods
/// - Filtered by topics using bloom filters
/// - Decoded using standard ABI decoding tools
///
/// # Examples
/// ```rust,no_run
/// // This is typically called by higher-level functions
/// let data = event_data.as_bytes();
/// let topics = [topic1, topic2, topic3]; // Each topic is 32 bytes
/// log(
///     data.as_ptr() as u64,
///     data.len() as u64,
///     topics.as_ptr() as u64,
///     3
/// );
/// ```
pub fn log(data_ptr: u64, data_size: u64, topics_ptr: u64, topics_size: u64) {
    unsafe {
        asm!(
            "ecall",
            in("a0") data_ptr,
            in("a1") data_size,
            in("a2") topics_ptr,
            in("a3") topics_size,
            in("t0") u8::from(Syscall::Log)
        );
    }
}

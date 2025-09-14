//! # Block Environment Information
//!
//! This module provides access to current block information in the Hybrid VM environment.
//! These functions correspond to EVM opcodes that provide blockchain context data.
//!
//! ## Available Information
//! - Block timestamp (TIMESTAMP opcode)
//! - Base fee per gas (BASEFEE opcode)
//! - Chain ID (CHAINID opcode)
//! - Block gas limit (GASLIMIT opcode)
//! - Block number (NUMBER opcode)
//!
//! ## Usage
//! ```rust,no_run
//! use hybrid_contract::env::block;
//!
//! let current_time = block::timestamp();
//! let current_block = block::number();
//! let chain = block::chain_id();
//! ```
//!
//! ## Safety
//! All functions use inline RISC-V assembly to make system calls to the Hybrid VM.
//! These are safe to call from any contract context.

use alloy_core::primitives::U256;
use core::arch::asm;
use hybrid_syscalls::Syscall;

/// Returns the current block timestamp in seconds since Unix epoch.
///
/// This function corresponds to the EVM TIMESTAMP opcode and provides the
/// timestamp when the current block was mined. The timestamp is returned
/// as a U256 value representing seconds since the Unix epoch (January 1, 1970).
///
/// # Returns
/// The current block timestamp as a U256
///
/// # Examples
/// ```rust,no_run
/// let now = timestamp();
/// // Use timestamp for time-based logic
/// if now > deadline {
///     // Handle expired condition
/// }
/// ```
pub fn timestamp() -> U256 {
    let first: u64;
    let second: u64;
    let third: u64;
    let fourth: u64;
    unsafe {
        asm!("ecall", lateout("a0") first, lateout("a1") second, lateout("a2") third, lateout("a3") fourth, in("t0") u8::from(Syscall::Timestamp));
    }
    U256::from_limbs([first, second, third, fourth])
}

/// Returns the current block's base fee per gas.
///
/// This function corresponds to the EVM BASEFEE opcode introduced in EIP-3198
/// as part of EIP-1559 (London hard fork). The base fee represents the minimum
/// fee per gas that must be paid for a transaction to be included in the block.
///
/// # Returns
/// The current block's base fee as a U256 (in wei per gas)
///
/// # Examples
/// ```rust,no_run
/// let current_base_fee = base_fee();
/// // Use for fee calculations or gas price logic
/// let priority_fee = gas_price() - current_base_fee;
/// ```
pub fn base_fee() -> U256 {
    let first: u64;
    let second: u64;
    let third: u64;
    let fourth: u64;
    unsafe {
        asm!("ecall", lateout("a0") first, lateout("a1") second, lateout("a2") third, lateout("a3") fourth, in("t0") u8::from(Syscall::BaseFee));
    }
    U256::from_limbs([first, second, third, fourth])
}

/// Returns the current blockchain's chain ID.
///
/// This function corresponds to the EVM CHAINID opcode introduced in EIP-1344.
/// The chain ID is used to prevent replay attacks across different blockchain
/// networks (e.g., Ethereum mainnet has chain ID 1, testnets have different IDs).
///
/// # Returns
/// The current chain ID as a u64
///
/// # Examples
/// ```rust,no_run
/// let chain = chain_id();
/// match chain {
///     1 => {/* Ethereum mainnet logic */},
///     11155111 => {/* Sepolia testnet logic */},
///     _ => {/* Other network logic */},
/// }
/// ```
pub fn chain_id() -> u64 {
    let id: u64;
    unsafe {
        asm!("ecall", lateout("a0") id, in("t0") u8::from(Syscall::ChainId));
    }
    id
}

/// Returns the current block's gas limit.
///
/// This function corresponds to the EVM GASLIMIT opcode and returns the maximum
/// amount of gas that can be consumed by all transactions in the current block.
/// This limit is set by the network and can vary between blocks.
///
/// # Returns
/// The current block's gas limit as a U256
///
/// # Examples
/// ```rust,no_run
/// let limit = gas_limit();
/// // Use for gas consumption calculations
/// if estimated_gas > limit {
///     // Transaction might not fit in a single block
/// }
/// ```
pub fn gas_limit() -> U256 {
    let first: u64;
    let second: u64;
    let third: u64;
    let fourth: u64;
    unsafe {
        asm!("ecall", lateout("a0") first, lateout("a1") second, lateout("a2") third, lateout("a3") fourth, in("t0") u8::from(Syscall::GasLimit));
    }
    U256::from_limbs([first, second, third, fourth])
}

/// Returns the current block number.
///
/// This function corresponds to the EVM NUMBER opcode and returns the sequential
/// number of the current block in the blockchain. Block numbers start from 0
/// (genesis block) and increment by 1 for each subsequent block.
///
/// # Returns
/// The current block number as a U256
///
/// # Examples
/// ```rust,no_run
/// let current_block = number();
/// let deadline_block = current_block + U256::from(100);
///
/// // Use for block-based timing or versioning
/// if current_block >= activation_block {
///     // New feature is active
/// }
/// ```
pub fn number() -> U256 {
    let first: u64;
    let second: u64;
    let third: u64;
    let fourth: u64;
    unsafe {
        asm!("ecall", lateout("a0") first, lateout("a1") second, lateout("a2") third, lateout("a3") fourth, in("t0") u8::from(Syscall::Number));
    }
    U256::from_limbs([first, second, third, fourth])
}

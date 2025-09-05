//! # Transaction Information Module
//!
//! This module provides access to current transaction information in the Hybrid VM environment.
//! These functions correspond to EVM opcodes that provide transaction context data, allowing
//! contracts to access details about the current transaction being executed.
//!
//! ## Available Information
//! - Gas price (GASPRICE opcode) - The gas price paid for the current transaction
//! - Transaction origin (ORIGIN opcode) - The original sender of the transaction chain
//!
//! ## Gas Price vs Base Fee
//! In EIP-1559 networks, the gas price represents the effective gas price paid by the transaction,
//! which includes both the base fee and the priority fee (tip). This is different from the
//! base fee, which can be accessed through the block environment module.
//!
//! ## Transaction Origin vs Message Sender
//! - **Origin** (`tx.origin`): The original external account that initiated the transaction
//! - **Sender** (`msg.sender`): The immediate caller of the current contract (can be a contract)
//!
//! ## Usage
//! ```rust,no_run
//! use hybrid_contract::tx;
//!
//! // Get transaction details
//! let price = tx::gas_price();
//! let original_sender = tx::origin();
//!
//! // Use for access control
//! if origin() != authorized_user {
//!     revert(); // Only allow direct calls from authorized user
//! }
//! ```
//!
//! ## Security Considerations
//! Using `tx.origin` for authorization is generally discouraged as it can be vulnerable
//! to certain attack patterns. Prefer using `msg.sender` for most authorization logic.

use alloy_core::primitives::{Address, U256};
use core::arch::asm;
use hybrid_syscalls::Syscall;

/// Returns the gas price of the current transaction.
/// Returns the gas price of the current transaction.
///
/// This function corresponds to the EVM GASPRICE opcode and returns the gas price
/// that was paid for the current transaction. In EIP-1559 networks, this represents
/// the effective gas price, which includes both the base fee and the priority fee.
///
/// # Returns
/// The gas price as a U256 value in wei per gas unit
///
/// # Examples
/// ```rust,no_run
/// let current_gas_price = gas_price();
/// let base_fee = block::base_fee();
/// let priority_fee = current_gas_price - base_fee;
///
/// // Use for gas optimization logic
/// if current_gas_price > high_gas_threshold {
///     // Use gas-efficient code path
/// }
/// ```
///
/// # EIP-1559 Context
/// In post-London hard fork networks:
/// - `gas_price() = base_fee + priority_fee`
/// - Base fee is burned, priority fee goes to miners
/// - This provides the total amount paid per gas unit
pub fn gas_price() -> U256 {
    let first: u64;
    let second: u64;
    let third: u64;
    let fourth: u64;
    unsafe {
        asm!("ecall", lateout("a0") first, lateout("a1") second, lateout("a2") third, lateout("a3") fourth, in("t0") u8::from(Syscall::GasPrice))
    }
    U256::from_limbs([first, second, third, fourth])
}

/// Returns the original sender of the entire transaction chain.
///
/// This function corresponds to the EVM ORIGIN opcode and returns the address of the
/// externally-owned account (EOA) that originally initiated the current transaction.
/// Unlike `msg.sender`, this value remains constant throughout the entire call chain,
/// even when contracts call other contracts.
///
/// # Returns
/// The address of the original transaction sender as an `Address`
///
/// # Security Warning
/// Using `tx.origin` for authorization is generally considered unsafe because it can
/// be exploited through contract-to-contract calls. An attacker could create a
/// malicious contract that calls your contract, and your contract would see the
/// victim's address as `tx.origin`.
///
/// # Examples
/// ```rust,no_run
/// let transaction_origin = origin();
/// let immediate_caller = msg_sender();
///
/// // These might be different in a contract-to-contract call
/// if transaction_origin != immediate_caller {
///     // This call came through another contract
/// }
///
/// // Discouraged: Using origin for authorization
/// // if origin() != owner { revert(); }  // Vulnerable to attacks
///
/// // Preferred: Using msg.sender for authorization
/// // if msg_sender() != owner { revert(); }  // Safer approach
/// ```
///
/// # Use Cases
/// Safe uses of `tx.origin` include:
/// - Logging and analytics (tracking original transaction initiators)
/// - Gas refund mechanisms (ensuring refunds go to transaction initiator)
/// - Rate limiting per user (tracking unique transaction originators)
///
/// # Attack Vector Example
/// ```solidity
/// // Vulnerable contract
/// contract Vulnerable {
///     function withdraw() external {
///         require(tx.origin == owner);  // BAD: vulnerable to attack
///         // ...
///     }
/// }
///
/// // Attacker contract
/// contract Attacker {
///     function attack(Vulnerable target) external {
///         target.withdraw();  // tx.origin is still the victim's address
///     }
/// }
/// ```
pub fn origin() -> Address {
    let first: u64;
    let second: u64;
    let third: u64;
    unsafe {
        asm!("ecall", lateout("a0") first, lateout("a1") second, lateout("a2") third, in("t0") u8::from(Syscall::Origin));
    }
    let mut bytes = [0u8; 20];
    bytes[0..8].copy_from_slice(&first.to_be_bytes());
    bytes[8..16].copy_from_slice(&second.to_be_bytes());
    bytes[16..20].copy_from_slice(&third.to_be_bytes()[..4]);
    Address::from_slice(&bytes)
}

//! # Environment Module
//!
//! This module provides access to blockchain environment information in the Hybrid VM.
//! It exposes various context data that smart contracts can access during execution,
//! similar to EVM opcodes that provide blockchain state information.
//!
//! ## Submodules
//!
//! ### `block`
//! Provides access to current block information including:
//! - Block timestamp
//! - Base fee per gas (EIP-1559)
//! - Chain ID
//! - Block gas limit
//! - Block number
//!
//! ### `msg`
//! Will provide access to message/transaction context information including:
//! - Message sender
//! - Message value
//! - Message data
//! - Gas information
//!
//! ## Usage
//! ```rust,no_run
//! use hybrid_contract::env::{block, msg};
//!
//! // Access block information
//! let current_time = block::timestamp();
//! let current_block = block::number();
//!
//! // Access message information (when implemented)
//! // let sender = msg::sender();
//! // let value = msg::value();
//! ```
//!
//! ## Safety
//! All environment functions use system calls to retrieve information from the
//! Hybrid VM. These calls are safe and do not modify state, only read current
//! blockchain context.

pub mod block;
pub mod msg;

//! # Mini EVM Interpreter
//!
//! A lightweight, hybrid EVM (Ethereum Virtual Machine) interpreter designed to run in a
//! no-std environment with RISC-V architecture support. This interpreter provides a minimal
//! but functional implementation of EVM bytecode execution.
//!
//! ## Overview
//!
//! This mini-EVM interpreter is built as a hybrid contract that can execute Ethereum bytecode
//! in constrained environments. It supports a subset of EVM opcodes through a custom instruction
//! table and operates with serialized input/output for state management.
//!
//! ## Architecture
//!
//! The interpreter consists of:
//! - **Instruction Table**: Custom opcode implementations optimized for the hybrid environment
//! - **Utilities**: Memory management, serialization, and I/O operations
//! - **Context Management**: Block, transaction, and configuration handling
//!
//! ## Memory Model
//!
//! The interpreter operates in a specific memory layout:
//! - Input data is read from a designated memory address
//! - Output is written back to the same location after execution
//! - Debug information can be written to a separate debug region
//!
//! ## Usage
//!
//! The interpreter expects serialized input containing:
//! - Interpreter state
//! - Block environment
//! - Transaction environment
//!
//! After execution, it outputs:
//! - Updated interpreter state
//! - Block environment
//! - Transaction environment
//! - Execution result (InterpreterAction)
//!
//! ## Safety
//!
//! This code uses `no_std` and `no_main` attributes and performs unsafe memory operations.
//! It's designed to run in a controlled hybrid contract environment where memory layout
//! and execution context are well-defined.

#![no_std]
#![no_main]

extern crate alloc;

mod instruction_table;
mod utils;

use ext_revm::{
    context::{CfgEnv, JournalTr},
    database::EmptyDB,
    Context, Journal,
};

use crate::{
    instruction_table::mini_instruction_table,
    utils::{read_input, write_output},
};

/// The chain ID used for this EVM interpreter instance.
///
/// Set to 1, which corresponds to Ethereum mainnet. This affects
/// the behavior of the CHAINID opcode and chain-specific validations.
const CHAIN_ID: u64 = 1;

/// Main entry point for the mini-EVM interpreter.
///
/// This function serves as the primary execution entry point for the hybrid contract.
/// It follows the complete EVM execution cycle:
///
/// 1. **Input Reading**: Deserializes interpreter state, block environment, and
///    transaction environment from memory
/// 2. **Context Setup**: Configures the execution context with chain ID and
///    creates a journal for state tracking
/// 3. **Execution**: Runs the interpreter with the custom instruction table
/// 4. **Output Writing**: Serializes and writes the execution results back to memory
///
/// ## Execution Flow
///
/// ```text
/// Memory Input → Deserialize → Setup Context → Execute Bytecode → Serialize → Memory Output
/// ```
///
/// ## Context Configuration
///
/// The execution context includes:
/// - **Block Environment**: Current block information (timestamp, number, etc.)
/// - **Transaction Environment**: Transaction-specific data (gas limit, value, etc.)
/// - **Configuration**: Chain-specific settings (chain ID, EVM version, etc.)
/// - **Journal**: State tracking for reversible operations
///
/// ## Error Handling
///
/// The function panics on input deserialization errors. In a production environment,
/// proper error handling should be implemented based on the specific hybrid contract
/// requirements.
///
/// ## Memory Safety
///
/// This function performs several unsafe operations through utility functions:
/// - Raw memory access for input/output operations
/// - Inline assembly for register manipulation
/// - Direct memory writes without bounds checking
///
/// ## Returns
///
/// This function never returns normally (marked with `!`). After writing the output,
/// it reaches an `unreachable!()` statement, indicating the end of execution in the
/// hybrid contract context.
#[hybrid_contract::entry]
fn main() -> ! {
    // Read and deserialize input data from memory
    // This includes the interpreter state, block environment, and transaction environment
    let input = read_input().unwrap();

    let mut interpreter = input.0; // EVM interpreter instance
    let block = input.1; // Block environment (block number, timestamp, etc.)
    let tx = input.2; // Transaction environment (gas limit, value, etc.)

    // Configure the EVM environment with chain-specific settings
    let mut cfg = CfgEnv::new();
    cfg = cfg.with_chain_id(CHAIN_ID);

    // Create the execution context with all necessary environments
    // The context tracks the complete state during EVM execution
    let mut context: Context = Context {
        block: block.clone(),                              // Block-level information
        cfg,                                               // Configuration settings
        chain: (), // Chain-specific data (empty for this implementation)
        journaled_state: Journal::new(EmptyDB::default()), // State journal with empty database
        error: Ok(()), // Error tracking (starts as Ok)
        tx: tx.clone(), // Transaction-level information
    };

    // Execute the EVM bytecode using the custom instruction table
    // This is the core execution step where bytecode is interpreted and executed
    let out = interpreter.run_plain(&mini_instruction_table(), &mut context);

    // Serialize and write the execution results back to memory
    // This includes the updated interpreter state and the execution result
    write_output(&interpreter, &block, &tx, &out);

    // Execution is complete - this point should never be reached in normal operation
    // The hybrid contract environment handles the termination after output is written
    unreachable!()
}

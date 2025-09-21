#![allow(dead_code)]
//! Utilities for the mini-EVM interpreter.
//!
//! This module provides essential utility functions for the hybrid mini-EVM interpreter,
//! including input/output serialization, memory operations, and debugging utilities.
//!
//! # Memory Layout
//!
//! The interpreter operates in a specific memory layout where:
//! - Input data starts at `CALLDATA_ADDRESS`
//! - Debug output is written to `CALLDATA_ADDRESS + 1GB - 2000`
//! - Data format includes length headers followed by serialized data
//!
//! # Serialization Format
//!
//! Input data format:
//! ```text
//! [interpreter_len: u64][block_len: u64][tx_len: u64][interpreter_data][block_data][tx_data]
//! ```
//!
//! Output data format:
//! ```text
//! [interpreter_len: u64][block_len: u64][tx_len: u64][action_len: u64]
//! [interpreter_data][block_data][tx_data][action_data]
//! ```

use alloc::string::String;
use alloc::vec::Vec;
use core::arch::asm;
use ext_revm::{
    context::{BlockEnv, TxEnv},
    interpreter::{Interpreter, InterpreterAction},
    primitives::Address,
};
use hybrid_contract::{slice_from_raw_parts, slice_from_raw_parts_mut, CALLDATA_ADDRESS};

/// Reads and deserializes input data from memory.
///
/// This function reads serialized interpreter state, block environment, and transaction
/// environment from the designated memory location and deserializes them into their
/// respective types.
///
/// # Returns
///
/// Returns a `Result` containing a tuple of:
/// - `Interpreter`: The EVM interpreter instance
/// - `BlockEnv`: The block environment configuration
/// - `TxEnv`: The transaction environment configuration
///
/// # Errors
///
/// Returns a `String` error if deserialization fails or if the input data is malformed.
///
/// # Safety
///
/// This function performs unsafe memory operations internally through `copy_from_mem()`.
pub fn read_input() -> Result<(Interpreter, BlockEnv, TxEnv), String> {
    let input = copy_from_mem();
    let interpreter_n_context = deserialize_input(input);

    Ok(interpreter_n_context)
}

/// Writes a static "Hello, world!" debug message to the debug memory location.
///
/// This function writes a hardcoded debug message to a specific memory address
/// used for debugging purposes. The message is written to a fixed offset from
/// the calldata address.
///
/// # Safety
///
/// This function is unsafe because it:
/// - Performs raw memory operations
/// - Assumes the target memory location is valid and writable
/// - Does not perform bounds checking
///
/// The caller must ensure that the memory region at the debug address is properly
/// allocated and accessible.
pub unsafe fn debug_println() {
    let address = CALLDATA_ADDRESS + (1024 * 1024 * 1024) - 2000;
    let data = b"Hello, world!";
    let dest = slice_from_raw_parts_mut(address, data.len());
    dest.copy_from_slice(data);
}

/// Writes arbitrary debug data to the debug memory location.
///
/// This function writes the provided byte slice to the designated debug memory
/// location for debugging purposes.
///
/// # Arguments
///
/// * `data` - A byte slice containing the data to write to debug memory
///
/// # Safety
///
/// This function is unsafe because it:
/// - Performs raw memory operations without bounds checking
/// - Assumes the target memory location is valid and writable
/// - May overwrite existing memory content
///
/// The caller must ensure that:
/// - The debug memory region can accommodate the data length
/// - The memory region is properly allocated and accessible
/// - No other code is concurrently accessing the same memory region
pub unsafe fn debug_println_dyn_data(data: &[u8]) {
    let address = CALLDATA_ADDRESS + (1024 * 1024 * 1024) - 2000;
    let dest = slice_from_raw_parts_mut(address, data.len());
    dest.copy_from_slice(data);
}

/// Serializes and writes the interpreter output to memory.
///
/// This function takes the final state of the interpreter execution and serializes
/// all components (interpreter state, block environment, transaction environment,
/// and interpreter action) into a single byte buffer, then writes it to the
/// designated output memory location.
///
/// The function also stores the total length of the serialized data in register `t6`
/// using inline assembly.
///
/// # Arguments
///
/// * `interpreter` - Reference to the interpreter instance after execution
/// * `block` - Reference to the block environment used during execution
/// * `tx` - Reference to the transaction environment used during execution
/// * `out` - Reference to the interpreter action result from execution
///
/// # Safety
///
/// This function performs unsafe operations:
/// - Uses inline assembly to set register `t6`
/// - Calls `write_to_memory` which performs unsafe memory operations
///
/// # Memory Layout
///
/// The output is written with the following structure:
/// - 4 × u64 length headers (interpreter, block, tx, action)
/// - Serialized interpreter data
/// - Serialized block data
/// - Serialized transaction data
/// - Serialized action data
pub fn write_output(
    interpreter: &Interpreter,
    block: &BlockEnv,
    tx: &TxEnv,
    out: &InterpreterAction,
) {
    let serialized = serialize_output(interpreter, block, tx, out);
    let length = serialized.len() as u64;

    unsafe {
        asm!(
            "mv t6, {val}",
            val = in(reg) length,
        );
    }
    unsafe {
        write_to_memory(CALLDATA_ADDRESS, &serialized);
    }
}

/// Copies input data from the designated memory location.
///
/// This function reads the length of the input data from the first 8 bytes at
/// `CALLDATA_ADDRESS`, then returns a slice referencing the actual data that
/// follows the length header.
///
/// # Returns
///
/// Returns a static byte slice containing the input data. The slice is valid
/// for the lifetime of the program since it references memory that persists
/// throughout execution.
///
/// # Memory Layout
///
/// The function expects the following memory layout at `CALLDATA_ADDRESS`:
/// - Bytes 0-7: Length of data as little-endian u64
/// - Bytes 8+: Actual data content
///
/// # Safety
///
/// This function performs unsafe memory operations by directly accessing
/// memory at `CALLDATA_ADDRESS`. The caller must ensure that:
/// - The memory location contains valid length data
/// - The memory region is properly initialized
/// - The length value is accurate and the corresponding data exists
pub fn copy_from_mem() -> &'static [u8] {
    let length = unsafe { slice_from_raw_parts(CALLDATA_ADDRESS, 8) };
    let length = u64::from_le_bytes([
        length[0], length[1], length[2], length[3], length[4], length[5], length[6], length[7],
    ]) as usize;
    unsafe { slice_from_raw_parts(CALLDATA_ADDRESS + 8, length) }
}

/// Writes data to a specific memory address.
///
/// This is a low-level utility function that copies the provided data directly
/// to the specified memory address.
///
/// # Arguments
///
/// * `address` - The target memory address where data should be written
/// * `data` - The byte slice containing data to write
///
/// # Safety
///
/// This function is unsafe because it:
/// - Performs raw memory operations without bounds checking
/// - Assumes the target memory address is valid and writable
/// - May overwrite existing memory content
/// - Does not verify that the memory region can accommodate the data length
///
/// The caller must ensure that:
/// - The target address is valid and properly allocated
/// - The memory region from `address` to `address + data.len()` is accessible
/// - No other code is concurrently accessing the same memory region
/// - The write operation will not corrupt critical system data
pub unsafe fn write_to_memory(address: usize, data: &[u8]) {
    let dest = slice_from_raw_parts_mut(address, data.len());
    dest.copy_from_slice(data);
}

/// Deserializes input data into interpreter components.
///
/// This function takes a byte slice containing serialized interpreter data and
/// deserializes it into the three main components: Interpreter, BlockEnv, and TxEnv.
///
/// # Arguments
///
/// * `data` - Byte slice containing the serialized input data
///
/// # Returns
///
/// Returns a tuple containing:
/// - `Interpreter`: Deserialized interpreter instance
/// - `BlockEnv`: Deserialized block environment
/// - `TxEnv`: Deserialized transaction environment
///
/// # Data Format
///
/// The input data must follow this exact format:
/// ```text
/// [si_len: u64][sb_len: u64][st_len: u64][interpreter_data][block_data][tx_data]
/// ```
///
/// Where:
/// - `si_len`: Length of serialized interpreter data
/// - `sb_len`: Length of serialized block environment data
/// - `st_len`: Length of serialized transaction environment data
///
/// # Panics
///
/// This function panics if:
/// - Input data is shorter than 24 bytes (minimum for 3 length headers)
/// - Total data length doesn't match the sum of individual component lengths plus headers
/// - Bincode deserialization fails for any component
///
/// # Examples
///
/// ```rust,no_run
/// let serialized_data = get_input_data();
/// let (interpreter, block_env, tx_env) = deserialize_input(&serialized_data);
/// ```
pub fn deserialize_input(data: &[u8]) -> (Interpreter, BlockEnv, TxEnv) {
    // Check minimum length for headers (16 bytes for two u64 lengths)
    if data.len() < 24 {
        panic!("Data too short for headers");
    }

    // Read the lengths from the first 16 bytes
    let si_len = u64::from_le_bytes(data[0..8].try_into().unwrap()) as usize;
    let sb_len = u64::from_le_bytes(data[8..16].try_into().unwrap()) as usize;
    let st_len = u64::from_le_bytes(data[16..24].try_into().unwrap()) as usize;

    // Check total length
    let expected_len = si_len + sb_len + st_len + 24;
    if data.len() != expected_len {
        panic!(
            "Data length mismatch: expected {}, got {}",
            expected_len,
            data.len()
        );
    }

    // Extract the interpreter bytes
    let interpreter_bytes = &data[24..24 + si_len];
    let interpreter: Interpreter =
        bincode::serde::decode_from_slice(interpreter_bytes, bincode::config::legacy())
            .unwrap()
            .0;

    // Extract the block bytes
    let block_bytes = &data[24 + si_len..24 + si_len + sb_len];
    let block: BlockEnv = bincode::serde::decode_from_slice(block_bytes, bincode::config::legacy())
        .unwrap()
        .0;

    // Extract the transaction bytes
    let tx_bytes = &data[24 + si_len + sb_len..24 + si_len + sb_len + st_len];
    let tx: TxEnv = bincode::serde::decode_from_slice(tx_bytes, bincode::config::legacy())
        .unwrap()
        .0;

    (interpreter, block, tx)
}

/// Serializes interpreter output components into a single byte vector.
///
/// This function takes the final state of all interpreter components and serializes
/// them into a single byte vector with a specific format that includes length headers
/// for each component.
///
/// # Arguments
///
/// * `interpreter` - Reference to the interpreter instance after execution
/// * `block` - Reference to the block environment used during execution
/// * `tx` - Reference to the transaction environment used during execution
/// * `out` - Reference to the interpreter action result from execution
///
/// # Returns
///
/// Returns a `Vec<u8>` containing the serialized output data in the following format:
/// ```text
/// [si_len: u64][sb_len: u64][st_len: u64][so_len: u64]
/// [interpreter_data][block_data][tx_data][action_data]
/// ```
///
/// Where:
/// - `si_len`: Length of serialized interpreter data
/// - `sb_len`: Length of serialized block environment data
/// - `st_len`: Length of serialized transaction environment data
/// - `so_len`: Length of serialized interpreter action data
///
/// # Serialization
///
/// All components are serialized using bincode with legacy configuration to ensure
/// compatibility with the expected data format.
///
/// # Examples
///
/// ```rust,no_run
/// let output_data = serialize_output(&interpreter, &block_env, &tx_env, &action);
/// write_to_memory(OUTPUT_ADDRESS, &output_data);
/// ```
pub fn serialize_output(
    interpreter: &Interpreter,
    block: &BlockEnv,
    tx: &TxEnv,
    out: &InterpreterAction,
) -> Vec<u8> {
    let s_interpreter =
        bincode::serde::encode_to_vec(interpreter, bincode::config::legacy()).unwrap();
    let s_block = bincode::serde::encode_to_vec(block, bincode::config::legacy()).unwrap();
    let s_tx = bincode::serde::encode_to_vec(tx, bincode::config::legacy()).unwrap();
    let s_out = bincode::serde::encode_to_vec(out, bincode::config::legacy()).unwrap();

    let si_len = s_interpreter.len();
    let sb_len = s_block.len();
    let st_len = s_tx.len();
    let so_len = s_out.len();

    let mut serialized = Vec::with_capacity(si_len + sb_len + st_len + so_len + 32);

    serialized.extend((si_len as u64).to_le_bytes());
    serialized.extend((sb_len as u64).to_le_bytes());
    serialized.extend((st_len as u64).to_le_bytes());
    serialized.extend((so_len as u64).to_le_bytes());

    serialized.extend(s_interpreter);
    serialized.extend(s_block);
    serialized.extend(s_tx);
    serialized.extend(s_out);

    serialized
}

pub fn __3u64_to_address(limb_one: u64, limb_two: u64, limb_three: u64) -> Address {
    let mut bytes = [0u8; 20];
    bytes[0..8].copy_from_slice(&limb_one.to_be_bytes());
    bytes[8..16].copy_from_slice(&limb_two.to_be_bytes());
    bytes[16..20].copy_from_slice(&limb_three.to_be_bytes()[4..]);
    Address::from_slice(&bytes)
}

pub fn __address_to_3u64(address: Address) -> (u64, u64, u64) {
    let bytes = address.0;
    let limb_one = u64::from_be_bytes(bytes[0..8].try_into().unwrap());
    let limb_two = u64::from_be_bytes(bytes[8..16].try_into().unwrap());
    let mut buf = [0u8; 8];
    buf[4..].copy_from_slice(&bytes[16..20]);
    let limb_three = u64::from_be_bytes(buf);
    (limb_one, limb_two, limb_three)
}

#![allow(dead_code)]
//! Utilities for the mini-evm interpreter.
use alloc::string::String;
use alloc::vec::Vec;
use core::arch::asm;
use hybrid_contract::{slice_from_raw_parts, slice_from_raw_parts_mut, CALLDATA_ADDRESS};
use ext_revm::{
    context::{BlockEnv, TxEnv},
    interpreter::{Interpreter, InterpreterAction},
};


pub fn read_input() -> Result<(Interpreter, BlockEnv, TxEnv), String> {
    let input = copy_from_mem();
    let interpreter_n_context = deserialize_input(input);

    Ok(interpreter_n_context)
}


pub unsafe fn debug_println() {
    let address = CALLDATA_ADDRESS + (1024 * 1024 * 1024) - 2000;
    let data = b"Hello, world!";
    let dest = slice_from_raw_parts_mut(address, data.len());
    dest.copy_from_slice(data);
}

pub unsafe fn debug_println_dyn_data(data: &[u8]) {
    let address = CALLDATA_ADDRESS + (1024 * 1024 * 1024) - 2000;
    let dest = slice_from_raw_parts_mut(address, data.len());
    dest.copy_from_slice(data);
}

pub fn write_output(interpreter: &Interpreter, block: &BlockEnv, tx: &TxEnv, out: &InterpreterAction) {
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

pub fn copy_from_mem() -> &'static [u8] {
    let length = unsafe { slice_from_raw_parts(CALLDATA_ADDRESS, 8) };
    let length = u64::from_le_bytes([
        length[0], length[1], length[2], length[3], length[4], length[5], length[6], length[7],
    ]) as usize;
    unsafe { slice_from_raw_parts(CALLDATA_ADDRESS + 8, length) }
}

pub unsafe fn write_to_memory(address: usize, data: &[u8]) {
    let dest = slice_from_raw_parts_mut(address, data.len());
    dest.copy_from_slice(data);
}



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

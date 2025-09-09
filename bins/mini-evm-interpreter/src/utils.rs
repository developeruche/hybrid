//! Utilities for the mini-evm interpreter.
use alloc::string::String;
use hybrid_contract::{slice_from_raw_parts, slice_from_raw_parts_mut, CALLDATA_ADDRESS};

use crate::{Input, Output};

pub fn read_input() -> Result<Input, String> {
    let input = copy_from_mem();
    let deserialized: Input = serde_json::from_slice(&input).unwrap();
    Ok(deserialized)
}

pub fn write_output(output: &Output) {
    let serialized = serde_json::to_vec(output).unwrap();
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

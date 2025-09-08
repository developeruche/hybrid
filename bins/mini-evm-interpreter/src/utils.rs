//! Utilities for the mini-evm interpreter.
use alloc::string::String;
use hybrid_contract::{slice_from_raw_parts, CALLDATA_ADDRESS};

use crate::{Input, Output};

// pub fn deserialize_input() -> Result<Input, String> {
//     let input = copy_from_mem();
//     let deserialized: Input = bincode::deserialize(&input).unwrap();
//     Ok(deserialized)
// }

// pub fn serialize_output(output: &Output) {
//     let serialized = bincode::serialize(output).unwrap();
//     // write this serialized output to an address in memory
// }

// pub fn copy_from_mem() -> &'static [u8] {
//     let length = unsafe { slice_from_raw_parts(CALLDATA_ADDRESS, 8) };
//     let length = u64::from_le_bytes([
//         length[0], length[1], length[2], length[3], length[4], length[5], length[6], length[7],
//     ]) as usize;
//     unsafe { slice_from_raw_parts(CALLDATA_ADDRESS + 8, length) }
// }

//! Utilities for the mini-evm interpreter.
use alloc::string::String;

use crate::Input;

pub fn deserialize_input(input: &[u8]) -> Result<Input, String> {
    // read this raw input from an address in memory
    let deserialized: Input = bincode::deserialize(&input).unwrap();
    Ok(deserialized)
}


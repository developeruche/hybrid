#![no_std]

use alloc::vec::Vec;
use hybrid_vm::eth_hybrid::EthEvmContext;
use revm::{
    handler::instructions::EthInstructions,
    interpreter::{instruction_table, interpreter::EthInterpreter, Interpreter},
    Context,
};
use serde::{Deserialize, Serialize};

use crate::utils::deserialize_input;
extern crate alloc;

mod instruction_table;
mod utils;

#[derive(Serialize, Deserialize)]
pub struct Input {
    context: Context,
    interpreter: Interpreter,
}


fn main() {
    // load bincode "context"
    // load bincode "instructions"
    // load bincode interpreter
    // step down to mini evm interpreter
    // execute interpreter
    // bincode `InterpretAction`
    let input = deserialize_input(&Vec::new()).unwrap();

    let mut context = input.context;
    let mut interpreter = input.interpreter;

    let out = interpreter.run_plain(&instruction_table(), &mut context);
}

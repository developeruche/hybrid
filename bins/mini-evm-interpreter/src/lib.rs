#![no_std]
#![no_main]

use alloy_core::primitives::{Address, U256};
use core::default::Default;
use hybrid_contract::hstd::*;
use hybrid_derive::{contract, payable, storage, Error, Event};
extern crate alloc;


// #![no_std]

// use alloc::vec::Vec;
// use hybrid_vm::eth_hybrid::EthEvmContext;
// use revm::{
//     handler::instructions::EthInstructions,
//     interpreter::{instruction_table, interpreter::EthInterpreter, Interpreter, InterpreterAction},
//     Context,
// };
// use serde::{Deserialize, Serialize};

// // use crate::utils::deserialize_input;
// extern crate alloc;

// mod instruction_table;
// mod utils;

// #[derive(Serialize, Deserialize)]
// pub struct Input {
//     context: Context,
//     interpreter: Interpreter,
// }


// #[derive(Serialize, Deserialize)]
// pub struct Output {
//     context: Context,
//     interpreter: Interpreter,
//     out: InterpreterAction
// }



// fn main() {
//     let input = deserialize_input(&Vec::new()).unwrap();

//     let mut context = input.context;
//     let mut interpreter = input.interpreter;

//     let out = interpreter.run_plain(&instruction_table(), &mut context);
    
//     let output = Output {
//         context,
//         interpreter,
//         out,
//     };

    
// }



#[hybrid_contract::entry]
fn main() -> ! {
    // let mut contract = #struct_name::default();
    // contract.call();
    hybrid_contract::return_riscv(0, 0)
}
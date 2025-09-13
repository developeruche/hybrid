#![no_std]
#![no_main]

extern crate alloc;

mod instruction_table;
mod utils;

use alloc::vec::Vec;
use ext_revm::{
    context::{BlockEnv, CfgEnv, JournalTr, TxEnv}, context_interface::block::BlobExcessGasAndPrice, database::EmptyDB, interpreter::{Interpreter, InterpreterAction}, Context, Journal
};
use ext_revm::primitives::{Address, B256, U256};
use ext_revm::{context::transaction::{AccessList, SignedAuthorization}, primitives::{Bytes, TxKind}};
use serde::{Deserialize, Serialize};

use crate::{
    instruction_table::mini_instruction_table,
    utils::{debug_println, debug_println_dyn_data, read_input, write_output, MiniContext},
};


pub struct Input {
    context: Context,
    interpreter: Interpreter,
}

pub struct Output {
    context: Context,
    interpreter: Interpreter,
    out: InterpreterAction,
}

#[hybrid_contract::entry]
fn main() -> ! {
    // let input = read_input().unwrap();
    let raw_interpreter: &[u8] = &[0, 0, 0, 0, 25, 1, 0, 0, 0, 0, 0, 0, 96, 128, 128, 96, 64, 82, 52, 96, 19, 87, 96, 223, 144, 129, 96, 25, 130, 57, 243, 91, 96, 0, 128, 253, 254, 96, 128, 128, 96, 64, 82, 96, 4, 54, 16, 21, 96, 18, 87, 96, 0, 128, 253, 91, 96, 0, 53, 96, 224, 28, 144, 129, 99, 63, 181, 193, 203, 20, 96, 146, 87, 129, 99, 131, 129, 245, 138, 20, 96, 121, 87, 80, 99, 208, 157, 224, 138, 20, 96, 60, 87, 96, 0, 128, 253, 91, 52, 96, 116, 87, 96, 0, 54, 96, 3, 25, 1, 18, 96, 116, 87, 96, 0, 84, 96, 0, 25, 129, 20, 96, 94, 87, 96, 1, 1, 96, 0, 85, 0, 91, 99, 78, 72, 123, 113, 96, 224, 27, 96, 0, 82, 96, 17, 96, 4, 82, 96, 36, 96, 0, 253, 91, 96, 0, 128, 253, 91, 52, 96, 116, 87, 96, 0, 54, 96, 3, 25, 1, 18, 96, 116, 87, 96, 32, 144, 96, 0, 84, 129, 82, 243, 91, 52, 96, 116, 87, 96, 32, 54, 96, 3, 25, 1, 18, 96, 116, 87, 96, 4, 53, 96, 0, 85, 0, 254, 162, 100, 105, 112, 102, 115, 88, 34, 18, 32, 233, 120, 39, 8, 131, 183, 186, 237, 16, 129, 12, 64, 121, 201, 65, 81, 46, 147, 167, 186, 28, 209, 16, 140, 120, 29, 75, 199, 56, 217, 9, 5, 100, 115, 111, 108, 99, 67, 0, 8, 26, 0, 51, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 248, 0, 0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 98, 105, 116, 118, 101, 99, 58, 58, 111, 114, 100, 101, 114, 58, 58, 76, 115, 98, 48, 8, 0, 25, 1, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 8, 0, 0, 0, 0, 32, 0, 0, 0, 128, 0, 0, 32, 4, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 95, 189, 178, 49, 86, 120, 175, 236, 179, 103, 240, 50, 217, 63, 100, 47, 100, 24, 10, 163, 20, 0, 0, 0, 0, 0, 0, 0, 243, 159, 214, 229, 26, 173, 136, 246, 244, 206, 106, 184, 130, 114, 121, 207, 255, 185, 34, 102, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 64, 34, 255, 255, 255, 135, 243, 0, 64, 34, 255, 255, 255, 135, 243, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 18, 0, 0, 0];
    
    let con: Interpreter = bincode::serde::decode_from_slice(&raw_interpreter, bincode::config::legacy()).unwrap().0;
    
    unsafe { debug_println(); };

    

    // let mut context = input.1;
    // let mut interpreter = input.0;

    // let out = interpreter.run_plain(&mini_instruction_table(), &mut context);

    // let output = Output {
    //     context,
    //     interpreter,
    //     out,
    // };

    // write_output(&output);

    unreachable!()
}

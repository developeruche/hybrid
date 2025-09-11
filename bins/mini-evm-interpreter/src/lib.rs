#![no_std]
#![no_main]

extern crate alloc;

mod instruction_table;
mod utils;

use revm::{
    interpreter::{Interpreter, InterpreterAction},
    Context,
};
use serde::{Deserialize, Serialize};

use crate::{
    instruction_table::mini_instruction_table,
    utils::{read_input, write_output},
};

#[derive(Serialize, Deserialize)]
pub struct Input {
    context: Context,
    interpreter: Interpreter,
}

#[derive(Serialize, Deserialize)]
pub struct Output {
    context: Context,
    interpreter: Interpreter,
    out: InterpreterAction,
}

#[hybrid_contract::entry]
fn main() -> ! {
    let input = read_input().unwrap();

    let mut context = input.1;
    let mut interpreter = input.0;

    let out = interpreter.run_plain(&mini_instruction_table(), &mut context);

    let output = Output {
        context,
        interpreter,
        out,
    };

    write_output(&output);

    unreachable!()
}

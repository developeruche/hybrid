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

const CHAIN_ID: u64 = 1;

#[hybrid_contract::entry]
fn main() -> ! {
    let input = read_input().unwrap();

    let mut interpreter = input.0;
    let block = input.1;
    let tx = input.2;

    let mut cfg = CfgEnv::new();
    cfg = cfg.with_chain_id(CHAIN_ID);

    let mut context: Context = Context {
        block: block.clone(),
        cfg,
        chain: (),
        journaled_state: Journal::new(EmptyDB::default()),
        error: Ok(()),
        tx: tx.clone(),
    };

    let out = interpreter.run_plain(&mini_instruction_table(), &mut context);

    write_output(&interpreter, &block, &tx, &out);

    unreachable!()
}

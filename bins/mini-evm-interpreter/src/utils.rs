//! Utilities for the mini-evm interpreter.
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use hybrid_contract::{slice_from_raw_parts, slice_from_raw_parts_mut, CALLDATA_ADDRESS};
use revm::{
    context::{BlockEnv, CfgEnv, JournalTr, TxEnv},
    context_interface::context::ContextError,
    database::EmptyDB,
    interpreter::{Interpreter, InterpreterAction},
    Context, Database, Journal,
};
use serde::{Deserialize, Serialize};

use crate::{Input, Output};

pub fn read_input() -> Result<(Interpreter, Context), String> {
    let input = copy_from_mem();
    let interpreter_n_context = deserialize_input(input);

    Ok(interpreter_n_context)
}

pub fn write_output(output: &Output) {
    let s_interpreter = serde_json::to_vec(&output.interpreter).unwrap();
    let serialized = serialize_output(&s_interpreter, &output.context, &output.out);
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

#[derive(Serialize, Deserialize)]
pub struct MiniContext<
    BLOCK = BlockEnv,
    TX = TxEnv,
    CFG = CfgEnv,
    DB: Database = EmptyDB,
    JOURNAL: JournalTr<Database = DB> = Journal<DB>,
    CHAIN = (),
> {
    /// Block information.
    pub block: BLOCK,
    /// Transaction information.
    pub tx: TX,
    /// Configurations.
    pub cfg: CFG,
    /// EVM State with journaling support and database.
    pub journaled_state: JOURNAL,
    /// Inner context.
    pub chain: CHAIN,
    #[serde(skip, default = "default_result::<DB>")]
    /// Error that happened during execution.
    pub error: Result<(), ContextError<DB::Error>>,
}

impl MiniContext {
    pub fn from_context(context: Context) -> Self {
        Self {
            block: context.block,
            tx: context.tx,
            cfg: context.cfg,
            journaled_state: context.journaled_state,
            chain: context.chain,
            error: context.error,
        }
    }
}

fn default_result<DB: Database>() -> Result<(), ContextError<DB::Error>> {
    Ok(())
}

impl From<MiniContext> for Context {
    fn from(mini_context: MiniContext) -> Self {
        Self {
            block: mini_context.block,
            tx: mini_context.tx,
            cfg: mini_context.cfg,
            journaled_state: mini_context.journaled_state,
            chain: mini_context.chain,
            error: mini_context.error,
        }
    }
}

pub fn deserialize_input(data: &[u8]) -> (Interpreter, Context) {
    // Check minimum length for headers (16 bytes for two u64 lengths)
    if data.len() < 16 {
        panic!("Data too short for headers");
    }

    // Read the lengths from the first 16 bytes
    let sc_len = u64::from_le_bytes(data[0..8].try_into().unwrap()) as usize;
    let mc_len = u64::from_le_bytes(data[8..16].try_into().unwrap()) as usize;

    // Check total length
    let expected_len = 16 + sc_len + mc_len;
    if data.len() != expected_len {
        // return Err(format!(
        //     "Data length mismatch: expected {}, got {}",
        //     expected_len,
        //     data.len()
        // )
        // .into());
        panic!(
            "Data length mismatch: expected {}, got {}",
            expected_len,
            data.len()
        );
    }

    // Extract the context bytes and deserialize
    let context_bytes = &data[16..16 + sc_len];
    let mini_context: MiniContext = serde_json::from_slice(context_bytes).unwrap();
    let context = Context::from(mini_context);

    // Extract the interpreter bytes
    let interpreter_bytes = &data[16 + sc_len..16 + sc_len + mc_len];
    let interpreter: Interpreter = serde_json::from_slice(interpreter_bytes).unwrap();

    (interpreter, context)
}

pub fn serialize_output(
    s_interpreter: &[u8],
    context: &Context,
    out: &InterpreterAction,
) -> Vec<u8> {
    let mini_context = MiniContext::from_context(context.clone());
    let s_context = serde_json::to_vec(&mini_context).unwrap();
    let s_out = serde_json::to_vec(out).unwrap();

    let sc_len = s_context.len();
    let mc_len = s_interpreter.len();
    let out_len = s_out.len();

    let mut serialized = Vec::with_capacity(sc_len + mc_len + out_len + 24);

    serialized.extend((sc_len as u64).to_le_bytes());
    serialized.extend((mc_len as u64).to_le_bytes());
    serialized.extend((out_len as u64).to_le_bytes());
    serialized.extend(s_context);
    serialized.extend(s_interpreter);
    serialized.extend(s_out);

    serialized
}

use reth::revm::{
    context::{BlockEnv, CfgEnv, JournalTr, TxEnv},
    context_interface::context::ContextError,
    db::EmptyDB,
    interpreter::{interpreter::EthInterpreter, Interpreter, InterpreterAction},
    Context, Database, Journal,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct InputRaw {
    pub context: MiniContext,
    pub interpreter: Interpreter,
}

pub struct Input {
    pub context: Context,
    pub interpreter: Interpreter<EthInterpreter>,
}

#[derive(Serialize, Deserialize)]
pub struct OutputRaw {
    context: MiniContext,
    interpreter: Interpreter,
    out: InterpreterAction,
}

pub struct Output {
    pub context: Context,
    pub interpreter: Interpreter,
    pub out: InterpreterAction,
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

fn default_result<DB: Database>() -> Result<(), ContextError<DB::Error>> {
    Ok(())
}

pub fn serialize_input(
    s_interpreter: &[u8],
    context: &Context,
) -> Result<Vec<u8>, serde_json::Error> {
    let mini_context = MiniContext::from_context(context.clone());
    let s_context = serde_json::to_vec(&mini_context)?;

    let sc_len = s_context.len();
    let mc_len = s_interpreter.len();

    let mut serialized = Vec::with_capacity(sc_len + mc_len + 16);

    serialized.extend((sc_len as u64).to_le_bytes());
    serialized.extend((mc_len as u64).to_le_bytes());
    serialized.extend(s_context);
    serialized.extend(s_interpreter);

    Ok(serialized)
}

pub fn deserialize_input(data: &[u8]) -> Result<(Vec<u8>, Context), Box<dyn std::error::Error>> {
    // Check minimum length for headers (16 bytes for two u64 lengths)
    if data.len() < 16 {
        return Err("Data too short for headers".into());
    }

    // Read the lengths from the first 16 bytes
    let sc_len = u64::from_le_bytes(data[0..8].try_into()?) as usize;
    let mc_len = u64::from_le_bytes(data[8..16].try_into()?) as usize;

    // Check total length
    let expected_len = 16 + sc_len + mc_len;
    if data.len() != expected_len {
        return Err(format!(
            "Data length mismatch: expected {}, got {}",
            expected_len,
            data.len()
        )
        .into());
    }

    // Extract the context bytes and deserialize
    let context_bytes = &data[16..16 + sc_len];
    let mini_context: MiniContext = serde_json::from_slice(context_bytes)?;
    let context = Context::from(mini_context);

    // Extract the interpreter bytes
    let interpreter_bytes = &data[16 + sc_len..16 + sc_len + mc_len];

    Ok((interpreter_bytes.to_vec(), context))
}

pub fn serialize_output(
    s_interpreter: &[u8],
    context: &Context,
    out: &InterpreterAction,
) -> Result<Vec<u8>, serde_json::Error> {
    let mini_context = MiniContext::from_context(context.clone());
    let s_context = serde_json::to_vec(&mini_context)?;
    let s_out = serde_json::to_vec(out)?;

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

    Ok(serialized)
}

pub fn deserialize_output_bytes(
    data: &[u8],
) -> Result<(Vec<u8>, Context, InterpreterAction), Box<dyn std::error::Error>> {
    // Check minimum length for headers (24 bytes for three u64 lengths)
    if data.len() < 24 {
        return Err("Data too short for headers".into());
    }

    // Read the lengths from the first 24 bytes
    let sc_len = u64::from_le_bytes(data[0..8].try_into()?) as usize;
    let mc_len = u64::from_le_bytes(data[8..16].try_into()?) as usize;
    let out_len = u64::from_le_bytes(data[16..24].try_into()?) as usize;

    // Check total length
    let expected_len = 24 + sc_len + mc_len + out_len;
    if data.len() != expected_len {
        return Err(format!(
            "Data length mismatch: expected {}, got {}",
            expected_len,
            data.len()
        )
        .into());
    }

    // Extract the context bytes and deserialize
    let context_bytes = &data[24..24 + sc_len];
    let mini_context: MiniContext = serde_json::from_slice(context_bytes)?;
    let context = Context::from(mini_context);

    // Extract the interpreter bytes
    let interpreter_bytes = &data[24 + sc_len..24 + sc_len + mc_len];

    // Extract the output bytes and deserialize
    let out_bytes = &data[24 + sc_len + mc_len..24 + sc_len + mc_len + out_len];
    let out: InterpreterAction = serde_json::from_slice(out_bytes)?;

    Ok((interpreter_bytes.to_vec(), context, out))
}

pub fn deserialize_output(
    data: &[u8],
) -> Result<(Interpreter, Context, InterpreterAction), Box<dyn std::error::Error>> {
    // Check minimum length for headers (24 bytes for three u64 lengths)
    if data.len() < 24 {
        return Err("Data too short for headers".into());
    }

    // Read the lengths from the first 24 bytes
    let sc_len = u64::from_le_bytes(data[0..8].try_into()?) as usize;
    let mc_len = u64::from_le_bytes(data[8..16].try_into()?) as usize;
    let out_len = u64::from_le_bytes(data[16..24].try_into()?) as usize;

    // Check total length
    let expected_len = 24 + sc_len + mc_len + out_len;
    if data.len() != expected_len {
        return Err(format!(
            "Data length mismatch: expected {}, got {}",
            expected_len,
            data.len()
        )
        .into());
    }

    // Extract the context bytes and deserialize
    let context_bytes = &data[24..24 + sc_len];
    let mini_context: MiniContext = serde_json::from_slice(context_bytes)?;
    let context = Context::from(mini_context);

    // Extract the interpreter bytes
    let interpreter_bytes = &data[24 + sc_len..24 + sc_len + mc_len];

    // Extract the output bytes and deserialize
    let out_bytes = &data[24 + sc_len + mc_len..24 + sc_len + mc_len + out_len];
    let out: InterpreterAction = serde_json::from_slice(out_bytes)?;

    let interpreter: Interpreter = serde_json::from_slice(interpreter_bytes)?;

    Ok((interpreter, context, out))
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

#[cfg(test)]
mod tests {
    use super::*;
    use reth::{
        primitives::TxType,
        revm::{
            context::{BlockEnv, CfgEnv, TxEnv},
            db::EmptyDB,
            interpreter::{instruction_table, Gas, InstructionResult, InterpreterResult},
            primitives::{Address, Bytes, TxKind, B256, U256},
            Journal,
        },
        rpc::types::AccessList,
    };

    fn create_test_context() -> Context {
        Context {
            block: BlockEnv {
                number: 100,
                beneficiary: Address::from([1u8; 20]),
                timestamp: 1234567890,
                gas_limit: 8000000,
                basefee: 1000000000,
                difficulty: U256::from(12345),
                prevrandao: Some(B256::from([2u8; 32])),
                blob_excess_gas_and_price: None,
            },
            tx: TxEnv {
                caller: Address::from([3u8; 20]),
                gas_limit: 21000,
                gas_price: 20000000000,
                value: U256::from(1000000000000000000u64), // 1 ETH
                data: reth::revm::primitives::Bytes::from(vec![0x60, 0x80, 0x60, 0x40]),
                nonce: 42,
                chain_id: Some(1),
                access_list: AccessList::default(),
                gas_priority_fee: None,
                blob_hashes: Vec::new(),
                max_fee_per_blob_gas: 100,
                authorization_list: Vec::new(),
                tx_type: TxType::Legacy as u8,
                kind: TxKind::Create,
            },
            cfg: CfgEnv::new(),
            journaled_state: Journal::new(EmptyDB::default()),
            chain: (),
            error: Ok(()),
        }
    }

    fn create_test_interpreter_bytes() -> Vec<u8> {
        // Mock interpreter bytes - in real usage this would be properly serialized interpreter data
        vec![
            123, 34, 98, 121, 116, 101, 99, 111, 100, 101, 34, 58, 123, 34, 98, 97, 115, 101, 34,
            58, 123, 34, 76, 101, 103, 97, 99, 121, 65, 110, 97, 108, 121, 122, 101, 100, 34, 58,
            123, 34, 98, 121, 116, 101, 99, 111, 100, 101, 34, 58, 34, 48, 120, 54, 48, 56, 48, 56,
            48, 54, 48, 52, 48, 53, 50, 51, 52, 54, 48, 49, 51, 53, 55, 54, 48, 100, 102, 57, 48,
            56, 49, 54, 48, 49, 57, 56, 50, 51, 57, 102, 51, 53, 98, 54, 48, 48, 48, 56, 48, 102,
            100, 102, 101, 54, 48, 56, 48, 56, 48, 54, 48, 52, 48, 53, 50, 54, 48, 48, 52, 51, 54,
            49, 48, 49, 53, 54, 48, 49, 50, 53, 55, 54, 48, 48, 48, 56, 48, 102, 100, 53, 98, 54,
            48, 48, 48, 51, 53, 54, 48, 101, 48, 49, 99, 57, 48, 56, 49, 54, 51, 51, 102, 98, 53,
            99, 49, 99, 98, 49, 52, 54, 48, 57, 50, 53, 55, 56, 49, 54, 51, 56, 51, 56, 49, 102,
            53, 56, 97, 49, 52, 54, 48, 55, 57, 53, 55, 53, 48, 54, 51, 100, 48, 57, 100, 101, 48,
            56, 97, 49, 52, 54, 48, 51, 99, 53, 55, 54, 48, 48, 48, 56, 48, 102, 100, 53, 98, 51,
            52, 54, 48, 55, 52, 53, 55, 54, 48, 48, 48, 51, 54, 54, 48, 48, 51, 49, 57, 48, 49, 49,
            50, 54, 48, 55, 52, 53, 55, 54, 48, 48, 48, 53, 52, 54, 48, 48, 48, 49, 57, 56, 49, 49,
            52, 54, 48, 53, 101, 53, 55, 54, 48, 48, 49, 48, 49, 54, 48, 48, 48, 53, 53, 48, 48,
            53, 98, 54, 51, 52, 101, 52, 56, 55, 98, 55, 49, 54, 48, 101, 48, 49, 98, 54, 48, 48,
            48, 53, 50, 54, 48, 49, 49, 54, 48, 48, 52, 53, 50, 54, 48, 50, 52, 54, 48, 48, 48,
            102, 100, 53, 98, 54, 48, 48, 48, 56, 48, 102, 100, 53, 98, 51, 52, 54, 48, 55, 52, 53,
            55, 54, 48, 48, 48, 51, 54, 54, 48, 48, 51, 49, 57, 48, 49, 49, 50, 54, 48, 55, 52, 53,
            55, 54, 48, 50, 48, 57, 48, 54, 48, 48, 48, 53, 52, 56, 49, 53, 50, 102, 51, 53, 98,
            51, 52, 54, 48, 55, 52, 53, 55, 54, 48, 50, 48, 51, 54, 54, 48, 48, 51, 49, 57, 48, 49,
            49, 50, 54, 48, 55, 52, 53, 55, 54, 48, 48, 52, 51, 53, 54, 48, 48, 48, 53, 53, 48, 48,
            102, 101, 97, 50, 54, 52, 54, 57, 55, 48, 54, 54, 55, 51, 53, 56, 50, 50, 49, 50, 50,
            48, 101, 57, 55, 56, 50, 55, 48, 56, 56, 51, 98, 55, 98, 97, 101, 100, 49, 48, 56, 49,
            48, 99, 52, 48, 55, 57, 99, 57, 52, 49, 53, 49, 50, 101, 57, 51, 97, 55, 98, 97, 49,
            99, 100, 49, 49, 48, 56, 99, 55, 56, 49, 100, 52, 98, 99, 55, 51, 56, 100, 57, 48, 57,
            48, 53, 54, 52, 55, 51, 54, 102, 54, 99, 54, 51, 52, 51, 48, 48, 48, 56, 49, 97, 48,
            48, 51, 51, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 34, 44, 34, 111, 114, 105, 103, 105, 110, 97, 108, 95, 108, 101, 110, 34,
            58, 50, 52, 56, 44, 34, 106, 117, 109, 112, 95, 116, 97, 98, 108, 101, 34, 58, 123, 34,
            111, 114, 100, 101, 114, 34, 58, 34, 98, 105, 116, 118, 101, 99, 58, 58, 111, 114, 100,
            101, 114, 58, 58, 76, 115, 98, 48, 34, 44, 34, 104, 101, 97, 100, 34, 58, 123, 34, 119,
            105, 100, 116, 104, 34, 58, 56, 44, 34, 105, 110, 100, 101, 120, 34, 58, 48, 125, 44,
            34, 98, 105, 116, 115, 34, 58, 50, 56, 49, 44, 34, 100, 97, 116, 97, 34, 58, 91, 48,
            44, 48, 44, 56, 44, 48, 44, 48, 44, 56, 44, 48, 44, 48, 44, 48, 44, 48, 44, 51, 50, 44,
            48, 44, 48, 44, 48, 44, 49, 50, 56, 44, 48, 44, 48, 44, 51, 50, 44, 52, 44, 48, 44, 48,
            44, 56, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48,
            44, 48, 44, 48, 44, 48, 44, 48, 93, 125, 125, 125, 44, 34, 112, 114, 111, 103, 114, 97,
            109, 95, 99, 111, 117, 110, 116, 101, 114, 34, 58, 48, 44, 34, 98, 121, 116, 101, 99,
            111, 100, 101, 95, 104, 97, 115, 104, 34, 58, 34, 48, 120, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 34, 125, 44, 34, 115, 116, 97, 99,
            107, 34, 58, 123, 34, 100, 97, 116, 97, 34, 58, 91, 93, 125, 44, 34, 114, 101, 116,
            117, 114, 110, 95, 100, 97, 116, 97, 34, 58, 34, 48, 120, 34, 44, 34, 109, 101, 109,
            111, 114, 121, 34, 58, 123, 34, 98, 117, 102, 102, 101, 114, 34, 58, 91, 93, 44, 34,
            99, 104, 101, 99, 107, 112, 111, 105, 110, 116, 115, 34, 58, 91, 48, 93, 44, 34, 108,
            97, 115, 116, 95, 99, 104, 101, 99, 107, 112, 111, 105, 110, 116, 34, 58, 48, 125, 44,
            34, 105, 110, 112, 117, 116, 34, 58, 123, 34, 116, 97, 114, 103, 101, 116, 95, 97, 100,
            100, 114, 101, 115, 115, 34, 58, 34, 48, 120, 53, 102, 98, 100, 98, 50, 51, 49, 53, 54,
            55, 56, 97, 102, 101, 99, 98, 51, 54, 55, 102, 48, 51, 50, 100, 57, 51, 102, 54, 52,
            50, 102, 54, 52, 49, 56, 48, 97, 97, 51, 34, 44, 34, 99, 97, 108, 108, 101, 114, 95,
            97, 100, 100, 114, 101, 115, 115, 34, 58, 34, 48, 120, 102, 51, 57, 102, 100, 54, 101,
            53, 49, 97, 97, 100, 56, 56, 102, 54, 102, 52, 99, 101, 54, 97, 98, 56, 56, 50, 55, 50,
            55, 57, 99, 102, 102, 102, 98, 57, 50, 50, 54, 54, 34, 44, 34, 105, 110, 112, 117, 116,
            34, 58, 34, 48, 120, 34, 44, 34, 99, 97, 108, 108, 95, 118, 97, 108, 117, 101, 34, 58,
            34, 48, 120, 48, 34, 125, 44, 34, 115, 117, 98, 95, 114, 111, 117, 116, 105, 110, 101,
            34, 58, 123, 34, 114, 101, 116, 117, 114, 110, 95, 115, 116, 97, 99, 107, 34, 58, 91,
            93, 44, 34, 99, 117, 114, 114, 101, 110, 116, 95, 99, 111, 100, 101, 95, 105, 100, 120,
            34, 58, 48, 125, 44, 34, 99, 111, 110, 116, 114, 111, 108, 34, 58, 123, 34, 105, 110,
            115, 116, 114, 117, 99, 116, 105, 111, 110, 95, 114, 101, 115, 117, 108, 116, 34, 58,
            34, 67, 111, 110, 116, 105, 110, 117, 101, 34, 44, 34, 110, 101, 120, 116, 95, 97, 99,
            116, 105, 111, 110, 34, 58, 34, 78, 111, 110, 101, 34, 44, 34, 103, 97, 115, 34, 58,
            123, 34, 108, 105, 109, 105, 116, 34, 58, 54, 56, 53, 52, 55, 57, 53, 50, 57, 50, 50,
            48, 49, 48, 49, 55, 54, 44, 34, 114, 101, 109, 97, 105, 110, 105, 110, 103, 34, 58, 54,
            56, 53, 52, 55, 57, 53, 50, 57, 50, 50, 48, 49, 48, 49, 55, 54, 44, 34, 114, 101, 102,
            117, 110, 100, 101, 100, 34, 58, 48, 44, 34, 109, 101, 109, 111, 114, 121, 34, 58, 123,
            34, 119, 111, 114, 100, 115, 95, 110, 117, 109, 34, 58, 48, 44, 34, 101, 120, 112, 97,
            110, 115, 105, 111, 110, 95, 99, 111, 115, 116, 34, 58, 48, 125, 125, 125, 44, 34, 114,
            117, 110, 116, 105, 109, 101, 95, 102, 108, 97, 103, 34, 58, 123, 34, 105, 115, 95,
            115, 116, 97, 116, 105, 99, 34, 58, 102, 97, 108, 115, 101, 44, 34, 105, 115, 95, 101,
            111, 102, 95, 105, 110, 105, 116, 34, 58, 102, 97, 108, 115, 101, 44, 34, 105, 115, 95,
            101, 111, 102, 34, 58, 102, 97, 108, 115, 101, 44, 34, 115, 112, 101, 99, 95, 105, 100,
            34, 58, 34, 80, 82, 65, 71, 85, 69, 34, 125, 44, 34, 101, 120, 116, 101, 110, 100, 34,
            58, 110, 117, 108, 108, 125,
        ]
    }

    fn create_test_interpreter_action() -> InterpreterAction {
        InterpreterAction::Return {
            result: InterpreterResult {
                result: InstructionResult::Return,
                output: Bytes::from(vec![0xa1, 0xa2, 0xa3, 0xa4]),
                gas: Gas::new(21000),
            },
        }
    }
    
    #[test]
    fn test_serialize_interpreter() {
        let tt = Interpreter::default();
        let c = serde_json::to_vec(&tt).unwrap();
        let cc: Interpreter = serde_json::from_slice(&c).unwrap();
    }

    #[test]
    fn test_mini_context_from_context() {
        let context = create_test_context();
        let mini_context = MiniContext::from_context(context.clone());

        assert_eq!(mini_context.block.number, context.block.number);
        assert_eq!(mini_context.tx.caller, context.tx.caller);
        assert_eq!(mini_context.cfg.chain_id, context.cfg.chain_id);
        assert_eq!(mini_context.chain, context.chain);
    }

    #[test]
    fn test_context_from_mini_context() {
        let original_context = create_test_context();
        let mini_context = MiniContext::from_context(original_context.clone());
        let restored_context = Context::from(mini_context);

        assert_eq!(restored_context.block.number, original_context.block.number);
        assert_eq!(restored_context.tx.caller, original_context.tx.caller);
        assert_eq!(restored_context.cfg.chain_id, original_context.cfg.chain_id);
        assert_eq!(restored_context.chain, original_context.chain);
    }

    #[test]
    fn test_serialize_input() {
        let context = create_test_context();
        let interpreter_bytes = create_test_interpreter_bytes();

        let result = serialize_input(&interpreter_bytes, &context);
        assert!(result.is_ok());

        let serialized = result.unwrap();

        // Check that we have at least the headers (16 bytes)
        assert!(serialized.len() >= 16);

        // Check that the lengths are correctly encoded
        let sc_len = u64::from_le_bytes(serialized[0..8].try_into().unwrap()) as usize;
        let mc_len = u64::from_le_bytes(serialized[8..16].try_into().unwrap()) as usize;

        assert_eq!(mc_len, interpreter_bytes.len());
        assert_eq!(serialized.len(), 16 + sc_len + mc_len);
    }

    #[test]
    fn test_deserialize_input() {
        let context = create_test_context();
        let interpreter_bytes = create_test_interpreter_bytes();

        // First serialize
        let serialized = serialize_input(&interpreter_bytes, &context).unwrap();

        // Then deserialize
        let result = deserialize_input(&serialized);
        assert!(result.is_ok());

        let (deserialized_interpreter, deserialized_context) = result.unwrap();

        // Check interpreter bytes
        assert_eq!(deserialized_interpreter, interpreter_bytes);

        // Check context fields
        assert_eq!(deserialized_context.block.number, context.block.number);
        assert_eq!(deserialized_context.tx.caller, context.tx.caller);
        assert_eq!(deserialized_context.cfg.chain_id, context.cfg.chain_id);
    }

    #[test]
    fn test_round_trip_serialization() {
        let original_context = create_test_context();
        let original_interpreter_bytes = create_test_interpreter_bytes();

        // Serialize
        let serialized = serialize_input(&original_interpreter_bytes, &original_context)
            .expect("Serialization should succeed");

        // Deserialize
        let (restored_interpreter_bytes, restored_context) =
            deserialize_input(&serialized).expect("Deserialization should succeed");

        // Verify interpreter bytes are identical
        assert_eq!(restored_interpreter_bytes, original_interpreter_bytes);

        // Verify context fields are identical
        assert_eq!(restored_context.block.number, original_context.block.number);
        assert_eq!(
            restored_context.block.timestamp,
            original_context.block.timestamp
        );
        assert_eq!(
            restored_context.block.gas_limit,
            original_context.block.gas_limit
        );
        assert_eq!(
            restored_context.block.basefee,
            original_context.block.basefee
        );

        assert_eq!(restored_context.tx.caller, original_context.tx.caller);
        assert_eq!(restored_context.tx.gas_limit, original_context.tx.gas_limit);
        assert_eq!(restored_context.tx.gas_price, original_context.tx.gas_price);
        assert_eq!(restored_context.tx.value, original_context.tx.value);
        assert_eq!(restored_context.tx.data, original_context.tx.data);
        assert_eq!(restored_context.tx.nonce, original_context.tx.nonce);

        assert_eq!(restored_context.cfg.chain_id, original_context.cfg.chain_id);

        assert_eq!(restored_context.chain, original_context.chain);
    }

    #[test]
    fn test_deserialize_invalid_data() {
        // Test with data too short for headers
        let short_data = vec![0x01, 0x02, 0x03];
        let result = deserialize_input(&short_data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Data too short"));

        // Test with invalid length
        let mut invalid_data = vec![0u8; 16];
        // Set context length to 100 but don't provide enough data
        invalid_data[0..8].copy_from_slice(&(100u64).to_le_bytes());
        invalid_data[8..16].copy_from_slice(&(50u64).to_le_bytes());

        let result = deserialize_input(&invalid_data);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Data length mismatch"));
    }

    #[test]
    fn test_empty_interpreter_bytes() {
        let context = create_test_context();
        let empty_interpreter_bytes = Vec::new();

        let serialized = serialize_input(&empty_interpreter_bytes, &context).unwrap();
        let (restored_interpreter_bytes, restored_context) =
            deserialize_input(&serialized).unwrap();

        assert_eq!(restored_interpreter_bytes, empty_interpreter_bytes);
        assert_eq!(restored_context.block.number, context.block.number);
    }

    #[test]
    fn test_large_interpreter_bytes() {
        let context = create_test_context();
        let large_interpreter_bytes = vec![0x42; 10000]; // 10KB of data

        let serialized = serialize_input(&large_interpreter_bytes, &context).unwrap();
        let (restored_interpreter_bytes, restored_context) =
            deserialize_input(&serialized).unwrap();

        assert_eq!(restored_interpreter_bytes, large_interpreter_bytes);
        assert_eq!(restored_context.tx.caller, context.tx.caller);
    }

    #[test]
    fn test_serialize_output() {
        let context = super::tests::create_test_context();
        let interpreter_bytes = super::tests::create_test_interpreter_bytes();
        let out = create_test_interpreter_action();

        let result = serialize_output(&interpreter_bytes, &context, &out);
        assert!(result.is_ok());

        let serialized = result.unwrap();

        // Check that we have at least the headers (24 bytes)
        assert!(serialized.len() >= 24);

        // Check that the lengths are correctly encoded
        let sc_len = u64::from_le_bytes(serialized[0..8].try_into().unwrap()) as usize;
        let mc_len = u64::from_le_bytes(serialized[8..16].try_into().unwrap()) as usize;
        let out_len = u64::from_le_bytes(serialized[16..24].try_into().unwrap()) as usize;

        assert_eq!(mc_len, interpreter_bytes.len());
        assert_eq!(serialized.len(), 24 + sc_len + mc_len + out_len);
    }

    #[test]
    fn test_deserialize_output() {
        let context = super::tests::create_test_context();
        let interpreter_bytes = super::tests::create_test_interpreter_bytes();
        let out = create_test_interpreter_action();

        // First serialize
        let serialized = serialize_output(&interpreter_bytes, &context, &out).unwrap();

        // Then deserialize
        let result = deserialize_output_bytes(&serialized);
        assert!(result.is_ok());

        let (deserialized_interpreter, deserialized_context, deserialized_out) = result.unwrap();

        // Check interpreter bytes
        assert_eq!(deserialized_interpreter, interpreter_bytes);

        // Check context fields
        assert_eq!(deserialized_context.block.number, context.block.number);
        assert_eq!(deserialized_context.tx.caller, context.tx.caller);

        // Check output action
        match (&out, &deserialized_out) {
            (
                InterpreterAction::Return { result: orig },
                InterpreterAction::Return { result: deser },
            ) => {
                assert_eq!(orig.result, deser.result);
                assert_eq!(orig.output, deser.output);
                assert_eq!(orig.gas.limit(), deser.gas.limit());
            }
            _ => panic!("Output action type mismatch"),
        }
    }

    #[test]
    fn test_output_round_trip_return_action() {
        let original_context = super::tests::create_test_context();
        let original_interpreter_bytes = super::tests::create_test_interpreter_bytes();
        let original_out = create_test_interpreter_action();

        // Serialize
        let serialized = serialize_output(
            &original_interpreter_bytes,
            &original_context,
            &original_out,
        )
        .expect("Serialization should succeed");

        // Deserialize
        let (restored_interpreter_bytes, restored_context, restored_out) =
            deserialize_output_bytes(&serialized).expect("Deserialization should succeed");

        // Verify interpreter bytes are identical
        assert_eq!(restored_interpreter_bytes, original_interpreter_bytes);

        // Verify context fields are identical
        assert_eq!(restored_context.block.number, original_context.block.number);
        assert_eq!(restored_context.tx.caller, original_context.tx.caller);
        assert_eq!(restored_context.cfg.chain_id, original_context.cfg.chain_id);

        // Verify output action is identical
        match (&original_out, &restored_out) {
            (
                InterpreterAction::Return { result: orig },
                InterpreterAction::Return { result: restored },
            ) => {
                assert_eq!(orig.result, restored.result);
                assert_eq!(orig.output, restored.output);
                assert_eq!(orig.gas.limit(), restored.gas.limit());
                assert_eq!(orig.gas.spent(), restored.gas.spent());
            }
            _ => panic!("Output action type mismatch"),
        }
    }

    #[test]
    fn test_deserialize_output_invalid_data() {
        // Test with data too short for headers
        let short_data = vec![0x01, 0x02, 0x03];
        let result = deserialize_output_bytes(&short_data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Data too short"));

        // Test with invalid length
        let mut invalid_data = vec![0u8; 24];
        // Set lengths that don't match actual data
        invalid_data[0..8].copy_from_slice(&(100u64).to_le_bytes());
        invalid_data[8..16].copy_from_slice(&(50u64).to_le_bytes());
        invalid_data[16..24].copy_from_slice(&(25u64).to_le_bytes());

        let result = deserialize_output_bytes(&invalid_data);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Data length mismatch"));
    }

    #[test]
    fn test_output_with_empty_interpreter_bytes() {
        let context = super::tests::create_test_context();
        let empty_interpreter_bytes = Vec::new();
        let out = create_test_interpreter_action();

        let serialized = serialize_output(&empty_interpreter_bytes, &context, &out).unwrap();
        let (restored_interpreter_bytes, restored_context, restored_out) =
            deserialize_output_bytes(&serialized).unwrap();

        assert_eq!(restored_interpreter_bytes, empty_interpreter_bytes);
        assert_eq!(restored_context.block.number, context.block.number);

        // Verify the output action was preserved
        match (&out, &restored_out) {
            (
                InterpreterAction::Return { result: orig },
                InterpreterAction::Return { result: restored },
            ) => {
                assert_eq!(orig.result, restored.result);
            }
            _ => panic!("Output action type mismatch"),
        }
    }

    #[test]
    fn test_output_serialization_format() {
        let context = super::tests::create_test_context();
        let interpreter_bytes = vec![0x11, 0x22, 0x33];
        let out = create_test_interpreter_action();

        let serialized = serialize_output(&interpreter_bytes, &context, &out).unwrap();

        // Verify the format: 24 bytes header + context + interpreter + output
        let sc_len = u64::from_le_bytes(serialized[0..8].try_into().unwrap()) as usize;
        let mc_len = u64::from_le_bytes(serialized[8..16].try_into().unwrap()) as usize;
        let out_len = u64::from_le_bytes(serialized[16..24].try_into().unwrap()) as usize;

        assert_eq!(mc_len, 3); // Our test interpreter bytes length
        assert!(sc_len > 0); // Context should have some length
        assert!(out_len > 0); // Output should have some length
        assert_eq!(serialized.len(), 24 + sc_len + mc_len + out_len);

        // Verify we can extract the interpreter bytes from the right position
        let extracted_interpreter = &serialized[24 + sc_len..24 + sc_len + mc_len];
        assert_eq!(extracted_interpreter, &interpreter_bytes);
    }

    #[test]
    fn test_interaglly() {
        let serial_input: &[u8] = &[
            2, 6, 0, 0, 0, 0, 0, 0, 244, 5, 0, 0, 0, 0, 0, 0, 123, 34, 98, 108, 111, 99, 107, 34,
            58, 123, 34, 110, 117, 109, 98, 101, 114, 34, 58, 49, 44, 34, 98, 101, 110, 101, 102,
            105, 99, 105, 97, 114, 121, 34, 58, 34, 48, 120, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 34, 44, 34, 116, 105, 109, 101, 115, 116, 97, 109,
            112, 34, 58, 49, 50, 44, 34, 103, 97, 115, 95, 108, 105, 109, 105, 116, 34, 58, 54, 56,
            53, 52, 55, 57, 53, 50, 57, 50, 50, 48, 54, 54, 57, 52, 52, 44, 34, 98, 97, 115, 101,
            102, 101, 101, 34, 58, 56, 55, 53, 48, 48, 48, 48, 48, 48, 44, 34, 100, 105, 102, 102,
            105, 99, 117, 108, 116, 121, 34, 58, 34, 48, 120, 48, 34, 44, 34, 112, 114, 101, 118,
            114, 97, 110, 100, 97, 111, 34, 58, 34, 48, 120, 49, 99, 56, 101, 97, 55, 52, 49, 98,
            48, 54, 98, 50, 52, 97, 98, 56, 48, 99, 99, 55, 48, 49, 52, 48, 57, 55, 52, 101, 51,
            57, 101, 98, 54, 101, 98, 53, 48, 100, 98, 49, 99, 55, 51, 51, 97, 52, 57, 101, 97, 49,
            57, 51, 48, 50, 57, 48, 51, 55, 99, 53, 52, 49, 56, 34, 44, 34, 98, 108, 111, 98, 95,
            101, 120, 99, 101, 115, 115, 95, 103, 97, 115, 95, 97, 110, 100, 95, 112, 114, 105, 99,
            101, 34, 58, 123, 34, 101, 120, 99, 101, 115, 115, 95, 98, 108, 111, 98, 95, 103, 97,
            115, 34, 58, 48, 44, 34, 98, 108, 111, 98, 95, 103, 97, 115, 112, 114, 105, 99, 101,
            34, 58, 49, 125, 125, 44, 34, 116, 120, 34, 58, 123, 34, 116, 120, 95, 116, 121, 112,
            101, 34, 58, 48, 44, 34, 99, 97, 108, 108, 101, 114, 34, 58, 34, 48, 120, 102, 51, 57,
            102, 100, 54, 101, 53, 49, 97, 97, 100, 56, 56, 102, 54, 102, 52, 99, 101, 54, 97, 98,
            56, 56, 50, 55, 50, 55, 57, 99, 102, 102, 102, 98, 57, 50, 50, 54, 54, 34, 44, 34, 103,
            97, 115, 95, 108, 105, 109, 105, 116, 34, 58, 54, 56, 53, 52, 55, 57, 53, 50, 57, 50,
            50, 48, 54, 54, 57, 52, 52, 44, 34, 103, 97, 115, 95, 112, 114, 105, 99, 101, 34, 58,
            48, 44, 34, 107, 105, 110, 100, 34, 58, 110, 117, 108, 108, 44, 34, 118, 97, 108, 117,
            101, 34, 58, 34, 48, 120, 48, 34, 44, 34, 100, 97, 116, 97, 34, 58, 34, 48, 120, 54,
            48, 56, 48, 56, 48, 54, 48, 52, 48, 53, 50, 51, 52, 54, 48, 49, 51, 53, 55, 54, 48,
            100, 102, 57, 48, 56, 49, 54, 48, 49, 57, 56, 50, 51, 57, 102, 51, 53, 98, 54, 48, 48,
            48, 56, 48, 102, 100, 102, 101, 54, 48, 56, 48, 56, 48, 54, 48, 52, 48, 53, 50, 54, 48,
            48, 52, 51, 54, 49, 48, 49, 53, 54, 48, 49, 50, 53, 55, 54, 48, 48, 48, 56, 48, 102,
            100, 53, 98, 54, 48, 48, 48, 51, 53, 54, 48, 101, 48, 49, 99, 57, 48, 56, 49, 54, 51,
            51, 102, 98, 53, 99, 49, 99, 98, 49, 52, 54, 48, 57, 50, 53, 55, 56, 49, 54, 51, 56,
            51, 56, 49, 102, 53, 56, 97, 49, 52, 54, 48, 55, 57, 53, 55, 53, 48, 54, 51, 100, 48,
            57, 100, 101, 48, 56, 97, 49, 52, 54, 48, 51, 99, 53, 55, 54, 48, 48, 48, 56, 48, 102,
            100, 53, 98, 51, 52, 54, 48, 55, 52, 53, 55, 54, 48, 48, 48, 51, 54, 54, 48, 48, 51,
            49, 57, 48, 49, 49, 50, 54, 48, 55, 52, 53, 55, 54, 48, 48, 48, 53, 52, 54, 48, 48, 48,
            49, 57, 56, 49, 49, 52, 54, 48, 53, 101, 53, 55, 54, 48, 48, 49, 48, 49, 54, 48, 48,
            48, 53, 53, 48, 48, 53, 98, 54, 51, 52, 101, 52, 56, 55, 98, 55, 49, 54, 48, 101, 48,
            49, 98, 54, 48, 48, 48, 53, 50, 54, 48, 49, 49, 54, 48, 48, 52, 53, 50, 54, 48, 50, 52,
            54, 48, 48, 48, 102, 100, 53, 98, 54, 48, 48, 48, 56, 48, 102, 100, 53, 98, 51, 52, 54,
            48, 55, 52, 53, 55, 54, 48, 48, 48, 51, 54, 54, 48, 48, 51, 49, 57, 48, 49, 49, 50, 54,
            48, 55, 52, 53, 55, 54, 48, 50, 48, 57, 48, 54, 48, 48, 48, 53, 52, 56, 49, 53, 50,
            102, 51, 53, 98, 51, 52, 54, 48, 55, 52, 53, 55, 54, 48, 50, 48, 51, 54, 54, 48, 48,
            51, 49, 57, 48, 49, 49, 50, 54, 48, 55, 52, 53, 55, 54, 48, 48, 52, 51, 53, 54, 48, 48,
            48, 53, 53, 48, 48, 102, 101, 97, 50, 54, 52, 54, 57, 55, 48, 54, 54, 55, 51, 53, 56,
            50, 50, 49, 50, 50, 48, 101, 57, 55, 56, 50, 55, 48, 56, 56, 51, 98, 55, 98, 97, 101,
            100, 49, 48, 56, 49, 48, 99, 52, 48, 55, 57, 99, 57, 52, 49, 53, 49, 50, 101, 57, 51,
            97, 55, 98, 97, 49, 99, 100, 49, 49, 48, 56, 99, 55, 56, 49, 100, 52, 98, 99, 55, 51,
            56, 100, 57, 48, 57, 48, 53, 54, 52, 55, 51, 54, 102, 54, 99, 54, 51, 52, 51, 48, 48,
            48, 56, 49, 97, 48, 48, 51, 51, 34, 44, 34, 110, 111, 110, 99, 101, 34, 58, 48, 44, 34,
            99, 104, 97, 105, 110, 95, 105, 100, 34, 58, 49, 44, 34, 97, 99, 99, 101, 115, 115, 95,
            108, 105, 115, 116, 34, 58, 91, 93, 44, 34, 103, 97, 115, 95, 112, 114, 105, 111, 114,
            105, 116, 121, 95, 102, 101, 101, 34, 58, 110, 117, 108, 108, 44, 34, 98, 108, 111, 98,
            95, 104, 97, 115, 104, 101, 115, 34, 58, 91, 93, 44, 34, 109, 97, 120, 95, 102, 101,
            101, 95, 112, 101, 114, 95, 98, 108, 111, 98, 95, 103, 97, 115, 34, 58, 48, 44, 34, 97,
            117, 116, 104, 111, 114, 105, 122, 97, 116, 105, 111, 110, 95, 108, 105, 115, 116, 34,
            58, 91, 93, 125, 44, 34, 99, 102, 103, 34, 58, 123, 34, 99, 104, 97, 105, 110, 95, 105,
            100, 34, 58, 49, 44, 34, 115, 112, 101, 99, 34, 58, 34, 80, 82, 65, 71, 85, 69, 34, 44,
            34, 108, 105, 109, 105, 116, 95, 99, 111, 110, 116, 114, 97, 99, 116, 95, 99, 111, 100,
            101, 95, 115, 105, 122, 101, 34, 58, 110, 117, 108, 108, 44, 34, 100, 105, 115, 97, 98,
            108, 101, 95, 110, 111, 110, 99, 101, 95, 99, 104, 101, 99, 107, 34, 58, 102, 97, 108,
            115, 101, 44, 34, 98, 108, 111, 98, 95, 116, 97, 114, 103, 101, 116, 95, 97, 110, 100,
            95, 109, 97, 120, 95, 99, 111, 117, 110, 116, 34, 58, 91, 91, 34, 67, 65, 78, 67, 85,
            78, 34, 44, 51, 44, 54, 93, 44, 91, 34, 80, 82, 65, 71, 85, 69, 34, 44, 54, 44, 57, 93,
            93, 44, 34, 100, 105, 115, 97, 98, 108, 101, 95, 98, 108, 111, 99, 107, 95, 103, 97,
            115, 95, 108, 105, 109, 105, 116, 34, 58, 102, 97, 108, 115, 101, 44, 34, 100, 105,
            115, 97, 98, 108, 101, 95, 101, 105, 112, 51, 54, 48, 55, 34, 58, 102, 97, 108, 115,
            101, 44, 34, 100, 105, 115, 97, 98, 108, 101, 95, 98, 97, 115, 101, 95, 102, 101, 101,
            34, 58, 102, 97, 108, 115, 101, 125, 44, 34, 106, 111, 117, 114, 110, 97, 108, 101,
            100, 95, 115, 116, 97, 116, 101, 34, 58, 123, 34, 100, 97, 116, 97, 98, 97, 115, 101,
            34, 58, 123, 34, 95, 112, 104, 97, 110, 116, 111, 109, 34, 58, 110, 117, 108, 108, 125,
            44, 34, 105, 110, 110, 101, 114, 34, 58, 123, 34, 115, 116, 97, 116, 101, 34, 58, 123,
            125, 44, 34, 116, 114, 97, 110, 115, 105, 101, 110, 116, 95, 115, 116, 111, 114, 97,
            103, 101, 34, 58, 123, 125, 44, 34, 108, 111, 103, 115, 34, 58, 91, 93, 44, 34, 100,
            101, 112, 116, 104, 34, 58, 48, 44, 34, 106, 111, 117, 114, 110, 97, 108, 34, 58, 91,
            91, 93, 93, 44, 34, 115, 112, 101, 99, 34, 58, 34, 80, 82, 65, 71, 85, 69, 34, 44, 34,
            119, 97, 114, 109, 95, 112, 114, 101, 108, 111, 97, 100, 101, 100, 95, 97, 100, 100,
            114, 101, 115, 115, 101, 115, 34, 58, 91, 93, 44, 34, 112, 114, 101, 99, 111, 109, 112,
            105, 108, 101, 115, 34, 58, 91, 93, 125, 125, 44, 34, 99, 104, 97, 105, 110, 34, 58,
            110, 117, 108, 108, 125, 123, 34, 98, 121, 116, 101, 99, 111, 100, 101, 34, 58, 123,
            34, 98, 97, 115, 101, 34, 58, 123, 34, 76, 101, 103, 97, 99, 121, 65, 110, 97, 108,
            121, 122, 101, 100, 34, 58, 123, 34, 98, 121, 116, 101, 99, 111, 100, 101, 34, 58, 34,
            48, 120, 54, 48, 56, 48, 56, 48, 54, 48, 52, 48, 53, 50, 51, 52, 54, 48, 49, 51, 53,
            55, 54, 48, 100, 102, 57, 48, 56, 49, 54, 48, 49, 57, 56, 50, 51, 57, 102, 51, 53, 98,
            54, 48, 48, 48, 56, 48, 102, 100, 102, 101, 54, 48, 56, 48, 56, 48, 54, 48, 52, 48, 53,
            50, 54, 48, 48, 52, 51, 54, 49, 48, 49, 53, 54, 48, 49, 50, 53, 55, 54, 48, 48, 48, 56,
            48, 102, 100, 53, 98, 54, 48, 48, 48, 51, 53, 54, 48, 101, 48, 49, 99, 57, 48, 56, 49,
            54, 51, 51, 102, 98, 53, 99, 49, 99, 98, 49, 52, 54, 48, 57, 50, 53, 55, 56, 49, 54,
            51, 56, 51, 56, 49, 102, 53, 56, 97, 49, 52, 54, 48, 55, 57, 53, 55, 53, 48, 54, 51,
            100, 48, 57, 100, 101, 48, 56, 97, 49, 52, 54, 48, 51, 99, 53, 55, 54, 48, 48, 48, 56,
            48, 102, 100, 53, 98, 51, 52, 54, 48, 55, 52, 53, 55, 54, 48, 48, 48, 51, 54, 54, 48,
            48, 51, 49, 57, 48, 49, 49, 50, 54, 48, 55, 52, 53, 55, 54, 48, 48, 48, 53, 52, 54, 48,
            48, 48, 49, 57, 56, 49, 49, 52, 54, 48, 53, 101, 53, 55, 54, 48, 48, 49, 48, 49, 54,
            48, 48, 48, 53, 53, 48, 48, 53, 98, 54, 51, 52, 101, 52, 56, 55, 98, 55, 49, 54, 48,
            101, 48, 49, 98, 54, 48, 48, 48, 53, 50, 54, 48, 49, 49, 54, 48, 48, 52, 53, 50, 54,
            48, 50, 52, 54, 48, 48, 48, 102, 100, 53, 98, 54, 48, 48, 48, 56, 48, 102, 100, 53, 98,
            51, 52, 54, 48, 55, 52, 53, 55, 54, 48, 48, 48, 51, 54, 54, 48, 48, 51, 49, 57, 48, 49,
            49, 50, 54, 48, 55, 52, 53, 55, 54, 48, 50, 48, 57, 48, 54, 48, 48, 48, 53, 52, 56, 49,
            53, 50, 102, 51, 53, 98, 51, 52, 54, 48, 55, 52, 53, 55, 54, 48, 50, 48, 51, 54, 54,
            48, 48, 51, 49, 57, 48, 49, 49, 50, 54, 48, 55, 52, 53, 55, 54, 48, 48, 52, 51, 53, 54,
            48, 48, 48, 53, 53, 48, 48, 102, 101, 97, 50, 54, 52, 54, 57, 55, 48, 54, 54, 55, 51,
            53, 56, 50, 50, 49, 50, 50, 48, 101, 57, 55, 56, 50, 55, 48, 56, 56, 51, 98, 55, 98,
            97, 101, 100, 49, 48, 56, 49, 48, 99, 52, 48, 55, 57, 99, 57, 52, 49, 53, 49, 50, 101,
            57, 51, 97, 55, 98, 97, 49, 99, 100, 49, 49, 48, 56, 99, 55, 56, 49, 100, 52, 98, 99,
            55, 51, 56, 100, 57, 48, 57, 48, 53, 54, 52, 55, 51, 54, 102, 54, 99, 54, 51, 52, 51,
            48, 48, 48, 56, 49, 97, 48, 48, 51, 51, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 34, 44, 34, 111, 114, 105, 103, 105, 110, 97,
            108, 95, 108, 101, 110, 34, 58, 50, 52, 56, 44, 34, 106, 117, 109, 112, 95, 116, 97,
            98, 108, 101, 34, 58, 123, 34, 111, 114, 100, 101, 114, 34, 58, 34, 98, 105, 116, 118,
            101, 99, 58, 58, 111, 114, 100, 101, 114, 58, 58, 76, 115, 98, 48, 34, 44, 34, 104,
            101, 97, 100, 34, 58, 123, 34, 119, 105, 100, 116, 104, 34, 58, 56, 44, 34, 105, 110,
            100, 101, 120, 34, 58, 48, 125, 44, 34, 98, 105, 116, 115, 34, 58, 50, 56, 49, 44, 34,
            100, 97, 116, 97, 34, 58, 91, 48, 44, 48, 44, 56, 44, 48, 44, 48, 44, 56, 44, 48, 44,
            48, 44, 48, 44, 48, 44, 51, 50, 44, 48, 44, 48, 44, 48, 44, 49, 50, 56, 44, 48, 44, 48,
            44, 51, 50, 44, 52, 44, 48, 44, 48, 44, 56, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44,
            48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 93, 125, 125, 125,
            44, 34, 112, 114, 111, 103, 114, 97, 109, 95, 99, 111, 117, 110, 116, 101, 114, 34, 58,
            48, 44, 34, 98, 121, 116, 101, 99, 111, 100, 101, 95, 104, 97, 115, 104, 34, 58, 34,
            48, 120, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 34, 125, 44, 34, 115, 116, 97, 99, 107, 34, 58, 123, 34, 100, 97, 116, 97, 34, 58,
            91, 93, 125, 44, 34, 114, 101, 116, 117, 114, 110, 95, 100, 97, 116, 97, 34, 58, 34,
            48, 120, 34, 44, 34, 109, 101, 109, 111, 114, 121, 34, 58, 123, 34, 98, 117, 102, 102,
            101, 114, 34, 58, 91, 93, 44, 34, 99, 104, 101, 99, 107, 112, 111, 105, 110, 116, 115,
            34, 58, 91, 48, 93, 44, 34, 108, 97, 115, 116, 95, 99, 104, 101, 99, 107, 112, 111,
            105, 110, 116, 34, 58, 48, 125, 44, 34, 105, 110, 112, 117, 116, 34, 58, 123, 34, 116,
            97, 114, 103, 101, 116, 95, 97, 100, 100, 114, 101, 115, 115, 34, 58, 34, 48, 120, 53,
            102, 98, 100, 98, 50, 51, 49, 53, 54, 55, 56, 97, 102, 101, 99, 98, 51, 54, 55, 102,
            48, 51, 50, 100, 57, 51, 102, 54, 52, 50, 102, 54, 52, 49, 56, 48, 97, 97, 51, 34, 44,
            34, 99, 97, 108, 108, 101, 114, 95, 97, 100, 100, 114, 101, 115, 115, 34, 58, 34, 48,
            120, 102, 51, 57, 102, 100, 54, 101, 53, 49, 97, 97, 100, 56, 56, 102, 54, 102, 52, 99,
            101, 54, 97, 98, 56, 56, 50, 55, 50, 55, 57, 99, 102, 102, 102, 98, 57, 50, 50, 54, 54,
            34, 44, 34, 105, 110, 112, 117, 116, 34, 58, 34, 48, 120, 34, 44, 34, 99, 97, 108, 108,
            95, 118, 97, 108, 117, 101, 34, 58, 34, 48, 120, 48, 34, 125, 44, 34, 115, 117, 98, 95,
            114, 111, 117, 116, 105, 110, 101, 34, 58, 123, 34, 114, 101, 116, 117, 114, 110, 95,
            115, 116, 97, 99, 107, 34, 58, 91, 93, 44, 34, 99, 117, 114, 114, 101, 110, 116, 95,
            99, 111, 100, 101, 95, 105, 100, 120, 34, 58, 48, 125, 44, 34, 99, 111, 110, 116, 114,
            111, 108, 34, 58, 123, 34, 105, 110, 115, 116, 114, 117, 99, 116, 105, 111, 110, 95,
            114, 101, 115, 117, 108, 116, 34, 58, 34, 67, 111, 110, 116, 105, 110, 117, 101, 34,
            44, 34, 110, 101, 120, 116, 95, 97, 99, 116, 105, 111, 110, 34, 58, 34, 78, 111, 110,
            101, 34, 44, 34, 103, 97, 115, 34, 58, 123, 34, 108, 105, 109, 105, 116, 34, 58, 54,
            56, 53, 52, 55, 57, 53, 50, 57, 50, 50, 48, 49, 48, 49, 55, 54, 44, 34, 114, 101, 109,
            97, 105, 110, 105, 110, 103, 34, 58, 54, 56, 53, 52, 55, 57, 53, 50, 57, 50, 50, 48,
            49, 48, 49, 55, 54, 44, 34, 114, 101, 102, 117, 110, 100, 101, 100, 34, 58, 48, 44, 34,
            109, 101, 109, 111, 114, 121, 34, 58, 123, 34, 119, 111, 114, 100, 115, 95, 110, 117,
            109, 34, 58, 48, 44, 34, 101, 120, 112, 97, 110, 115, 105, 111, 110, 95, 99, 111, 115,
            116, 34, 58, 48, 125, 125, 125, 44, 34, 114, 117, 110, 116, 105, 109, 101, 95, 102,
            108, 97, 103, 34, 58, 123, 34, 105, 115, 95, 115, 116, 97, 116, 105, 99, 34, 58, 102,
            97, 108, 115, 101, 44, 34, 105, 115, 95, 101, 111, 102, 95, 105, 110, 105, 116, 34, 58,
            102, 97, 108, 115, 101, 44, 34, 105, 115, 95, 101, 111, 102, 34, 58, 102, 97, 108, 115,
            101, 44, 34, 115, 112, 101, 99, 95, 105, 100, 34, 58, 34, 80, 82, 65, 71, 85, 69, 34,
            125, 44, 34, 101, 120, 116, 101, 110, 100, 34, 58, 110, 117, 108, 108, 125,
        ];

        let raw_interpreter: &[u8] = &[
            123, 34, 98, 121, 116, 101, 99, 111, 100, 101, 34, 58, 123, 34, 98, 97, 115, 101, 34,
            58, 123, 34, 76, 101, 103, 97, 99, 121, 65, 110, 97, 108, 121, 122, 101, 100, 34, 58,
            123, 34, 98, 121, 116, 101, 99, 111, 100, 101, 34, 58, 34, 48, 120, 54, 48, 56, 48, 56,
            48, 54, 48, 52, 48, 53, 50, 51, 52, 54, 48, 49, 51, 53, 55, 54, 48, 100, 102, 57, 48,
            56, 49, 54, 48, 49, 57, 56, 50, 51, 57, 102, 51, 53, 98, 54, 48, 48, 48, 56, 48, 102,
            100, 102, 101, 54, 48, 56, 48, 56, 48, 54, 48, 52, 48, 53, 50, 54, 48, 48, 52, 51, 54,
            49, 48, 49, 53, 54, 48, 49, 50, 53, 55, 54, 48, 48, 48, 56, 48, 102, 100, 53, 98, 54,
            48, 48, 48, 51, 53, 54, 48, 101, 48, 49, 99, 57, 48, 56, 49, 54, 51, 51, 102, 98, 53,
            99, 49, 99, 98, 49, 52, 54, 48, 57, 50, 53, 55, 56, 49, 54, 51, 56, 51, 56, 49, 102,
            53, 56, 97, 49, 52, 54, 48, 55, 57, 53, 55, 53, 48, 54, 51, 100, 48, 57, 100, 101, 48,
            56, 97, 49, 52, 54, 48, 51, 99, 53, 55, 54, 48, 48, 48, 56, 48, 102, 100, 53, 98, 51,
            52, 54, 48, 55, 52, 53, 55, 54, 48, 48, 48, 51, 54, 54, 48, 48, 51, 49, 57, 48, 49, 49,
            50, 54, 48, 55, 52, 53, 55, 54, 48, 48, 48, 53, 52, 54, 48, 48, 48, 49, 57, 56, 49, 49,
            52, 54, 48, 53, 101, 53, 55, 54, 48, 48, 49, 48, 49, 54, 48, 48, 48, 53, 53, 48, 48,
            53, 98, 54, 51, 52, 101, 52, 56, 55, 98, 55, 49, 54, 48, 101, 48, 49, 98, 54, 48, 48,
            48, 53, 50, 54, 48, 49, 49, 54, 48, 48, 52, 53, 50, 54, 48, 50, 52, 54, 48, 48, 48,
            102, 100, 53, 98, 54, 48, 48, 48, 56, 48, 102, 100, 53, 98, 51, 52, 54, 48, 55, 52, 53,
            55, 54, 48, 48, 48, 51, 54, 54, 48, 48, 51, 49, 57, 48, 49, 49, 50, 54, 48, 55, 52, 53,
            55, 54, 48, 50, 48, 57, 48, 54, 48, 48, 48, 53, 52, 56, 49, 53, 50, 102, 51, 53, 98,
            51, 52, 54, 48, 55, 52, 53, 55, 54, 48, 50, 48, 51, 54, 54, 48, 48, 51, 49, 57, 48, 49,
            49, 50, 54, 48, 55, 52, 53, 55, 54, 48, 48, 52, 51, 53, 54, 48, 48, 48, 53, 53, 48, 48,
            102, 101, 97, 50, 54, 52, 54, 57, 55, 48, 54, 54, 55, 51, 53, 56, 50, 50, 49, 50, 50,
            48, 101, 57, 55, 56, 50, 55, 48, 56, 56, 51, 98, 55, 98, 97, 101, 100, 49, 48, 56, 49,
            48, 99, 52, 48, 55, 57, 99, 57, 52, 49, 53, 49, 50, 101, 57, 51, 97, 55, 98, 97, 49,
            99, 100, 49, 49, 48, 56, 99, 55, 56, 49, 100, 52, 98, 99, 55, 51, 56, 100, 57, 48, 57,
            48, 53, 54, 52, 55, 51, 54, 102, 54, 99, 54, 51, 52, 51, 48, 48, 48, 56, 49, 97, 48,
            48, 51, 51, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 34, 44, 34, 111, 114, 105, 103, 105, 110, 97, 108, 95, 108, 101, 110, 34,
            58, 50, 52, 56, 44, 34, 106, 117, 109, 112, 95, 116, 97, 98, 108, 101, 34, 58, 123, 34,
            111, 114, 100, 101, 114, 34, 58, 34, 98, 105, 116, 118, 101, 99, 58, 58, 111, 114, 100,
            101, 114, 58, 58, 76, 115, 98, 48, 34, 44, 34, 104, 101, 97, 100, 34, 58, 123, 34, 119,
            105, 100, 116, 104, 34, 58, 56, 44, 34, 105, 110, 100, 101, 120, 34, 58, 48, 125, 44,
            34, 98, 105, 116, 115, 34, 58, 50, 56, 49, 44, 34, 100, 97, 116, 97, 34, 58, 91, 48,
            44, 48, 44, 56, 44, 48, 44, 48, 44, 56, 44, 48, 44, 48, 44, 48, 44, 48, 44, 51, 50, 44,
            48, 44, 48, 44, 48, 44, 49, 50, 56, 44, 48, 44, 48, 44, 51, 50, 44, 52, 44, 48, 44, 48,
            44, 56, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48,
            44, 48, 44, 48, 44, 48, 44, 48, 93, 125, 125, 125, 44, 34, 112, 114, 111, 103, 114, 97,
            109, 95, 99, 111, 117, 110, 116, 101, 114, 34, 58, 48, 44, 34, 98, 121, 116, 101, 99,
            111, 100, 101, 95, 104, 97, 115, 104, 34, 58, 34, 48, 120, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 34, 125, 44, 34, 115, 116, 97, 99,
            107, 34, 58, 123, 34, 100, 97, 116, 97, 34, 58, 91, 93, 125, 44, 34, 114, 101, 116,
            117, 114, 110, 95, 100, 97, 116, 97, 34, 58, 34, 48, 120, 34, 44, 34, 109, 101, 109,
            111, 114, 121, 34, 58, 123, 34, 98, 117, 102, 102, 101, 114, 34, 58, 91, 93, 44, 34,
            99, 104, 101, 99, 107, 112, 111, 105, 110, 116, 115, 34, 58, 91, 48, 93, 44, 34, 108,
            97, 115, 116, 95, 99, 104, 101, 99, 107, 112, 111, 105, 110, 116, 34, 58, 48, 125, 44,
            34, 105, 110, 112, 117, 116, 34, 58, 123, 34, 116, 97, 114, 103, 101, 116, 95, 97, 100,
            100, 114, 101, 115, 115, 34, 58, 34, 48, 120, 53, 102, 98, 100, 98, 50, 51, 49, 53, 54,
            55, 56, 97, 102, 101, 99, 98, 51, 54, 55, 102, 48, 51, 50, 100, 57, 51, 102, 54, 52,
            50, 102, 54, 52, 49, 56, 48, 97, 97, 51, 34, 44, 34, 99, 97, 108, 108, 101, 114, 95,
            97, 100, 100, 114, 101, 115, 115, 34, 58, 34, 48, 120, 102, 51, 57, 102, 100, 54, 101,
            53, 49, 97, 97, 100, 56, 56, 102, 54, 102, 52, 99, 101, 54, 97, 98, 56, 56, 50, 55, 50,
            55, 57, 99, 102, 102, 102, 98, 57, 50, 50, 54, 54, 34, 44, 34, 105, 110, 112, 117, 116,
            34, 58, 34, 48, 120, 34, 44, 34, 99, 97, 108, 108, 95, 118, 97, 108, 117, 101, 34, 58,
            34, 48, 120, 48, 34, 125, 44, 34, 115, 117, 98, 95, 114, 111, 117, 116, 105, 110, 101,
            34, 58, 123, 34, 114, 101, 116, 117, 114, 110, 95, 115, 116, 97, 99, 107, 34, 58, 91,
            93, 44, 34, 99, 117, 114, 114, 101, 110, 116, 95, 99, 111, 100, 101, 95, 105, 100, 120,
            34, 58, 48, 125, 44, 34, 99, 111, 110, 116, 114, 111, 108, 34, 58, 123, 34, 105, 110,
            115, 116, 114, 117, 99, 116, 105, 111, 110, 95, 114, 101, 115, 117, 108, 116, 34, 58,
            34, 67, 111, 110, 116, 105, 110, 117, 101, 34, 44, 34, 110, 101, 120, 116, 95, 97, 99,
            116, 105, 111, 110, 34, 58, 34, 78, 111, 110, 101, 34, 44, 34, 103, 97, 115, 34, 58,
            123, 34, 108, 105, 109, 105, 116, 34, 58, 54, 56, 53, 52, 55, 57, 53, 50, 57, 50, 50,
            48, 49, 48, 49, 55, 54, 44, 34, 114, 101, 109, 97, 105, 110, 105, 110, 103, 34, 58, 54,
            56, 53, 52, 55, 57, 53, 50, 57, 50, 50, 48, 49, 48, 49, 55, 54, 44, 34, 114, 101, 102,
            117, 110, 100, 101, 100, 34, 58, 48, 44, 34, 109, 101, 109, 111, 114, 121, 34, 58, 123,
            34, 119, 111, 114, 100, 115, 95, 110, 117, 109, 34, 58, 48, 44, 34, 101, 120, 112, 97,
            110, 115, 105, 111, 110, 95, 99, 111, 115, 116, 34, 58, 48, 125, 125, 125, 44, 34, 114,
            117, 110, 116, 105, 109, 101, 95, 102, 108, 97, 103, 34, 58, 123, 34, 105, 115, 95,
            115, 116, 97, 116, 105, 99, 34, 58, 102, 97, 108, 115, 101, 44, 34, 105, 115, 95, 101,
            111, 102, 95, 105, 110, 105, 116, 34, 58, 102, 97, 108, 115, 101, 44, 34, 105, 115, 95,
            101, 111, 102, 34, 58, 102, 97, 108, 115, 101, 44, 34, 115, 112, 101, 99, 95, 105, 100,
            34, 58, 34, 80, 82, 65, 71, 85, 69, 34, 125, 44, 34, 101, 120, 116, 101, 110, 100, 34,
            58, 110, 117, 108, 108, 125,
        ];

        let input = deserialize_input(serial_input).unwrap();

        assert_eq!(raw_interpreter, input.0);

        let mut context = input.1;
        let mut interpreter: Interpreter = serde_json::from_slice(&input.0).unwrap();

        let out = interpreter.run_plain(&instruction_table(), &mut context);

        let output = Output {
            context,
            interpreter,
            out,
        };

        println!("This is the out: {:?}", output.out);
    }
}

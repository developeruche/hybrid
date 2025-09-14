use reth::revm::{
    context::{BlockEnv, CfgEnv, JournalTr, TxEnv},
    context_interface::context::ContextError,
    db::EmptyDB,
    interpreter::{interpreter::EthInterpreter, Interpreter, InterpreterAction},
    Context, Database, Journal,
};
use serde::{Deserialize, Serialize};

pub fn serialize_input(interpreter: &Interpreter, block: &BlockEnv, tx: &TxEnv) -> Vec<u8> {
    let s_interpreter =
        bincode::serde::encode_to_vec(interpreter, bincode::config::legacy()).unwrap();
    let s_block = bincode::serde::encode_to_vec(block, bincode::config::legacy()).unwrap();
    let s_tx = bincode::serde::encode_to_vec(tx, bincode::config::legacy()).unwrap();

    let si_len = s_interpreter.len();
    let sb_len = s_block.len();
    let st_len = s_tx.len();

    let mut serialized = Vec::with_capacity(si_len + sb_len + st_len + 24);

    serialized.extend((si_len as u64).to_le_bytes());
    serialized.extend((sb_len as u64).to_le_bytes());
    serialized.extend((st_len as u64).to_le_bytes());

    serialized.extend(s_interpreter);
    serialized.extend(s_block);
    serialized.extend(s_tx);

    serialized
}

pub fn deserialize_input(data: &[u8]) -> (Interpreter, BlockEnv, TxEnv) {
    // Check minimum length for headers (16 bytes for two u64 lengths)
    if data.len() < 24 {
        panic!("Data too short for headers");
    }

    // Read the lengths from the first 16 bytes
    let si_len = u64::from_le_bytes(data[0..8].try_into().unwrap()) as usize;
    let sb_len = u64::from_le_bytes(data[8..16].try_into().unwrap()) as usize;
    let st_len = u64::from_le_bytes(data[16..24].try_into().unwrap()) as usize;

    // Check total length
    let expected_len = si_len + sb_len + st_len + 24;
    if data.len() != expected_len {
        panic!(
            "Data length mismatch: expected {}, got {}",
            expected_len,
            data.len()
        );
    }

    // Extract the interpreter bytes
    let interpreter_bytes = &data[24..24 + si_len];
    let interpreter: Interpreter =
        bincode::serde::decode_from_slice(interpreter_bytes, bincode::config::legacy())
            .unwrap()
            .0;

    // Extract the block bytes
    let block_bytes = &data[24 + si_len..24 + si_len + sb_len];
    let block: BlockEnv = bincode::serde::decode_from_slice(block_bytes, bincode::config::legacy())
        .unwrap()
        .0;

    // Extract the transaction bytes
    let tx_bytes = &data[24 + si_len + sb_len..24 + si_len + sb_len + st_len];
    let tx: TxEnv = bincode::serde::decode_from_slice(tx_bytes, bincode::config::legacy())
        .unwrap()
        .0;

    (interpreter, block, tx)
}

pub fn serialize_output(
    interpreter: &Interpreter,
    block: &BlockEnv,
    tx: &TxEnv,
    out: &InterpreterAction,
) -> Vec<u8> {
    let s_interpreter =
        bincode::serde::encode_to_vec(interpreter, bincode::config::legacy()).unwrap();
    let s_block = bincode::serde::encode_to_vec(block, bincode::config::legacy()).unwrap();
    let s_tx = bincode::serde::encode_to_vec(tx, bincode::config::legacy()).unwrap();
    let s_out = bincode::serde::encode_to_vec(out, bincode::config::legacy()).unwrap();

    let si_len = s_interpreter.len();
    let sb_len = s_block.len();
    let st_len = s_tx.len();
    let so_len = s_out.len();

    let mut serialized = Vec::with_capacity(si_len + sb_len + st_len + so_len + 32);

    serialized.extend((si_len as u64).to_le_bytes());
    serialized.extend((sb_len as u64).to_le_bytes());
    serialized.extend((st_len as u64).to_le_bytes());
    serialized.extend((so_len as u64).to_le_bytes());

    serialized.extend(s_interpreter);
    serialized.extend(s_block);
    serialized.extend(s_tx);
    serialized.extend(s_out);

    serialized
}

pub fn deserialize_output(serialized: &[u8]) -> (Interpreter, BlockEnv, TxEnv, InterpreterAction) {
    // Check minimum length for headers (32 bytes for four u64 lengths)
    if serialized.len() < 32 {
        panic!("Data too short for headers");
    }

    // Read the lengths from the first 32 bytes
    let si_len = u64::from_le_bytes(serialized[0..8].try_into().unwrap()) as usize;
    let sb_len = u64::from_le_bytes(serialized[8..16].try_into().unwrap()) as usize;
    let st_len = u64::from_le_bytes(serialized[16..24].try_into().unwrap()) as usize;
    let so_len = u64::from_le_bytes(serialized[24..32].try_into().unwrap()) as usize;

    // Check total length
    let expected_len = si_len + sb_len + st_len + so_len + 32;
    if serialized.len() != expected_len {
        panic!(
            "Data length mismatch: expected {}, got {}",
            expected_len,
            serialized.len()
        );
    }

    // Extract the interpreter bytes
    let interpreter_bytes = &serialized[32..32 + si_len];
    let interpreter: Interpreter =
        bincode::serde::decode_from_slice(interpreter_bytes, bincode::config::legacy())
            .unwrap()
            .0;

    // Extract the block bytes
    let block_bytes = &serialized[32 + si_len..32 + si_len + sb_len];
    let block: BlockEnv = bincode::serde::decode_from_slice(block_bytes, bincode::config::legacy())
        .unwrap()
        .0;

    // Extract the transaction bytes
    let tx_bytes = &serialized[32 + si_len + sb_len..32 + si_len + sb_len + st_len];
    let tx: TxEnv = bincode::serde::decode_from_slice(tx_bytes, bincode::config::legacy())
        .unwrap()
        .0;

    // Extract the output bytes
    let out_bytes =
        &serialized[32 + si_len + sb_len + st_len..32 + si_len + sb_len + st_len + so_len];
    let out: InterpreterAction =
        bincode::serde::decode_from_slice(out_bytes, bincode::config::legacy())
            .unwrap()
            .0;

    (interpreter, block, tx, out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reth::{
        primitives::TxType,
        revm::{
            context::{BlockEnv, CfgEnv, TxEnv}, db::EmptyDB, interpreter::{
                instruction_table, Gas, InstructionResult, Interpreter, InterpreterAction,
                InterpreterResult, Stack,
            }, primitives::{Address, Bytes, TxKind, B256, U256}, state::Bytecode, Journal
        },
        rpc::types::AccessList,
    };

    fn create_test_block() -> BlockEnv {
        BlockEnv {
            number: 100,
            beneficiary: Address::from([1u8; 20]),
            timestamp: 1234567890,
            gas_limit: 8000000,
            basefee: 1000000000,
            difficulty: U256::from(12345),
            prevrandao: Some(B256::from([2u8; 32])),
            blob_excess_gas_and_price: None,
        }
    }

    fn create_test_tx() -> TxEnv {
        TxEnv {
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
        }
    }

    fn create_test_interpreter() -> Interpreter {
        let bytecode = Bytecode::new_raw(Bytes::from(vec![0x60, 0x80, 0x60, 0x40]));
        let mut interpreter = Interpreter::default();
        interpreter = interpreter.with_bytecode(bytecode);
        interpreter
    }

    fn create_test_interpreter_action() -> InterpreterAction {
        InterpreterAction::Return {
            result: InterpreterResult {
                result: InstructionResult::Return,
                output: Bytes::from(vec![0x1, 0x2, 0x3, 0x4]),
                gas: Gas::new(21000),
            },
        }
    }

    #[test]
    fn test_input_round_trip() {
        // Create test data
        let interpreter = create_test_interpreter();
        let block = create_test_block();
        let tx = create_test_tx();

        // Serialize
        let serialized = serialize_input(&interpreter, &block, &tx);

        // Deserialize
        let (deserialized_interpreter, deserialized_block, deserialized_tx) =
            deserialize_input(&serialized);

        // Compare interpreter bytecode (since Interpreter doesn't implement PartialEq)
        assert_eq!(interpreter.bytecode.bytecode(), deserialized_interpreter.bytecode.bytecode());

        // Compare block
        assert_eq!(block.number, deserialized_block.number);
        assert_eq!(block.beneficiary, deserialized_block.beneficiary);
        assert_eq!(block.timestamp, deserialized_block.timestamp);
        assert_eq!(block.gas_limit, deserialized_block.gas_limit);
        assert_eq!(block.basefee, deserialized_block.basefee);
        assert_eq!(block.difficulty, deserialized_block.difficulty);
        assert_eq!(block.prevrandao, deserialized_block.prevrandao);

        // Compare transaction
        assert_eq!(tx.caller, deserialized_tx.caller);
        assert_eq!(tx.gas_limit, deserialized_tx.gas_limit);
        assert_eq!(tx.gas_price, deserialized_tx.gas_price);
        assert_eq!(tx.value, deserialized_tx.value);
        assert_eq!(tx.data, deserialized_tx.data);
        assert_eq!(tx.nonce, deserialized_tx.nonce);
        assert_eq!(tx.chain_id, deserialized_tx.chain_id);
        assert_eq!(tx.kind, deserialized_tx.kind);
    }

    #[test]
    fn test_output_round_trip() {
        // Create test data
        let interpreter = create_test_interpreter();
        let block = create_test_block();
        let tx = create_test_tx();
        let action = create_test_interpreter_action();

        // Serialize
        let serialized = serialize_output(&interpreter, &block, &tx, &action);

        // Deserialize
        let (deserialized_interpreter, deserialized_block, deserialized_tx, deserialized_action) =
            deserialize_output(&serialized);

        // Compare interpreter bytecode
        assert_eq!(interpreter.bytecode.bytecode(), deserialized_interpreter.bytecode.bytecode());

        // Compare block
        assert_eq!(block.number, deserialized_block.number);
        assert_eq!(block.beneficiary, deserialized_block.beneficiary);
        assert_eq!(block.timestamp, deserialized_block.timestamp);
        assert_eq!(block.gas_limit, deserialized_block.gas_limit);
        assert_eq!(block.basefee, deserialized_block.basefee);
        assert_eq!(block.difficulty, deserialized_block.difficulty);
        assert_eq!(block.prevrandao, deserialized_block.prevrandao);

        // Compare transaction
        assert_eq!(tx.caller, deserialized_tx.caller);
        assert_eq!(tx.gas_limit, deserialized_tx.gas_limit);
        assert_eq!(tx.gas_price, deserialized_tx.gas_price);
        assert_eq!(tx.value, deserialized_tx.value);
        assert_eq!(tx.data, deserialized_tx.data);
        assert_eq!(tx.nonce, deserialized_tx.nonce);
        assert_eq!(tx.chain_id, deserialized_tx.chain_id);
        assert_eq!(tx.kind, deserialized_tx.kind);

        // Compare action (match on the discriminant)
        match (&action, &deserialized_action) {
            (
                InterpreterAction::Return { result: r1 },
                InterpreterAction::Return { result: r2 },
            ) => {
                assert_eq!(r1.result, r2.result);
                assert_eq!(r1.output, r2.output);
                assert_eq!(r1.gas.limit(), r2.gas.limit());
            }
            _ => panic!("Action types don't match"),
        }
    }

    #[test]
    fn test_input_serialization_error_handling() {
        // Test with data too short
        let short_data = vec![1, 2, 3];
        let result = std::panic::catch_unwind(|| {
            deserialize_input(&short_data);
        });
        assert!(result.is_err());

        // Test with incorrect length
        let mut incorrect_data = vec![0u8; 32]; // Headers claiming certain lengths
        incorrect_data[0] = 100; // Claim 100 bytes for interpreter
        let result = std::panic::catch_unwind(|| {
            deserialize_input(&incorrect_data);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_output_serialization_error_handling() {
        // Test with data too short
        let short_data = vec![1, 2, 3];
        let result = std::panic::catch_unwind(|| {
            deserialize_output(&short_data);
        });
        assert!(result.is_err());

        // Test with incorrect length
        let mut incorrect_data = vec![0u8; 40]; // Headers claiming certain lengths
        incorrect_data[0] = 100; // Claim 100 bytes for interpreter
        let result = std::panic::catch_unwind(|| {
            deserialize_output(&incorrect_data);
        });
        assert!(result.is_err());
    }

    // #[test]
    // fn test_interaglly() {
    //     let serial_input: &[u8] = &[195, 2, 0, 0, 0, 0, 0, 0, 142, 2, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 136, 243, 0, 192, 112, 39, 52, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 32, 0, 0, 0, 0, 0, 0, 0, 154, 128, 77, 211, 231, 224, 24, 151, 92, 16, 128, 79, 29, 191, 60, 164, 211, 181, 169, 42, 69, 88, 244, 226, 128, 172, 36, 155, 215, 149, 147, 93, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 243, 159, 214, 229, 26, 173, 136, 246, 244, 206, 106, 184, 130, 114, 121, 207, 255, 185, 34, 102, 0, 0, 0, 0, 0, 136, 243, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 248, 0, 0, 0, 0, 0, 0, 0, 96, 128, 128, 96, 64, 82, 52, 96, 19, 87, 96, 223, 144, 129, 96, 25, 130, 57, 243, 91, 96, 0, 128, 253, 254, 96, 128, 128, 96, 64, 82, 96, 4, 54, 16, 21, 96, 18, 87, 96, 0, 128, 253, 91, 96, 0, 53, 96, 224, 28, 144, 129, 99, 63, 181, 193, 203, 20, 96, 146, 87, 129, 99, 131, 129, 245, 138, 20, 96, 121, 87, 80, 99, 208, 157, 224, 138, 20, 96, 60, 87, 96, 0, 128, 253, 91, 52, 96, 116, 87, 96, 0, 54, 96, 3, 25, 1, 18, 96, 116, 87, 96, 0, 84, 96, 0, 25, 129, 20, 96, 94, 87, 96, 1, 1, 96, 0, 85, 0, 91, 99, 78, 72, 123, 113, 96, 224, 27, 96, 0, 82, 96, 17, 96, 4, 82, 96, 36, 96, 0, 253, 91, 96, 0, 128, 253, 91, 52, 96, 116, 87, 96, 0, 54, 96, 3, 25, 1, 18, 96, 116, 87, 96, 32, 144, 96, 0, 84, 129, 82, 243, 91, 52, 96, 116, 87, 96, 32, 54, 96, 3, 25, 1, 18, 96, 116, 87, 96, 4, 53, 96, 0, 85, 0, 254, 162, 100, 105, 112, 102, 115, 88, 34, 18, 32, 233, 120, 39, 8, 131, 183, 186, 237, 16, 129, 12, 64, 121, 201, 65, 81, 46, 147, 167, 186, 28, 209, 16, 140, 120, 29, 75, 199, 56, 217, 9, 5, 100, 115, 111, 108, 99, 67, 0, 8, 26, 0, 51, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 18, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 17, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 18, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 25, 1, 0, 0, 0, 0, 0, 0, 96, 128, 128, 96, 64, 82, 52, 96, 19, 87, 96, 223, 144, 129, 96, 25, 130, 57, 243, 91, 96, 0, 128, 253, 254, 96, 128, 128, 96, 64, 82, 96, 4, 54, 16, 21, 96, 18, 87, 96, 0, 128, 253, 91, 96, 0, 53, 96, 224, 28, 144, 129, 99, 63, 181, 193, 203, 20, 96, 146, 87, 129, 99, 131, 129, 245, 138, 20, 96, 121, 87, 80, 99, 208, 157, 224, 138, 20, 96, 60, 87, 96, 0, 128, 253, 91, 52, 96, 116, 87, 96, 0, 54, 96, 3, 25, 1, 18, 96, 116, 87, 96, 0, 84, 96, 0, 25, 129, 20, 96, 94, 87, 96, 1, 1, 96, 0, 85, 0, 91, 99, 78, 72, 123, 113, 96, 224, 27, 96, 0, 82, 96, 17, 96, 4, 82, 96, 36, 96, 0, 253, 91, 96, 0, 128, 253, 91, 52, 96, 116, 87, 96, 0, 54, 96, 3, 25, 1, 18, 96, 116, 87, 96, 32, 144, 96, 0, 84, 129, 82, 243, 91, 52, 96, 116, 87, 96, 32, 54, 96, 3, 25, 1, 18, 96, 116, 87, 96, 4, 53, 96, 0, 85, 0, 254, 162, 100, 105, 112, 102, 115, 88, 34, 18, 32, 233, 120, 39, 8, 131, 183, 186, 237, 16, 129, 12, 64, 121, 201, 65, 81, 46, 147, 167, 186, 28, 209, 16, 140, 120, 29, 75, 199, 56, 217, 9, 5, 100, 115, 111, 108, 99, 67, 0, 8, 26, 0, 51, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 248, 0, 0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 98, 105, 116, 118, 101, 99, 58, 58, 111, 114, 100, 101, 114, 58, 58, 76, 115, 98, 48, 8, 0, 25, 1, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 8, 0, 0, 0, 0, 32, 0, 0, 0, 128, 0, 0, 32, 4, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 95, 189, 178, 49, 86, 120, 175, 236, 179, 103, 240, 50, 217, 63, 100, 47, 100, 24, 10, 163, 20, 0, 0, 0, 0, 0, 0, 0, 243, 159, 214, 229, 26, 173, 136, 246, 244, 206, 106, 184, 130, 114, 121, 207, 255, 185, 34, 102, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 64, 34, 255, 255, 255, 135, 243, 0, 64, 34, 255, 255, 255, 135, 243, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 18, 0, 0, 0];

    //     let raw_interpreter: &[u8] = &[0, 0, 0, 0, 25, 1, 0, 0, 0, 0, 0, 0, 96, 128, 128, 96, 64, 82, 52, 96, 19, 87, 96, 223, 144, 129, 96, 25, 130, 57, 243, 91, 96, 0, 128, 253, 254, 96, 128, 128, 96, 64, 82, 96, 4, 54, 16, 21, 96, 18, 87, 96, 0, 128, 253, 91, 96, 0, 53, 96, 224, 28, 144, 129, 99, 63, 181, 193, 203, 20, 96, 146, 87, 129, 99, 131, 129, 245, 138, 20, 96, 121, 87, 80, 99, 208, 157, 224, 138, 20, 96, 60, 87, 96, 0, 128, 253, 91, 52, 96, 116, 87, 96, 0, 54, 96, 3, 25, 1, 18, 96, 116, 87, 96, 0, 84, 96, 0, 25, 129, 20, 96, 94, 87, 96, 1, 1, 96, 0, 85, 0, 91, 99, 78, 72, 123, 113, 96, 224, 27, 96, 0, 82, 96, 17, 96, 4, 82, 96, 36, 96, 0, 253, 91, 96, 0, 128, 253, 91, 52, 96, 116, 87, 96, 0, 54, 96, 3, 25, 1, 18, 96, 116, 87, 96, 32, 144, 96, 0, 84, 129, 82, 243, 91, 52, 96, 116, 87, 96, 32, 54, 96, 3, 25, 1, 18, 96, 116, 87, 96, 4, 53, 96, 0, 85, 0, 254, 162, 100, 105, 112, 102, 115, 88, 34, 18, 32, 233, 120, 39, 8, 131, 183, 186, 237, 16, 129, 12, 64, 121, 201, 65, 81, 46, 147, 167, 186, 28, 209, 16, 140, 120, 29, 75, 199, 56, 217, 9, 5, 100, 115, 111, 108, 99, 67, 0, 8, 26, 0, 51, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 248, 0, 0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 98, 105, 116, 118, 101, 99, 58, 58, 111, 114, 100, 101, 114, 58, 58, 76, 115, 98, 48, 8, 0, 25, 1, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 8, 0, 0, 0, 0, 32, 0, 0, 0, 128, 0, 0, 32, 4, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 95, 189, 178, 49, 86, 120, 175, 236, 179, 103, 240, 50, 217, 63, 100, 47, 100, 24, 10, 163, 20, 0, 0, 0, 0, 0, 0, 0, 243, 159, 214, 229, 26, 173, 136, 246, 244, 206, 106, 184, 130, 114, 121, 207, 255, 185, 34, 102, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 64, 34, 255, 255, 255, 135, 243, 0, 64, 34, 255, 255, 255, 135, 243, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 18, 0, 0, 0];

    //     let input = deserialize_input(serial_input).unwrap();

    //     assert_eq!(raw_interpreter, input.0);

    //     let mut context = input.1;
    //     let mut interpreter: Interpreter = bincode::serde::decode_from_slice(&input.0, bincode::config::legacy()).unwrap().0;

    //     let out = interpreter.run_plain(&instruction_table(), &mut context);

    //     let output = Output {
    //         context,
    //         interpreter,
    //         out,
    //     };

    //     println!("This is the out: {:?}", output.out);
    // }
}

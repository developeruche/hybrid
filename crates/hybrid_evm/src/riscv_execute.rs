//! This module contains the implementation of the Interpreter for the RISC-V architecture.
//! NOTICE: Some code in this module was copied and modified from the r55 implemenation.
//! r55 github: http://github.com/r55-eth/r55/

use eth_riscv_interpreter::setup_from_elf;
use reth::revm::{
    context::{ContextTr, JournalTr, result::FromStringError},
    handler::{
        ContextTrDbError, EthFrame, EvmTr, FrameInitOrResult, PrecompileProvider,
        instructions::InstructionProvider,
    },
    interpreter::{FrameInput, InterpreterResult, interpreter::EthInterpreter},
    primitives::alloy_primitives::U32,
};

use crate::execution::execute_riscv;

pub fn run_riscv_interpreter<EVM, ERROR>(
    bytecode: &[u8],
    frame: &mut EthFrame<EVM, ERROR, <EVM::Instructions as InstructionProvider>::InterpreterTypes>,
    evm: &mut EVM,
) -> Result<FrameInitOrResult<EthFrame<EVM, ERROR, EthInterpreter>>, ERROR>
where
    EVM: EvmTr<
            Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
            Instructions: InstructionProvider<
                Context = EVM::Context,
                InterpreterTypes = EthInterpreter,
            >,
        >,
    ERROR: From<ContextTrDbError<EVM::Context>> + FromStringError,
{
    let mut last_created_address = None;

    let (code, calldata) = match &frame.input {
        FrameInput::Call(call_inputs) => (bytecode, call_inputs.input.0.as_ref()),
        FrameInput::Create(c) => {
            let account = evm.ctx().journal().load_account(c.caller).unwrap();
            last_created_address = Some(c.created_address(account.info.nonce - 1));

            let (code_size, init_code) = bytecode.split_at(4);

            let Some((_, bytecode)) = init_code.split_first() else {
                return Err(ERROR::from_string(
                    "This contract is not valid for RISC-V".to_string(),
                ));
            };

            let code_size = U32::from_be_slice(code_size).to::<usize>() - 1; // deduct control byte `0xFF`
            let end_of_args = init_code.len() - 34; // deduct control byte + ignore empty (32 byte) word appended by revm

            (&bytecode[..code_size], &bytecode[code_size..end_of_args])
        }
        FrameInput::EOFCreate(_eofcreate_inputs) => {
            todo!("No EOF standard for RISC-V at the moment")
        }
    };

    let mut emulator = match setup_from_elf(code, calldata) {
        Ok(emulator) => emulator,
        Err(err) => {
            return Err(ERROR::from_string(
                "Error occurred setting up emulator: ".to_string() + &err.to_string(),
            ));
        }
    };

    let interpreter_action = execute_riscv(
        &mut emulator,
        &mut frame.interpreter,
        evm,
        &last_created_address,
    )
    .map_err(ERROR::from_string)?;

    frame.process_next_action(evm, interpreter_action)
}

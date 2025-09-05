//! This file holds modifications to the EVM frame to accomdate the Hybrid logic.
use reth::revm::{
    context::{
        result::{EVMError, InvalidTransaction},
        ContextTr, JournalOutput, JournalTr,
    },
    handler::{
        instructions::InstructionProvider, EthFrame, EvmTr, Frame, FrameInitOrResult,
        PrecompileProvider,
    },
    interpreter::{interpreter::EthInterpreter, InterpreterResult},
    Database,
};

use crate::hybrid_execute::run_hybrid_interpreter;

pub fn hybrid_frame_call<EVM>(
    frame: &mut EthFrame<
        EVM,
        EVMError<<<EVM::Context as ContextTr>::Db as Database>::Error, InvalidTransaction>,
        <EVM::Instructions as InstructionProvider>::InterpreterTypes,
    >,
    evm: &mut EVM,
) -> Result<
    FrameInitOrResult<
        EthFrame<
            EVM,
            EVMError<<<EVM::Context as ContextTr>::Db as Database>::Error, InvalidTransaction>,
            <EVM::Instructions as InstructionProvider>::InterpreterTypes,
        >,
    >,
    EVMError<<<EVM::Context as ContextTr>::Db as Database>::Error, InvalidTransaction>,
>
where
    EVM: EvmTr<
        Context: ContextTr<Journal: JournalTr<FinalOutput = JournalOutput>>,
        Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
        >,
    >,
{
    let bytecode_clone = frame.interpreter.bytecode.clone();
    let split_result = bytecode_clone.bytecode().split_first();

    if split_result.is_some() && *split_result.unwrap().0 == 0xFF {
        let (_, bytecode) = split_result.unwrap();

        return run_hybrid_interpreter::<
            EVM,
            EVMError<<<EVM::Context as ContextTr>::Db as Database>::Error, InvalidTransaction>,
        >(bytecode, frame, evm);
    } else {
        return Frame::run(frame, evm);
    }
}

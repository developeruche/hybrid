//! This file holds modifications to the EVM frame to accomdate the Hybrid logic.
use reth::revm::{
    Database,
    context::{
        ContextTr, JournalOutput, JournalTr,
        result::{EVMError, InvalidTransaction},
    },
    handler::{
        EthFrame, EvmTr, Frame, FrameInitOrResult, PrecompileProvider,
        instructions::InstructionProvider,
    },
    interpreter::{InterpreterResult, interpreter::EthInterpreter},
};

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
    let split_result = frame.interpreter.bytecode.bytecode().split_first();
    if split_result.is_some() && *split_result.unwrap().0 == 0xFF {
        let (_, _bytecode) = split_result.unwrap();
        // Rest of the code would go here

        todo!()
    } else {
        return Frame::run(frame, evm);
    }
}

use reth::revm::{
    context::{
        result::{EVMError, HaltReason, InvalidTransaction},
        JournalOutput,
    },
    context_interface::{ContextTr, JournalTr},
    handler::{
        instructions::InstructionProvider, EthFrame, EvmTr, FrameResult, Handler,
        PrecompileProvider,
    },
    inspector::{Inspector, InspectorEvmTr, InspectorHandler},
    interpreter::{interpreter::EthInterpreter, InterpreterResult},
    Database,
};

pub struct HybridHandler<EVM> {
    pub _phantom: core::marker::PhantomData<EVM>,
}

impl<EVM> Default for HybridHandler<EVM> {
    fn default() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<EVM> Handler for HybridHandler<EVM>
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
    type Evm = EVM;
    type Error = EVMError<<<EVM::Context as ContextTr>::Db as Database>::Error, InvalidTransaction>;
    type Frame = EthFrame<
        EVM,
        EVMError<<<EVM::Context as ContextTr>::Db as Database>::Error, InvalidTransaction>,
        <EVM::Instructions as InstructionProvider>::InterpreterTypes,
    >;
    type HaltReason = HaltReason;

    // would be making modification to the handler to also accomadate the RISC V exec logic
}

impl<EVM> InspectorHandler for HybridHandler<EVM>
where
    EVM: InspectorEvmTr<
        Inspector: Inspector<<<Self as Handler>::Evm as EvmTr>::Context, EthInterpreter>,
        Context: ContextTr<Journal: JournalTr<FinalOutput = JournalOutput>>,
        Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
        >,
    >,
{
    type IT = EthInterpreter;
}

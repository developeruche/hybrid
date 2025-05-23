use reth::revm::{
    Database,
    context::{
        JournalOutput,
        result::{EVMError, HaltReason, InvalidTransaction},
    },
    context_interface::{ContextTr, JournalTr},
    handler::{EthFrame, EvmTr, Handler, PrecompileProvider, instructions::InstructionProvider},
    inspector::{Inspector, InspectorEvmTr, InspectorHandler},
    interpreter::{InterpreterResult, interpreter::EthInterpreter},
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

    // Here are this things I need to apply
    // Looks like it is just the exection that would be update...
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

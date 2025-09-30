use reth::revm::{
    context::{
        result::{EVMError, HaltReason, InvalidTransaction},
        JournalOutput,
    },
    context_interface::{ContextTr, JournalTr},
    handler::{
        instructions::InstructionProvider, EthFrame, EvmTr, Frame, FrameInitOrResult,
        FrameOrResult, FrameResult, Handler, ItemOrResult, PrecompileProvider,
    },
    inspector::{Inspector, InspectorEvmTr, InspectorHandler},
    interpreter::{interpreter::EthInterpreter, InterpreterResult},
    Database,
};

use crate::frame::hybrid_frame_call;

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

    #[inline]
    fn first_frame_init(
        &mut self,
        evm: &mut Self::Evm,
        frame_input: <Self::Frame as Frame>::FrameInit,
    ) -> Result<FrameOrResult<Self::Frame>, Self::Error> {
        let out = Self::Frame::init_first(evm, frame_input);
        out
    }

    #[inline]
    fn frame_call(
        &mut self,
        frame: &mut Self::Frame,
        evm: &mut Self::Evm,
    ) -> Result<FrameInitOrResult<Self::Frame>, Self::Error> {
        hybrid_frame_call(frame, evm)
    }

    #[inline]
    fn run_exec_loop(
        &mut self,
        evm: &mut Self::Evm,
        frame: Self::Frame,
    ) -> Result<FrameResult, Self::Error> {
        let mut frame_stack: Vec<Self::Frame> = vec![frame];
        loop {
            let frame = frame_stack.last_mut().unwrap();
            let call_or_result = self.frame_call(frame, evm)?;

            let result = match call_or_result {
                ItemOrResult::Item(init) => {
                    match self.frame_init(frame, evm, init)? {
                        ItemOrResult::Item(new_frame) => {
                            frame_stack.push(new_frame);
                            continue;
                        }
                        // Do not pop the frame since no new frame was created
                        ItemOrResult::Result(result) => result,
                    }
                }
                ItemOrResult::Result(result) => {
                    // Remove the frame that returned the result
                    frame_stack.pop();
                    result
                }
            };

            let Some(frame) = frame_stack.last_mut() else {
                return Ok(result);
            };
            self.frame_return_result(frame, evm, result)?;
        }
    }
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

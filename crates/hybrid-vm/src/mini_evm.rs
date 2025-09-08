//! MiniEVM is a minimalistic Ethereum Virtual Machine design to run smart contracts on a
//! RISC-V machine as an on-the-fly EVM interpreter.

//! This module contains the implementation of the Interpreter for the RISC-V architecture.
//! NOTICE: Some code in this module was copied and modified from the r55 implemenation.
//! r55 github: http://github.com/r55-eth/r55/

use reth::revm::{
    context::{result::FromStringError, ContextTr, JournalTr},
    handler::{
        instructions::InstructionProvider, ContextTrDbError, EthFrame, EvmTr, FrameInitOrResult,
        PrecompileProvider,
    },
    interpreter::{interpreter::EthInterpreter, FrameInput, InterpreterResult},
    primitives::alloy_primitives::U32,
};

use crate::{execution::execute_riscv, setup::setup_from_elf};

pub fn run_mini_evm_interpreter<EVM, ERROR>(
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
    //

    // frame.process_next_action(evm, interpreter_action)

    todo!()
}

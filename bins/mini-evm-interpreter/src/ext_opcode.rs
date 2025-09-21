use ext_revm::interpreter::gas::warm_cold_cost;
/// This module contains the implementation of the EVM opcodes that need to interact with the host.
use ext_revm::{
    interpreter::{
        gas,
        instructions::utility::IntoAddress,
        interpreter_types::{LoopControl, RuntimeFlag, StackTr},
        popn_top, Host, InstructionResult, Interpreter, InterpreterTypes,
    },
    primitives::hardfork::SpecId::*,
};

pub fn balance<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    popn_top!([], top, interpreter);
    let address = top.into_address();
    let Some(balance) = host.balance(address) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    let spec_id = interpreter.runtime_flag.spec_id();
    gas!(
        interpreter,
        if spec_id.is_enabled_in(BERLIN) {
            warm_cold_cost(balance.is_cold)
        } else if spec_id.is_enabled_in(ISTANBUL) {
            // EIP-1884: Repricing for trie-size-dependent opcodes
            700
        } else if spec_id.is_enabled_in(TANGERINE) {
            400
        } else {
            20
        }
    );
    *top = balance.data;
}

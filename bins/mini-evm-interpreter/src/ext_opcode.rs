use core::cmp::min;

use ext_revm::interpreter::gas::warm_cold_cost;
use ext_revm::interpreter::{
    as_usize_or_fail, as_usize_saturated, gas_or_fail, popn, resize_memory,
};
use ext_revm::primitives::U256;
/// This module contains the implementation of the EVM opcodes that need to interact with the host.
use ext_revm::{
    interpreter::{
        gas,
        instructions::utility::IntoAddress,
        interpreter_types::{LoopControl, MemoryTr, RuntimeFlag, StackTr},
        popn_top, Host, InstructionResult, Interpreter, InterpreterTypes,
    },
    primitives::hardfork::SpecId::*,
};

use crate::ext_syscalls::{host_balance, host_load_account_code};

pub fn balance<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    popn_top!([], top, interpreter);
    let address = top.into_address();
    let Some(balance) = host_balance(address) else {
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

pub fn extcodesize<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    popn_top!([], top, interpreter);
    let address = top.into_address();
    let Some(code) = host_load_account_code(address) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    let spec_id = interpreter.runtime_flag.spec_id();
    if spec_id.is_enabled_in(BERLIN) {
        gas!(interpreter, warm_cold_cost(code.is_cold));
    } else if spec_id.is_enabled_in(TANGERINE) {
        gas!(interpreter, 700);
    } else {
        gas!(interpreter, 20);
    }

    *top = U256::from(code.len());
}

pub fn extcodecopy<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    popn!([address, memory_offset, code_offset, len_u256], interpreter);
    let address = address.into_address();
    let Some(code) = host_load_account_code(address) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    let len = as_usize_or_fail!(interpreter, len_u256);
    gas_or_fail!(
        interpreter,
        gas::extcodecopy_cost(interpreter.runtime_flag.spec_id(), len, code.is_cold)
    );
    if len == 0 {
        return;
    }
    let memory_offset = as_usize_or_fail!(interpreter, memory_offset);
    let code_offset = min(as_usize_saturated!(code_offset), code.len());
    resize_memory!(interpreter, memory_offset, len);

    // Note: This can't panic because we resized memory to fit.
    interpreter
        .memory
        .set_data(memory_offset, code_offset, len, &code);
}

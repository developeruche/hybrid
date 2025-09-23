use alloc::boxed::Box;
use core::cmp::{max, min};
use ext_revm::interpreter::gas::{warm_cold_cost, CALL_STIPEND, MIN_CALLEE_GAS};
use ext_revm::interpreter::instructions::contract::{
    calc_call_gas, extcall_input, get_memory_input_and_out_ranges, pop_extcall_target_address,
};
use ext_revm::interpreter::instructions::utility::IntoU256;
use ext_revm::interpreter::interpreter_types::{InputsTr, ReturnData};
use ext_revm::interpreter::{
    as_u64_saturated, as_usize_or_fail, as_usize_saturated, gas_or_fail, popn, push, require_eof,
    require_non_staticcall, resize_memory, CallInputs, CallScheme, CallValue, FrameInput,
    InterpreterAction,
};
use ext_revm::primitives::{Address, B256, BLOCK_HASH_HISTORY, U256};

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

use crate::ext_syscalls::{
    host_balance, host_block_hash, host_block_number, host_load_account_code,
    host_load_account_code_hash, host_load_account_delegated, host_selfdestruct, host_sload,
    host_sstore, host_tload, host_tstore,
};

pub fn balance<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
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
    _host: &mut H,
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
    _host: &mut H,
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

pub fn extcodehash<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    popn_top!([], top, interpreter);
    let address = top.into_address();
    let Some(code_hash) = host_load_account_code_hash(address) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    let spec_id = interpreter.runtime_flag.spec_id();
    if spec_id.is_enabled_in(BERLIN) {
        gas!(interpreter, warm_cold_cost(code_hash.is_cold));
    } else if spec_id.is_enabled_in(ISTANBUL) {
        gas!(interpreter, 700);
    } else {
        gas!(interpreter, 400);
    }
    *top = code_hash.into_u256();
}

pub fn blockhash<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::BLOCKHASH);
    popn_top!([], number, interpreter);

    let requested_number = as_u64_saturated!(number);

    let block_number = host_block_number();

    let Some(diff) = block_number.checked_sub(requested_number) else {
        *number = U256::ZERO;
        return;
    };

    // blockhash should push zero if number is same as current block number.
    if diff == 0 {
        *number = U256::ZERO;
        return;
    }

    *number = if diff <= BLOCK_HASH_HISTORY {
        let Some(hash) = host_block_hash(requested_number) else {
            interpreter
                .control
                .set_instruction_result(InstructionResult::FatalExternalError);
            return;
        };
        U256::from_be_bytes(hash.0)
    } else {
        U256::ZERO
    }
}

pub fn selfbalance<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::LOW);

    let Some(balance) = host_balance(interpreter.input.target_address()) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    push!(interpreter, balance.data);
}

pub fn sload<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    popn_top!([], index, interpreter);

    let Some(value) = host_sload(interpreter.input.target_address(), *index) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    gas!(
        interpreter,
        gas::sload_cost(interpreter.runtime_flag.spec_id(), value.is_cold)
    );
    *index = value.data;
}

pub fn sstore<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    require_non_staticcall!(interpreter);

    popn!([index, value], interpreter);

    let Some(state_load) = host_sstore(interpreter.input.target_address(), index, value) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    // EIP-1706 Disable SSTORE with gasleft lower than call stipend
    if interpreter.runtime_flag.spec_id().is_enabled_in(ISTANBUL)
        && interpreter.control.gas().remaining() <= CALL_STIPEND
    {
        interpreter
            .control
            .set_instruction_result(InstructionResult::ReentrancySentryOOG);
        return;
    }
    gas!(
        interpreter,
        gas::sstore_cost(
            interpreter.runtime_flag.spec_id(),
            &state_load.data,
            state_load.is_cold
        )
    );

    interpreter
        .control
        .gas_mut()
        .record_refund(gas::sstore_refund(
            interpreter.runtime_flag.spec_id(),
            &state_load.data,
        ));
}

pub fn tload<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::WARM_STORAGE_READ_COST);

    popn_top!([], index, interpreter);

    *index = host_tload(interpreter.input.target_address(), *index);
}

pub fn tstore<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    require_non_staticcall!(interpreter);
    gas!(interpreter, gas::WARM_STORAGE_READ_COST);

    popn!([index, value], interpreter);

    host_tstore(interpreter.input.target_address(), index, value);
}

pub fn call<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    popn!([local_gas_limit, to, value], interpreter);
    let to = to.into_address();
    // Max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    let has_transfer = !value.is_zero();
    if interpreter.runtime_flag.is_static() && has_transfer {
        interpreter
            .control
            .set_instruction_result(InstructionResult::CallNotAllowedInsideStatic);
        return;
    }

    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(interpreter) else {
        return;
    };

    let Some(account_load) = host_load_account_delegated(to) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    let Some(mut gas_limit) =
        calc_call_gas(interpreter, account_load, has_transfer, local_gas_limit)
    else {
        return;
    };

    gas!(interpreter, gas_limit);

    // Add call stipend if there is value to be transferred.
    if has_transfer {
        gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
    }

    // Call host to interact with target contract
    interpreter.control.set_next_action(
        InterpreterAction::NewFrame(FrameInput::Call(Box::new(CallInputs {
            input,
            gas_limit,
            target_address: to,
            caller: interpreter.input.target_address(),
            bytecode_address: to,
            value: CallValue::Transfer(value),
            scheme: CallScheme::Call,
            is_static: interpreter.runtime_flag.is_static(),
            is_eof: false,
            return_memory_offset,
        }))),
        InstructionResult::CallOrCreate,
    );
}

pub fn call_code<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    popn!([local_gas_limit, to, value], interpreter);
    let to = Address::from_word(B256::from(to));
    // Max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    //pop!(interpreter, value);
    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(interpreter) else {
        return;
    };

    let Some(mut load) = host_load_account_delegated(to) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    // Set `is_empty` to false as we are not creating this account.
    load.is_empty = false;
    let Some(mut gas_limit) = calc_call_gas(interpreter, load, !value.is_zero(), local_gas_limit)
    else {
        return;
    };

    gas!(interpreter, gas_limit);

    // Add call stipend if there is value to be transferred.
    if !value.is_zero() {
        gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
    }

    // Call host to interact with target contract
    interpreter.control.set_next_action(
        InterpreterAction::NewFrame(FrameInput::Call(Box::new(CallInputs {
            input,
            gas_limit,
            target_address: interpreter.input.target_address(),
            caller: interpreter.input.target_address(),
            bytecode_address: to,
            value: CallValue::Transfer(value),
            scheme: CallScheme::CallCode,
            is_static: interpreter.runtime_flag.is_static(),
            is_eof: false,
            return_memory_offset,
        }))),
        InstructionResult::CallOrCreate,
    );
}

pub fn delegate_call<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    popn!([local_gas_limit, to], interpreter);
    let to = Address::from_word(B256::from(to));
    // Max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(interpreter) else {
        return;
    };

    let Some(mut load) = host_load_account_delegated(to) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    // Set is_empty to false as we are not creating this account.
    load.is_empty = false;
    let Some(gas_limit) = calc_call_gas(interpreter, load, false, local_gas_limit) else {
        return;
    };

    gas!(interpreter, gas_limit);

    // Call host to interact with target contract
    interpreter.control.set_next_action(
        InterpreterAction::NewFrame(FrameInput::Call(Box::new(CallInputs {
            input,
            gas_limit,
            target_address: interpreter.input.target_address(),
            caller: interpreter.input.caller_address(),
            bytecode_address: to,
            value: CallValue::Apparent(interpreter.input.call_value()),
            scheme: CallScheme::DelegateCall,
            is_static: interpreter.runtime_flag.is_static(),
            is_eof: false,
            return_memory_offset,
        }))),
        InstructionResult::CallOrCreate,
    );
}

pub fn extcall_gas_calc<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
    target: Address,
    transfers_value: bool,
) -> Option<u64> {
    let Some(account_load) = host_load_account_delegated(target) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return None;
    };

    // account_load.is_empty will be accounted if there is transfer value
    // Berlin can be hardcoded as extcall came after berlin.
    let call_cost = gas::call_cost(
        interpreter.runtime_flag.spec_id(),
        transfers_value,
        account_load,
    );
    gas!(interpreter, call_cost, None);

    // Calculate the gas available to callee as caller’s
    // remaining gas reduced by max(ceil(gas/64), MIN_RETAINED_GAS) (MIN_RETAINED_GAS is 5000).
    let gas_reduce = max(interpreter.control.gas().remaining() / 64, 5000);
    let gas_limit = interpreter
        .control
        .gas()
        .remaining()
        .saturating_sub(gas_reduce);

    // The MIN_CALLEE_GAS rule is a replacement for stipend:
    // it simplifies the reasoning about the gas costs and is
    // applied uniformly for all introduced EXT*CALL instructions.
    //
    // If Gas available to callee is less than MIN_CALLEE_GAS trigger light failure (Same as Revert).
    if gas_limit < MIN_CALLEE_GAS {
        // Push 1 to stack to indicate that call light failed.
        // It is safe to ignore stack overflow error as we already popped multiple values from stack.
        let _ = interpreter.stack.push(U256::from(1));
        interpreter.return_data.clear();
        // Return none to continue execution.
        return None;
    }

    gas!(interpreter, gas_limit, None);
    Some(gas_limit)
}

pub fn extcall<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    require_eof!(interpreter);

    // Pop target address
    let Some(target_address) = pop_extcall_target_address(interpreter) else {
        return;
    };

    // Input call
    let Some(input) = extcall_input(interpreter) else {
        return;
    };

    popn!([value], interpreter);
    let has_transfer = !value.is_zero();
    if interpreter.runtime_flag.is_static() && has_transfer {
        interpreter
            .control
            .set_instruction_result(InstructionResult::CallNotAllowedInsideStatic);
        return;
    }

    let Some(gas_limit) = extcall_gas_calc(interpreter, host, target_address, has_transfer) else {
        return;
    };

    // Call host to interact with target contract
    interpreter.control.set_next_action(
        InterpreterAction::NewFrame(FrameInput::Call(Box::new(CallInputs {
            input,
            gas_limit,
            target_address,
            caller: interpreter.input.target_address(),
            bytecode_address: target_address,
            value: CallValue::Transfer(value),
            scheme: CallScheme::ExtCall,
            is_static: interpreter.runtime_flag.is_static(),
            is_eof: true,
            return_memory_offset: 0..0,
        }))),
        InstructionResult::CallOrCreate,
    );
}

pub fn extdelegatecall<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    require_eof!(interpreter);

    // Pop target address
    let Some(target_address) = pop_extcall_target_address(interpreter) else {
        return;
    };

    // Input call
    let Some(input) = extcall_input(interpreter) else {
        return;
    };

    let Some(gas_limit) = extcall_gas_calc(interpreter, host, target_address, false) else {
        return;
    };

    // Call host to interact with target contract
    interpreter.control.set_next_action(
        InterpreterAction::NewFrame(FrameInput::Call(Box::new(CallInputs {
            input,
            gas_limit,
            target_address: interpreter.input.target_address(),
            caller: interpreter.input.caller_address(),
            bytecode_address: target_address,
            value: CallValue::Apparent(interpreter.input.call_value()),
            scheme: CallScheme::ExtDelegateCall,
            is_static: interpreter.runtime_flag.is_static(),
            is_eof: true,
            return_memory_offset: 0..0,
        }))),
        InstructionResult::CallOrCreate,
    );
}

pub fn static_call<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    popn!([local_gas_limit, to], interpreter);
    let to = Address::from_word(B256::from(to));
    // Max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(interpreter) else {
        return;
    };

    let Some(mut load) = host_load_account_delegated(to) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    // Set `is_empty` to false as we are not creating this account.
    load.is_empty = false;
    let Some(gas_limit) = calc_call_gas(interpreter, load, false, local_gas_limit) else {
        return;
    };
    gas!(interpreter, gas_limit);

    // Call host to interact with target contract
    interpreter.control.set_next_action(
        InterpreterAction::NewFrame(FrameInput::Call(Box::new(CallInputs {
            input,
            gas_limit,
            target_address: to,
            caller: interpreter.input.target_address(),
            bytecode_address: to,
            value: CallValue::Transfer(U256::ZERO),
            scheme: CallScheme::StaticCall,
            is_static: true,
            is_eof: false,
            return_memory_offset,
        }))),
        InstructionResult::CallOrCreate,
    );
}

pub fn extstaticcall<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    require_eof!(interpreter);

    // Pop target address
    let Some(target_address) = pop_extcall_target_address(interpreter) else {
        return;
    };

    // Input call
    let Some(input) = extcall_input(interpreter) else {
        return;
    };

    let Some(gas_limit) = extcall_gas_calc(interpreter, host, target_address, false) else {
        return;
    };

    // Call host to interact with target contract
    interpreter.control.set_next_action(
        InterpreterAction::NewFrame(FrameInput::Call(Box::new(CallInputs {
            input,
            gas_limit,
            target_address,
            caller: interpreter.input.target_address(),
            bytecode_address: target_address,
            value: CallValue::Transfer(U256::ZERO),
            scheme: CallScheme::ExtStaticCall,
            is_static: true,
            is_eof: true,
            return_memory_offset: 0..0,
        }))),
        InstructionResult::CallOrCreate,
    );
}

pub fn selfdestruct<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    require_non_staticcall!(interpreter);
    popn!([target], interpreter);
    let target = target.into_address();

    let Some(res) = host_selfdestruct(interpreter.input.target_address(), target) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    // EIP-3529: Reduction in refunds
    if !interpreter.runtime_flag.spec_id().is_enabled_in(LONDON) && !res.previously_destroyed {
        interpreter
            .control
            .gas_mut()
            .record_refund(gas::SELFDESTRUCT)
    }

    gas!(
        interpreter,
        gas::selfdestruct_cost(interpreter.runtime_flag.spec_id(), res)
    );

    interpreter
        .control
        .set_instruction_result(InstructionResult::SelfDestruct);
}

//! Helper functions for the Hybrid EVM execution.
use reth::revm::{
    context::CreateScheme,
    interpreter::{CreateInputs, FrameInput},
};
use reth::revm::{
    interpreter::{
        CallInputs, CallScheme, CallValue, Host, InstructionResult, Interpreter, InterpreterAction,
        InterpreterResult,
    },
    primitives::{Address, Bytes, U256},
};
use rvemu::emulator::Emulator;
use std::collections::BTreeMap;

use crate::{
    execution::gas::{CALL_BASE, CALL_EMPTY_ACCOUNT, CALL_NEW_ACCOUNT, CALL_VALUE, CREATE_BASE},
    syscall_gas,
};

pub fn r55_gas_used(inst_count: &BTreeMap<String, u64>) -> u64 {
    let total_cost = inst_count
        .iter()
        .map(|(inst_name, count)|
            // Gas cost = number of instructions * cycles per instruction
            match inst_name.as_str() {
                // Gas map to approximate cost of each instruction
                // References:
                // http://ithare.com/infographics-operation-costs-in-cpu-clock-cycles/
                // https://www.evm.codes/?fork=cancun#54
                // Division and remainder
                s if s.starts_with("div") || s.starts_with("rem") => count * 25,
                // Multiplications
                s if s.starts_with("mul") => count * 5,
                // Loads
                "lb" | "lh" | "lw" | "ld" | "lbu" | "lhu" | "lwu" => count * 3, // Cost analagous to `MLOAD`
                // Stores
                "sb" | "sh" | "sw" | "sd" | "sc.w" | "sc.d" => count * 3, // Cost analagous to `MSTORE`
                // Branching
                "beq" | "bne" | "blt" | "bge" | "bltu" | "bgeu" | "jal" | "jalr" => count * 3,
                _ => *count, // All other instructions including `add` and `sub`
        })
        .sum::<u64>();

    // This is the minimum 'gas used' to ABI decode 'empty' calldata into Rust type arguments. Real calldata will take more gas.
    // Internalising this would focus gas metering more on the function logic
    let abi_decode_cost = 9_175_538;

    total_cost - abi_decode_cost
}

/// Returns RISC-V DRAM slice in a given size range, starts with a given offset
pub fn dram_slice(emu: &mut Emulator, ret_offset: u64, ret_size: u64) -> Result<&mut [u8], String> {
    if ret_size != 0 {
        Ok(emu
            .cpu
            .bus
            .get_dram_slice(ret_offset..(ret_offset + ret_size))
            .map_err(|e| "Failed to get DRAM slice".to_string() + ": " + &e.exception_message())?)
    } else {
        Ok(&mut [])
    }
}

pub fn execute_create(
    emu: &mut Emulator,
    interpreter: &mut Interpreter,
    _host: &mut dyn Host,
) -> Result<InterpreterAction, String> {
    let value: u64 = emu.cpu.xregs.read(10);

    // Get initcode
    let args_offset: u64 = emu.cpu.xregs.read(11);
    let args_size: u64 = emu.cpu.xregs.read(12);
    let init_code: Bytes = emu
        .cpu
        .bus
        .get_dram_slice(args_offset..(args_offset + args_size))
        .unwrap_or(&mut [])
        .to_vec()
        .into();

    // TODO: calculate gas cost properly
    let create_gas_cost = CREATE_BASE;
    syscall_gas!(interpreter, create_gas_cost);

    // proactively spend gas limit as the remaining will be refunded (otherwise it underflows)
    let create_gas_limit = interpreter.control.gas.remaining();
    syscall_gas!(interpreter, create_gas_limit);

    Ok(InterpreterAction::NewFrame(FrameInput::Create(Box::new(
        CreateInputs {
            init_code,
            gas_limit: create_gas_limit,
            caller: interpreter.input.target_address,
            value: U256::from(value),
            scheme: CreateScheme::Create,
        },
    ))))
}

pub fn execute_call(
    emu: &mut Emulator,
    interpreter: &mut Interpreter,
    host: &mut dyn Host,
    is_static: bool,
) -> Result<InterpreterAction, String> {
    let a0: u64 = emu.cpu.xregs.read(10);
    let a1: u64 = emu.cpu.xregs.read(11);
    let a2: u64 = emu.cpu.xregs.read(12);
    let addr = Address::from_word(U256::from_limbs([a0, a1, a2, 0]).into());
    let value: u64 = emu.cpu.xregs.read(13);

    // Get calldata
    let args_offset: u64 = emu.cpu.xregs.read(14);
    let args_size: u64 = emu.cpu.xregs.read(15);
    let calldata: Bytes = emu
        .cpu
        .bus
        .get_dram_slice(args_offset..(args_offset + args_size))
        .unwrap_or(&mut [])
        .to_vec()
        .into();

    // Calculate gas cost of the call
    // TODO: check correctness (tried using evm.codes as ref but i'm no gas wizard)
    // TODO: unsure whether memory expansion cost is missing (should be captured in the risc-v costs)
    let (empty_account_cost, addr_access_cost) = match host.load_account_delegated(addr) {
        Some(account) => {
            if account.is_cold {
                (0, CALL_NEW_ACCOUNT)
            } else {
                (0, CALL_BASE)
            }
        }
        None => (CALL_EMPTY_ACCOUNT, CALL_NEW_ACCOUNT),
    };
    let value_cost = if value != 0 { CALL_VALUE } else { 0 };
    let call_gas_cost = empty_account_cost + addr_access_cost + value_cost;
    syscall_gas!(interpreter, call_gas_cost);

    // proactively spend gas limit as the remaining will be refunded (otherwise it underflows)
    let call_gas_limit = interpreter.control.gas.remaining();
    syscall_gas!(interpreter, call_gas_limit);

    Ok(InterpreterAction::NewFrame(FrameInput::Call(Box::new(
        CallInputs {
            input: calldata,
            gas_limit: call_gas_limit,
            target_address: addr,
            bytecode_address: addr,
            caller: interpreter.input.target_address,
            value: CallValue::Transfer(U256::from(value)),
            scheme: CallScheme::Call,
            is_static,
            is_eof: false,
            return_memory_offset: 0..0, // handled with RETURNDATACOPY
        },
    ))))
}

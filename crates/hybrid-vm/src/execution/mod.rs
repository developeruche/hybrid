//! This module contains logic for executing an instance of a RISV-V interpreter.
//! Notice: This module was copied and modified from the r55 implementation.
//! r55 github: https://github.com/r55-eth/r55
use hybrid_syscalls::Syscall;
use reth::{
    primitives::Log,
    revm::{
        context::{ContextTr, Transaction},
        handler::{instructions::InstructionProvider, EvmTr, PrecompileProvider},
        interpreter::{
            interpreter::EthInterpreter,
            interpreter_types::{LoopControl, ReturnData},
            Host, InstructionResult, Interpreter, InterpreterAction, InterpreterResult,
        },
        primitives::{alloy_primitives::Keccak256, Address, Bytes, B256, U256},
    },
};
use rvemu::{emulator::Emulator, exception::Exception};

use crate::{
    execution::helper::{dram_slice, execute_call, execute_create, hybrid_gas_used},
    syscall_gas,
};

pub mod gas;
pub mod helper;

pub fn execute_riscv_contract<EVM>(
    emu: &mut Emulator,
    interpreter: &mut Interpreter,
    evm: &mut EVM,
    last_created_contract: &Option<Address>,
) -> Result<InterpreterAction, String>
where
    EVM: EvmTr<
        Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
        >,
    >,
{
    emu.cpu.is_count = true;

    let return_revert = |interpreter: &mut Interpreter, gas_used: u64| {
        let _ = interpreter.control.gas_mut().record_cost(gas_used);
        Ok(InterpreterAction::Return {
            result: InterpreterResult {
                result: InstructionResult::Revert,
                // return empty bytecode
                output: Bytes::new(),
                gas: interpreter.control.gas,
            },
        })
    };

    let host = &mut evm.ctx();

    loop {
        let run_result = emu.estart();

        match run_result {
            Err(Exception::EnvironmentCallFromMMode) => {
                let t0: u64 = emu.cpu.xregs.read(5);

                let Ok(syscall) = Syscall::try_from(t0 as u8) else {
                    return return_revert(interpreter, interpreter.control.gas.spent());
                };

                match syscall {
                    Syscall::Return => {
                        let ret_offset: u64 = emu.cpu.xregs.read(10);
                        let ret_size: u64 = emu.cpu.xregs.read(11);

                        let r55_gas = hybrid_gas_used(&emu.cpu.inst_counter);

                        // RETURN logs the gas of the whole risc-v instruction set
                        syscall_gas!(interpreter, r55_gas);

                        let data_bytes = dram_slice(emu, ret_offset, ret_size)?;

                        return Ok(InterpreterAction::Return {
                            result: InterpreterResult {
                                result: InstructionResult::Return,
                                output: data_bytes.to_vec().into(),
                                gas: interpreter.control.gas, // FIXME: gas is not correct
                            },
                        });
                    }
                    Syscall::SLoad => {
                        let key1: u64 = emu.cpu.xregs.read(10);
                        let key2: u64 = emu.cpu.xregs.read(11);
                        let key3: u64 = emu.cpu.xregs.read(12);
                        let key4: u64 = emu.cpu.xregs.read(13);
                        let key = U256::from_limbs([key1, key2, key3, key4]);

                        match host.sload(interpreter.input.target_address, key) {
                            Some(state_load) => {
                                let limbs = state_load.data.as_limbs();
                                emu.cpu.xregs.write(10, limbs[0]);
                                emu.cpu.xregs.write(11, limbs[1]);
                                emu.cpu.xregs.write(12, limbs[2]);
                                emu.cpu.xregs.write(13, limbs[3]);
                                syscall_gas!(
                                    interpreter,
                                    if state_load.is_cold {
                                        gas::SLOAD_COLD
                                    } else {
                                        gas::SLOAD_WARM
                                    }
                                );
                            }
                            _ => {
                                return return_revert(interpreter, interpreter.control.gas.spent());
                            }
                        }
                    }
                    Syscall::SStore => {
                        let key1: u64 = emu.cpu.xregs.read(10);
                        let key2: u64 = emu.cpu.xregs.read(11);
                        let key3: u64 = emu.cpu.xregs.read(12);
                        let key4: u64 = emu.cpu.xregs.read(13);
                        let key = U256::from_limbs([key1, key2, key3, key4]);

                        let val1: u64 = emu.cpu.xregs.read(14);
                        let val2: u64 = emu.cpu.xregs.read(15);
                        let val3: u64 = emu.cpu.xregs.read(16);
                        let val4: u64 = emu.cpu.xregs.read(17);
                        let value = U256::from_limbs([val1, val2, val3, val4]);

                        let result = host.sstore(interpreter.input.target_address, key, value);
                        if let Some(result) = result {
                            syscall_gas!(
                                interpreter,
                                if result.is_cold {
                                    gas::SSTORE_COLD
                                } else {
                                    gas::SSTORE_WARM
                                }
                            );
                        }
                    }
                    Syscall::ReturnDataSize => {
                        let size = interpreter.return_data.buffer().len();
                        emu.cpu.xregs.write(10, size as u64);
                    }
                    Syscall::ReturnDataCopy => {
                        let dest_offset = emu.cpu.xregs.read(10);
                        let offset = emu.cpu.xregs.read(11) as usize;
                        let size = emu.cpu.xregs.read(12) as usize;
                        let data = &interpreter.return_data.buffer()[offset..offset + size];

                        // write return data to memory
                        let return_memory = emu
                            .cpu
                            .bus
                            .get_dram_slice(dest_offset..(dest_offset + size as u64))
                            .map_err(|_| "Failed to get DRAM slice".to_string())?;
                        return_memory.copy_from_slice(data);
                    }
                    Syscall::Call => return execute_call(emu, interpreter, host, false),
                    Syscall::StaticCall => return execute_call(emu, interpreter, host, true),
                    Syscall::Create => return execute_create(emu, interpreter, host),
                    Syscall::ReturnCreateAddress => {
                        let dest_offset = emu.cpu.xregs.read(10);
                        let addr = last_created_contract.unwrap_or_default();

                        // write return data to memory
                        let return_memory = emu
                            .cpu
                            .bus
                            .get_dram_slice(dest_offset..(dest_offset + 20_u64))
                            .map_err(|_| "Failed to get DRAM slice".to_string())?;
                        return_memory.copy_from_slice(addr.as_slice());
                    }
                    Syscall::Revert => {
                        let ret_offset: u64 = emu.cpu.xregs.read(10);
                        let ret_size: u64 = emu.cpu.xregs.read(11);
                        let data_bytes: Vec<u8> = dram_slice(emu, ret_offset, ret_size)?.into();

                        return Ok(InterpreterAction::Return {
                            result: InterpreterResult {
                                result: InstructionResult::Revert,
                                output: Bytes::from(data_bytes),
                                gas: interpreter.control.gas, // FIXME: gas is not correct
                            },
                        });
                    }
                    Syscall::Caller => {
                        let caller = interpreter.input.caller_address;
                        // Break address into 3 u64s and write to registers
                        let caller_bytes = caller.as_slice();
                        let first_u64 = u64::from_be_bytes(
                            caller_bytes[0..8]
                                .try_into()
                                .map_err(|_| "Error converting caller address to u64")?,
                        );
                        emu.cpu.xregs.write(10, first_u64);
                        let second_u64 = u64::from_be_bytes(
                            caller_bytes[8..16]
                                .try_into()
                                .map_err(|_| "Error converting caller address to u64")?,
                        );
                        emu.cpu.xregs.write(11, second_u64);
                        let mut padded_bytes = [0u8; 8];
                        padded_bytes[..4].copy_from_slice(&caller_bytes[16..20]);
                        let third_u64 = u64::from_be_bytes(padded_bytes);
                        emu.cpu.xregs.write(12, third_u64);
                    }
                    Syscall::Keccak256 => {
                        let ret_offset: u64 = emu.cpu.xregs.read(10);
                        let ret_size: u64 = emu.cpu.xregs.read(11);
                        let data_bytes = dram_slice(emu, ret_offset, ret_size)?;

                        let mut hasher = Keccak256::new();
                        hasher.update(data_bytes);
                        let hash: U256 = hasher.finalize().into();

                        let limbs = hash.as_limbs();
                        emu.cpu.xregs.write(10, limbs[0]);
                        emu.cpu.xregs.write(11, limbs[1]);
                        emu.cpu.xregs.write(12, limbs[2]);
                        emu.cpu.xregs.write(13, limbs[3]);
                    }
                    Syscall::CallValue => {
                        let value = interpreter.input.call_value;
                        let limbs = value.into_limbs();
                        emu.cpu.xregs.write(10, limbs[0]);
                        emu.cpu.xregs.write(11, limbs[1]);
                        emu.cpu.xregs.write(12, limbs[2]);
                        emu.cpu.xregs.write(13, limbs[3]);
                    }
                    Syscall::BaseFee => {
                        let value = host.basefee();
                        let limbs = value.as_limbs();
                        emu.cpu.xregs.write(10, limbs[0]);
                        emu.cpu.xregs.write(11, limbs[1]);
                        emu.cpu.xregs.write(12, limbs[2]);
                        emu.cpu.xregs.write(13, limbs[3]);
                    }
                    Syscall::ChainId => {
                        let value = host.chain_id();
                        let value = value.as_le_bytes();
                        let mut arr = [0u8; 8];
                        arr.copy_from_slice(&value[..]);
                        emu.cpu.xregs.write(10, u64::from_le_bytes(arr));
                    }
                    Syscall::GasLimit => {
                        let limit = host.gas_limit();
                        let limbs = limit.as_limbs();
                        emu.cpu.xregs.write(10, limbs[0]);
                        emu.cpu.xregs.write(11, limbs[1]);
                        emu.cpu.xregs.write(12, limbs[2]);
                        emu.cpu.xregs.write(13, limbs[3]);
                    }
                    Syscall::Number => {
                        let number = host.block_number();
                        let limbs = U256::from(number);
                        let limbs = limbs.as_limbs();
                        emu.cpu.xregs.write(10, limbs[0]);
                        emu.cpu.xregs.write(11, limbs[1]);
                        emu.cpu.xregs.write(12, limbs[2]);
                        emu.cpu.xregs.write(13, limbs[3]);
                    }
                    Syscall::Timestamp => {
                        let timestamp = host.timestamp();
                        let limbs = timestamp.as_limbs();
                        emu.cpu.xregs.write(10, limbs[0]);
                        emu.cpu.xregs.write(11, limbs[1]);
                        emu.cpu.xregs.write(12, limbs[2]);
                        emu.cpu.xregs.write(13, limbs[3]);
                    }
                    Syscall::GasPrice => {
                        let value = host.tx().gas_price();
                        let limbs = U256::from(value);
                        let limbs = limbs.as_limbs();
                        emu.cpu.xregs.write(10, limbs[0]);
                        emu.cpu.xregs.write(11, limbs[1]);
                        emu.cpu.xregs.write(12, limbs[2]);
                        emu.cpu.xregs.write(13, limbs[3]);
                    }
                    Syscall::Origin => {
                        // Syscall::Origin
                        let origin = host.tx().caller();
                        // Break address into 3 u64s and write to registers
                        let origin_bytes = origin.as_slice();

                        let first_u64 = u64::from_be_bytes(origin_bytes[0..8].try_into().unwrap());
                        emu.cpu.xregs.write(10, first_u64);

                        let second_u64 =
                            u64::from_be_bytes(origin_bytes[8..16].try_into().unwrap());
                        emu.cpu.xregs.write(11, second_u64);

                        let mut padded_bytes = [0u8; 8];
                        padded_bytes[..4].copy_from_slice(&origin_bytes[16..20]);
                        let third_u64 = u64::from_be_bytes(padded_bytes);
                        emu.cpu.xregs.write(12, third_u64);
                    }
                    Syscall::Log => {
                        let data_ptr: u64 = emu.cpu.xregs.read(10);
                        let data_size: u64 = emu.cpu.xregs.read(11);
                        let topics_ptr: u64 = emu.cpu.xregs.read(12);
                        let topics_size: u64 = emu.cpu.xregs.read(13);

                        // Read data
                        let data = if data_size == 0 {
                            Vec::new()
                        } else {
                            let data_slice = emu
                                .cpu
                                .bus
                                .get_dram_slice(data_ptr..(data_ptr + data_size))
                                .unwrap_or(&mut []);
                            data_slice.to_vec()
                        };

                        // Read topics
                        let topics_start = topics_ptr;
                        let topics_end = topics_ptr + topics_size * 32;
                        let topics_slice = emu
                            .cpu
                            .bus
                            .get_dram_slice(topics_start..topics_end)
                            .unwrap_or(&mut []);
                        let topics = topics_slice
                            .chunks(32)
                            .map(B256::from_slice)
                            .collect::<Vec<B256>>();

                        host.log(Log::new_unchecked(
                            interpreter.input.target_address,
                            topics,
                            data.into(),
                        ));
                    }
                }
            }
            Ok(_) => {
                continue;
            }
            Err(e) => {
                println!("Error On Execute: {:?}", e);
                syscall_gas!(interpreter, hybrid_gas_used(&emu.cpu.inst_counter));
                return return_revert(interpreter, interpreter.control.gas.spent());
            }
        }
    }
}

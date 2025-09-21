use core::arch::asm;

use crate::utils::{__address_to_3u64, serialize_sstore_input};
use ext_revm::{
    interpreter::{SStoreResult, StateLoad},
    primitives::{Address, Bytes, FixedBytes, B256, U256},
};
use hybrid_contract::{slice_from_raw_parts, slice_from_raw_parts_mut};
use serde::{Deserialize, Serialize};

pub mod mini_evm_syscalls_ids {
    pub const HOST_BALANCE: u64 = 10;
    pub const HOST_LOAD_ACCOUNT_CODE: u64 = 11;
    pub const HOST_LOAD_ACCOUNT_CODE_HASH: u64 = 12;
    pub const HOST_BLOCK_NUMBER: u64 = 13;
    pub const HOST_BLOCK_HASH: u64 = 14;
    pub const HOST_SLOAD: u64 = 15;
    pub const HOST_SSTORE: u64 = 16;
    pub const HOST_TLOAD: u64 = 17;
    pub const HOST_TSTORE: u64 = 18;
}

#[derive(Serialize, Deserialize)]
pub struct SStoreInput {
    address: Address,
    index: U256,
    value: U256,
}

/// Allocating the last 20MB of the address space for the mini-evm syscalls
/// @dev When the emu have paging active, the memory address is not guaranteed to be available.
pub const MINI_EVM_SYSCALLS_MEM_ADDR: usize = 0xBEC00000;

pub fn host_balance(address: Address) -> Option<StateLoad<U256>> {
    let (limb_1, limb_2, limb_3) = __address_to_3u64(address);
    let mut size;

    unsafe {
        asm!(
            "ecall",
            in("a0") limb_1,
            in("a1") limb_2,
            in("a2") limb_3,
            lateout("a0") size,
            in("t0") mini_evm_syscalls_ids::HOST_BALANCE
        );
    }

    let out_serialized = unsafe { slice_from_raw_parts(MINI_EVM_SYSCALLS_MEM_ADDR, size) };

    let out: Option<StateLoad<U256>> =
        bincode::serde::decode_from_slice(out_serialized, bincode::config::legacy())
            .unwrap()
            .0;

    out
}

pub fn host_load_account_code(address: Address) -> Option<StateLoad<Bytes>> {
    let (limb_1, limb_2, limb_3) = __address_to_3u64(address);
    let mut size;

    unsafe {
        asm!(
            "ecall",
            in("a0") limb_1,
            in("a1") limb_2,
            in("a2") limb_3,
            lateout("a0") size,
            in("t0") mini_evm_syscalls_ids::HOST_LOAD_ACCOUNT_CODE
        );
    }

    let out_serialized = unsafe { slice_from_raw_parts(MINI_EVM_SYSCALLS_MEM_ADDR, size) };

    let out: Option<StateLoad<Bytes>> =
        bincode::serde::decode_from_slice(out_serialized, bincode::config::legacy())
            .unwrap()
            .0;

    out
}

pub fn host_load_account_code_hash(address: Address) -> Option<StateLoad<FixedBytes<32>>> {
    let (limb_1, limb_2, limb_3) = __address_to_3u64(address);
    let mut size;

    unsafe {
        asm!(
            "ecall",
            in("a0") limb_1,
            in("a1") limb_2,
            in("a2") limb_3,
            lateout("a0") size,
            in("t0") mini_evm_syscalls_ids::HOST_LOAD_ACCOUNT_CODE_HASH
        );
    }

    let out_serialized = unsafe { slice_from_raw_parts(MINI_EVM_SYSCALLS_MEM_ADDR, size) };

    let out: Option<StateLoad<FixedBytes<32>>> =
        bincode::serde::decode_from_slice(out_serialized, bincode::config::legacy())
            .unwrap()
            .0;

    out
}

pub fn host_block_number() -> u64 {
    let mut block_number;

    unsafe {
        asm!(
            "ecall",
            lateout("a0") block_number,
            in("t0") mini_evm_syscalls_ids::HOST_BLOCK_NUMBER
        );
    }

    block_number
}

pub fn host_block_hash(block_number: u64) -> Option<B256> {
    let mut size;

    unsafe {
        asm!(
            "ecall",
            in("a0") block_number,
            lateout("a0") size,
            in("t0") mini_evm_syscalls_ids::HOST_BLOCK_HASH
        );
    }

    let out_serialized = unsafe { slice_from_raw_parts(MINI_EVM_SYSCALLS_MEM_ADDR, size) };

    let out: Option<B256> =
        bincode::serde::decode_from_slice(out_serialized, bincode::config::legacy())
            .unwrap()
            .0;

    out
}

pub fn host_sload(address: Address, key: U256) -> Option<StateLoad<U256>> {
    let (addr_limb_1, addr_limb_2, addr_limb_3) = __address_to_3u64(address);
    let key_limbs = key.as_limbs();
    let mut size;

    unsafe {
        asm!(
            "ecall",
            in("a0") addr_limb_1,
            in("a1") addr_limb_2,
            in("a2") addr_limb_3,
            in("a3") key_limbs[0],
            in("a4") key_limbs[1],
            in("a5") key_limbs[2],
            in("a6") key_limbs[3],
            lateout("a0") size,
            in("t0") mini_evm_syscalls_ids::HOST_SLOAD
        );
    }

    let out_serialized = unsafe { slice_from_raw_parts(MINI_EVM_SYSCALLS_MEM_ADDR, size) };

    let out: Option<StateLoad<U256>> =
        bincode::serde::decode_from_slice(out_serialized, bincode::config::legacy())
            .unwrap()
            .0;

    out
}

pub fn host_sstore(address: Address, index: U256, value: U256) -> Option<StateLoad<SStoreResult>> {
    let input_serialized = serialize_sstore_input(address, index, value);
    let input_size = input_serialized.len();

    unsafe {
        let dest = slice_from_raw_parts_mut(MINI_EVM_SYSCALLS_MEM_ADDR, input_serialized.len());
        dest.copy_from_slice(&input_serialized);
    }

    let mut output_size;

    unsafe {
        asm!(
            "ecall",
            in("a0") input_size,
            lateout("a0") output_size,
            in("t0") mini_evm_syscalls_ids::HOST_SSTORE
        );
    }

    let out_serialized = unsafe { slice_from_raw_parts(MINI_EVM_SYSCALLS_MEM_ADDR, output_size) };

    let out: Option<StateLoad<SStoreResult>> =
        bincode::serde::decode_from_slice(out_serialized, bincode::config::legacy())
            .unwrap()
            .0;

    out
}

pub fn host_tload(address: Address, key: U256) -> U256 {
    let (addr_limb_1, addr_limb_2, addr_limb_3) = __address_to_3u64(address);
    let key_limbs = key.as_limbs();

    let mut out_limb_1;
    let mut out_limb_2;
    let mut out_limb_3;
    let mut out_limb_4;

    unsafe {
        asm!(
            "ecall",
            in("a0") addr_limb_1,
            in("a1") addr_limb_2,
            in("a2") addr_limb_3,
            in("a3") key_limbs[0],
            in("a4") key_limbs[1],
            in("a5") key_limbs[2],
            in("a6") key_limbs[3],
            lateout("a0") out_limb_1,
            lateout("a1") out_limb_2,
            lateout("a2") out_limb_3,
            lateout("a3") out_limb_4,
            in("t0") mini_evm_syscalls_ids::HOST_TLOAD
        );
    }

    U256::from_limbs([out_limb_1, out_limb_2, out_limb_3, out_limb_4])
}

pub fn host_tstore(address: Address, index: U256, value: U256) {
    let input_serialized = serialize_sstore_input(address, index, value);
    let input_size = input_serialized.len();

    unsafe {
        let dest = slice_from_raw_parts_mut(MINI_EVM_SYSCALLS_MEM_ADDR, input_serialized.len());
        dest.copy_from_slice(&input_serialized);
    }

    unsafe {
        asm!(
            "ecall",
            in("a0") input_size,
            in("t0") mini_evm_syscalls_ids::HOST_TSTORE
        );
    }
}

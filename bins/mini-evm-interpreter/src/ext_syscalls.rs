use core::arch::asm;

use crate::utils::__address_to_3u64;
use ext_revm::{
    interpreter::StateLoad,
    primitives::{Address, Bytes, U256},
};
use hybrid_contract::slice_from_raw_parts;

pub mod mini_evm_syscalls_ids {
    pub const HOST_BALANCE: u64 = 10;
    pub const HOST_LOAD_ACCOUNT_CODE: u64 = 11;
}

/// Allocating the last 20MB of the address space for the mini-evm syscalls
/// @dev When the emu have paging active, the memory address is not guaranteed to be available.
pub const MINI_EVM_SYSCALLS_MEM_ADDR: usize = 0xBEC00000;

pub fn host_balance(address: Address) -> Option<StateLoad<U256>> {
    let (limb_1, limb_2, limb_3) = __address_to_3u64(address);
    let mut offset_addr = 0;
    let mut size = 0;

    unsafe {
        asm!(
            "ecall",
            in("a0") limb_1,
            in("a1") limb_2,
            in("a2") limb_3,
            lateout("a0") offset_addr,
            lateout("a1") size,
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
    let mut offset_addr = 0;
    let mut size = 0;

    unsafe {
        asm!(
            "ecall",
            in("a0") limb_1,
            in("a1") limb_2,
            in("a2") limb_3,
            lateout("a0") offset_addr,
            lateout("a1") size,
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

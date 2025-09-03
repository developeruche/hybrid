//! This module holds global constants employed by the emulator.


/// This is the number of registers for this VM
pub const NUM_REGISTERS: usize = 32;


/// One byte len
pub const BYTE: u8 = 8;

/// Half word len
pub const HALFWORD: u8 = 16;

/// Whole word len
pub const WORD: u8 = 32;

/// Doubleword len
pub const DOUBLEWORD: u8 = 64;

/// riscv-pk is passing x10 and x11 registers to kernel. x11 is expected to have the pointer to DTB.
/// https://github.com/riscv/riscv-pk/blob/master/machine/mentry.S#L233-L235
pub const POINTER_TO_DTB: u64 = 0x1020;

/// An address where the RAM starts reading from
pub const RAM_BASE: u64 = 0x8000_0000;

/// Size of the RAM
pub const RAM_SIZE: u64 = 0x40000000;

/// An address where the RAM mem ends
const RAM_END: u64 = RAM_BASE + RAM_SIZE;

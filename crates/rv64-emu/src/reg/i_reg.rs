//! This module handles the Registers (Intergers)

use crate::{bus::DRAM_BASE, cpu::{POINTER_TO_DTB, REGISTERS_COUNT}, mem::DRAM_SIZE};


/// RV64 interger register
#[derive(Debug)]
pub struct IntRegister {
    regs: [u64; REGISTERS_COUNT],
}

impl IntRegister {
    /// Function creates new interger register
    pub fn new() -> Self {
        let mut regs = [0; REGISTERS_COUNT];
        // The stack pointer is set in the default maximum memory size + the start address of dram.
        regs[2] = DRAM_BASE + DRAM_SIZE;
        // From riscv-pk:
        // https://github.com/riscv/riscv-pk/blob/master/machine/mentry.S#L233-L235
        //   save a0 and a1; arguments from previous boot loader stage:
        //   // li x10, 0
        //   // li x11, 0
        //
        // void init_first_hart(uintptr_t hartid, uintptr_t dtb)
        //   x10 (a0): hartid
        //   x11 (a1): pointer to dtb
        //
        // So, we need to set registers register to the state as they are when a bootloader finished.
        regs[10] = 0;
        regs[11] = POINTER_TO_DTB;

        Self { regs }
    }

    /// Read the value from a register.
    pub fn read(&self, index: u64) -> u64 {
        self.regs[index as usize]
    }

    /// Write the value to a register.
    pub fn write(&mut self, index: u64, value: u64) {
        // Register x0 is hardwired with all bits equal to 0.
        if index != 0 {
            self.regs[index as usize] = value;
        }
    }
}

impl core::fmt::Display for IntRegister {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let abi = [
            "zero", " ra ", " sp ", " gp ", " tp ", " t0 ", " t1 ", " t2 ", " s0 ", " s1 ", " a0 ",
            " a1 ", " a2 ", " a3 ", " a4 ", " a5 ", " a6 ", " a7 ", " s2 ", " s3 ", " s4 ", " s5 ",
            " s6 ", " s7 ", " s8 ", " s9 ", " s10", " s11", " t3 ", " t4 ", " t5 ", " t6 ",
        ];
        let mut output = String::from("");
        for i in (0..REGISTERS_COUNT).step_by(4) {
            output = format!(
                "{}\n{}",
                output,
                format!(
                    "x{:02}({})={:>#18x} x{:02}({})={:>#18x} x{:02}({})={:>#18x} x{:02}({})={:>#18x}",
                    i,
                    abi[i],
                    self.read(i as u64),
                    i + 1,
                    abi[i + 1],
                    self.read(i as u64 + 1),
                    i + 2,
                    abi[i + 2],
                    self.read(i as u64 + 2),
                    i + 3,
                    abi[i + 3],
                    self.read(i as u64 + 3),
                )
            );
        }
        write!(f, "{}", output)
    }
}

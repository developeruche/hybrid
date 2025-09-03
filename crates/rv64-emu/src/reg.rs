//! This module handles the Registers (Intergers and Floating point)
use crate::primitives::constants::{NUM_REGISTERS, POINTER_TO_DTB, RAM_BASE, RAM_SIZE};

/// RV64 interger register
pub struct IntRegister {
    regs: [u64; NUM_REGISTERS],
}

impl IntRegister {
    /// Function creates new interger register
    pub fn new() -> Self {
        let mut regs = [0; NUM_REGISTERS];
        // The stack pointer is set in the default maximum memory size + the start address of dram.
        regs[2] = RAM_BASE + RAM_SIZE;
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

//TODO::@developeruche -> Implement Display on IntRegister
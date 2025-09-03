//! This module handles the Registers (Floating Point)
use crate::primitives::constants::NUM_REGISTERS;

/// RV64 floating point register
#[derive(Debug)]
pub struct FloatRegister {
    regs: [f64; NUM_REGISTERS],
}

impl FloatRegister {
    /// Create a new `FRegisters` object.
    pub fn new() -> Self {
        Self {
            regs: [0.0; NUM_REGISTERS],
        }
    }

    /// Read the value from a register.
    pub fn read(&self, index: u64) -> f64 {
        self.regs[index as usize]
    }

    /// Write the value to a register.
    pub fn write(&mut self, index: u64, value: f64) {
        self.regs[index as usize] = value;
    }
}

#[cfg(feature = "std")]
impl core::fmt::Display for FloatRegister {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let abi = [
            // ft0-7: FP temporaries
            " ft0", " ft1", " ft2", " ft3", " ft4", " ft5", " ft6", " ft7",
            // fs0-1: FP saved registers
            " fs0", " fs1", // fa0-1: FP arguments/return values
            " fa0", " fa1", // fa2–7: FP arguments
            " fa2", " fa3", " fa4", " fa5", " fa6", " fa7",
            // fs2–11: FP saved registers
            " fs2", " fs3", " fs4", " fs5", " fs6", " fs7", " fs8", " fs9", "fs10", "fs11",
            // ft8–11: FP temporaries
            " ft8", " ft9", "ft10", "ft11",
        ];
        let mut output = String::from("");
        for i in (0..NUM_REGISTERS).step_by(4) {
            output = format!(
                "{}\n{}",
                output,
                format!(
                    "f{:02}({})={:>width$.prec$} f{:02}({})={:>width$.prec$} f{:02}({})={:>width$.prec$} f{:02}({})={:>width$.prec$}",
                    i,
                    abi[i],
                    self.read(i as u64),
                    i + 1,
                    abi[i + 1],
                    self.read(i as u64 + 1),
                    i + 2,
                    abi[i + 2],
                    self.read(i  as u64+ 2),
                    i + 3,
                    abi[i + 3],
                    self.read(i as u64 + 3),
                    width=18,
                    prec=8,
                )
            );
        }
        // Remove the first new line.
        output.remove(0);
        write!(f, "{}", output)
    }
}

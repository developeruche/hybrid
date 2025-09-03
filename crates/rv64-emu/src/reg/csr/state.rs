//! This module manages the state of all CRSs
use crate::reg::csr::{
    utils::to_range, CsrAddress, CsrFieldRange, CSR_SIZE, MCAUSE, MEDELEG, MEPC, MIDELEG, MIE, MIP,
    MISA, MSTATUS, MTVEC, MXLEN, SCAUSE, SEDELEG, SEPC, SIDELEG, SIE, SSIP_BIT, SSTATUS,
    SSTATUS_MASK, STVEC, TIME, UCAUSE, UEPC, USTATUS, UTVEC,
};
use core::ops::RangeBounds;

/// The state to contains all the CSRs.
#[derive(Debug)]
pub struct State {
    csrs: [u64; CSR_SIZE],
}

impl State {
    /// Create a new `state` object.
    pub fn new() -> Self {
        let mut csrs = [0; CSR_SIZE];
        let misa: u64 = (2 << 62) | // MXL[1:0]=2 (XLEN is 64)
            (1 << 20) | // Extensions[20] (User mode implemented)
            (1 << 18) | // Extensions[18] (Supervisor mode implemented)
            (1 << 12) | // Extensions[12] (Integer Multiply/Divide extension)
            (1 << 8) | // Extensions[8] (RV32I/64I/128I base ISA)
            (1 << 5) | // Extensions[5] (Single-precision floating-point extension)
            (1 << 3) | // Extensions[3] (Double-precision floating-point extension)
            (1 << 2) | // Extensions[2] (Compressed extension)
            1; // Extensions[0] (Atomic extension)
        csrs[MISA as usize] = misa;

        Self { csrs }
    }

    /// Increment the value in the TIME register.
    pub fn increment_time(&mut self) {
        self.csrs[TIME as usize] = self.csrs[TIME as usize].wrapping_add(1);
    }

    /// Read the val from the CSR.
    pub fn read(&self, addr: CsrAddress) -> u64 {
        // 4.1 Supervisor CSRs
        // "The supervisor should only view CSR state that should be visible to a supervisor-level
        // operating system. In particular, there is no information about the existence (or
        // non-existence) of higher privilege levels (machine level or other) visible in the CSRs
        // accessible by the supervisor.  Many supervisor CSRs are a subset of the equivalent
        // machine-mode CSR, and the machinemode chapter should be read first to help understand
        // the supervisor-level CSR descriptions."
        match addr {
            SSTATUS => self.csrs[MSTATUS as usize] & SSTATUS_MASK,
            SIE => self.csrs[MIE as usize] & self.csrs[MIDELEG as usize],
            SIP => self.csrs[MIP as usize] & self.csrs[MIDELEG as usize],
            _ => self.csrs[addr as usize],
        }
    }

    /// Write the val to the CSR.
    pub fn write(&mut self, addr: CsrAddress, val: u64) {
        // 4.1 Supervisor CSRs
        // "The supervisor should only view CSR state that should be visible to a supervisor-level
        // operating system. In particular, there is no information about the existence (or
        // non-existence) of higher privilege levels (machine level or other) visible in the CSRs
        // accessible by the supervisor.  Many supervisor CSRs are a subset of the equivalent
        // machine-mode CSR, and the machinemode chapter should be read first to help understand
        // the supervisor-level CSR descriptions."
        match addr {
            MVENDORID => {}
            MARCHID => {}
            MIMPID => {}
            MHARTID => {}
            SSTATUS => {
                self.csrs[MSTATUS as usize] =
                    (self.csrs[MSTATUS as usize] & !SSTATUS_MASK) | (val & SSTATUS_MASK);
            }
            SIE => {
                self.csrs[MIE as usize] = (self.csrs[MIE as usize] & !self.csrs[MIDELEG as usize])
                    | (val & self.csrs[MIDELEG as usize]);
            }
            SIP => {
                let mask = SSIP_BIT & self.csrs[MIDELEG as usize];
                self.csrs[MIP as usize] = (self.csrs[MIP as usize] & !mask) | (val & mask);
            }
            _ => self.csrs[addr as usize] = val,
        }
    }

    /// Read a bit from the CSR.
    pub fn read_bit(&self, addr: CsrAddress, bit: usize) -> u64 {
        if bit >= MXLEN {
            // TODO: raise exception?
        }

        if (self.read(addr) & (1 << bit)) != 0 {
            1
        } else {
            0
        }
    }

    /// Read a arbitrary length of bits from the CSR.
    pub fn read_bits<T: RangeBounds<usize>>(&self, addr: CsrAddress, range: T) -> u64 {
        let range = to_range(&range, MXLEN);

        if (range.start >= MXLEN) | (range.end > MXLEN) | (range.start >= range.end) {
            // TODO: ranse exception?
        }

        // Bitmask for high bits.
        let mut bitmask = 0;
        if range.end != 64 {
            bitmask = !0 << range.end;
        }

        // Shift away low bits.
        (self.read(addr) as u64 & !bitmask) >> range.start
    }

    /// Write a bit to the CSR.
    pub fn write_bit(&mut self, addr: CsrAddress, bit: usize, val: u64) {
        if bit >= MXLEN {
            // TODO: raise exception?
        }
        if val > 1 {
            // TODO: raise exception
        }

        if val == 1 {
            self.write(addr, self.read(addr) | 1 << bit);
        } else if val == 0 {
            self.write(addr, self.read(addr) & !(1 << bit));
        }
    }

    /// Write an arbitrary length of bits to the CSR.
    pub fn write_bits<T: RangeBounds<usize>>(&mut self, addr: CsrAddress, range: T, val: u64) {
        let range = to_range(&range, MXLEN);

        if (range.start >= MXLEN) | (range.end > MXLEN) | (range.start >= range.end) {
            // TODO: ranse exception?
        }
        if (val >> (range.end - range.start)) != 0 {
            // TODO: raise exception
        }

        let bitmask = (!0 << range.end) | !(!0 << range.start);
        // Set bits.
        self.write(addr, (self.read(addr) & bitmask) | (val << range.start))
    }

    /// Read bit(s) from a given field in the SSTATUS register.
    pub fn read_sstatus(&self, range: CsrFieldRange) -> u64 {
        self.read_bits(SSTATUS, range)
    }

    /// Read bit(s) from a given field in the MSTATUS register.
    pub fn read_mstatus(&self, range: CsrFieldRange) -> u64 {
        self.read_bits(MSTATUS, range)
    }

    /// Write bit(s) to a given field in the SSTATUS register.
    pub fn write_sstatus(&mut self, range: CsrFieldRange, val: u64) {
        self.write_bits(SSTATUS, range, val);
    }

    /// Write bit(s) to a given field in the MSTATUS register.
    pub fn write_mstatus(&mut self, range: CsrFieldRange, val: u64) {
        self.write_bits(MSTATUS, range, val);
    }

    /// Reset all the CSRs.
    pub fn reset(&mut self) {
        self.csrs = [0; CSR_SIZE];

        let misa: u64 = (2 << 62) | // MXL[1:0]=2 (XLEN is 64)
            (1 << 18) | // Extensions[18] (Supervisor mode implemented)
            (1 << 12) | // Extensions[12] (Integer Multiply/Divide extension)
            (1 << 8) | // Extensions[8] (RV32I/64I/128I base ISA)
            (1 << 5) | // Extensions[5] (Single-precision floating-point extension)
            (1 << 3) | // Extensions[3] (Double-precision floating-point extension)
            (1 << 2) | // Extensions[2] (Compressed extension)
            1; // Extensions[0] (Atomic extension)
        self.csrs[MISA as usize] = misa;
    }
}

impl core::fmt::Display for State {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "{}\n{}\n{}",
                format!(
                    "mstatus={:>#18x}   mtvec={:>#18x}    mepc={:>#18x}\n mcause={:>#18x} medeleg={:>#18x} mideleg={:>#18x}",
                    self.read(MSTATUS),
                    self.read(MTVEC),
                    self.read(MEPC),
                    self.read(MCAUSE),
                    self.read(MEDELEG),
                    self.read(MIDELEG),
                ),
                format!(
                    "sstatus={:>#18x}   stvec={:>#18x}    sepc={:>#18x}\n scause={:>#18x} sedeleg={:>#18x} sideleg={:>#18x}",
                    self.read(SSTATUS),
                    self.read(STVEC),
                    self.read(SEPC),
                    self.read(SCAUSE),
                    self.read(SEDELEG),
                    self.read(SIDELEG),
                ),
                format!(
                    "ustatus={:>#18x}   utvec={:>#18x}    uepc={:>#18x}\n ucause={:>#18x}",
                    self.read(USTATUS),
                    self.read(UTVEC),
                    self.read(UEPC),
                    self.read(UCAUSE),
                ),
            )
        )
    }
}

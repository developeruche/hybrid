//! CPU module for the RV64 emulator.
use crate::{
    bus::Bus,
    cpu::{compressed_exec::execute_compressed_inner, general_exec::execute_general_inner},
    devices::{
        uart_cli::UART_IRQ,
        virtio_blk::{Virtio, VIRTIO_IRQ},
    },
    exception::Exception,
    interrupt::Interrupt,
    reg::{
        csr::{
            state::State, MEIP_BIT, MIE, MIP, MSIP_BIT, MSTATUS_MIE, MSTATUS_MPP, MSTATUS_MPRV,
            MTIP_BIT, SATP, SEIP_BIT, SSIP_BIT, STIP_BIT, XSTATUS_SIE,
        },
        f_reg::FloatRegister,
        i_reg::IntRegister,
    },
};
use std::collections::BTreeMap;

pub mod compressed_exec;
pub mod general_exec;


/// The number of registers.
pub const REGISTERS_COUNT: usize = 32;
/// The page size (4 KiB) for the virtual memory system.
const PAGE_SIZE: u64 = 4096;

/// 8 bits. 1 byte.
pub const BYTE: u8 = 8;
/// 16 bits. 2 bytes.
pub const HALFWORD: u8 = 16;
/// 32 bits. 4 bytes.
pub const WORD: u8 = 32;
/// 64 bits. 8 bytes.
pub const DOUBLEWORD: u8 = 64;

/// riscv-pk is passing x10 and x11 registers to kernel. x11 is expected to have the pointer to DTB.
/// https://github.com/riscv/riscv-pk/blob/master/machine/mentry.S#L233-L235
pub const POINTER_TO_DTB: u64 = 0x1020;


/// Access type that is used in the virtual address translation process. It decides which exception
/// should raises (InstructionPageFault, LoadPageFault or StoreAMOPageFault).
#[derive(Debug, PartialEq, PartialOrd)]
pub enum AccessType {
    /// Raises the exception InstructionPageFault. It is used for an instruction fetch.
    Instruction,
    /// Raises the exception LoadPageFault.
    Load,
    /// Raises the exception StoreAMOPageFault.
    Store,
}

/// The privileged mode.
#[derive(Debug, PartialEq, PartialOrd, Eq, Copy, Clone)]
pub enum Mode {
    User = 0b00,
    Supervisor = 0b01,
    Machine = 0b11,
    Debug,
}

#[derive(Debug)]
pub struct CPU {
    /// Interger registers
    pub int_regs: IntRegister,
    /// Floating-point registers
    pub float_regs: FloatRegister,
    /// Program counter.
    pub pc: u64,
    /// Control and status registers (CSR).
    pub state: State,
    /// Privilege level.
    pub mode: Mode,
    /// System bus.
    pub bus: Bus,
    /// SV39 paging flag.
    enable_paging: bool,
    /// Physical page number (PPN) × PAGE_SIZE (4096).
    page_table: u64,
    /// A set of bytes that subsumes the bytes in the addressed word used in
    /// load-reserved/store-conditional instructions.
    reservation_set: Vec<u64>,
    /// Idle state. True when WFI is called, and becomes false when an interrupt happens.
    pub idle: bool,
    /// Counter of each instructions for debug.
    pub inst_counter: BTreeMap<String, u64>,
    /// The count flag. Count the number of each instruction executed.
    pub is_count: bool,
    /// Previous instruction. This is for debug.
    pub pre_inst: u64,
}

impl CPU {
    /// Create a new `CPU` object.
    pub fn new() -> CPU {
        CPU {
            int_regs: IntRegister::new(),
            float_regs: FloatRegister::new(),
            pc: 0,
            state: State::new(),
            mode: Mode::Machine,
            bus: Bus::new(),
            enable_paging: false,
            page_table: 0,
            reservation_set: Vec::new(),
            idle: false,
            inst_counter: BTreeMap::new(),
            is_count: false,
            pre_inst: 0,
        }
    }

    /// Reset CPU states.
    pub fn reset(&mut self) {
        self.pc = 0;
        self.mode = Mode::Machine;
        self.state.reset();
        for i in 0..REGISTERS_COUNT {
            self.int_regs.write(i as u64, 0);
            self.float_regs.write(i as u64, 0.0);
        }
    }

    /// Check interrupt flags for all devices that can interrupt.
    pub fn check_pending_interrupt(&mut self) -> Option<Interrupt> {
        // global interrupt: PLIC (Platform Local Interrupt Controller) dispatches global
        //                   interrupts to multiple harts.
        // local interrupt: CLINT (Core Local Interrupter) dispatches local interrupts to a hart
        //                  which directly connected to CLINT.

        // 3.1.6.1 Privilege and Global Interrupt-Enable Stack in mstatus register
        // "When a hart is executing in privilege mode x, interrupts are globally enabled when
        // xIE=1 and globally disabled when xIE=0."
        match self.mode {
            Mode::Machine => {
                // Check if the MIE bit is enabled.
                if self.state.read_mstatus(MSTATUS_MIE) == 0 {
                    return None;
                }
            }
            Mode::Supervisor => {
                // Check if the SIE bit is enabled.
                if self.state.read_sstatus(XSTATUS_SIE) == 0 {
                    return None;
                }
            }
            _ => {}
        }

        // TODO: Take interrupts based on priorities.

        // Check external interrupt for uart and virtio.
        let irq;
        if self.bus.uart.is_interrupting() {
            irq = UART_IRQ;
        } else if self.bus.virtio.is_interrupting() {
            // An interrupt is raised after a disk access is done.
            Virtio::disk_access(self).expect("failed to access the disk");
            irq = VIRTIO_IRQ;
        } else {
            irq = 0;
        }

        if irq != 0 {
            // TODO: assume that hart is 0
            // TODO: write a value to MCLAIM if the mode is machine
            self.bus.plic.update_pending(irq);
            self.state.write(MIP, self.state.read(MIP) | SEIP_BIT);
        }

        // 3.1.9 Machine Interrupt Registers (mip and mie)
        // "An interrupt i will be taken if bit i is set in both mip and mie, and if interrupts are
        // globally enabled. By default, M-mode interrupts are globally enabled if the hart’s
        // current privilege mode is less than M, or if the current privilege mode is M and the MIE
        // bit in the mstatus register is set. If bit i in mideleg is set, however, interrupts are
        // considered to be globally enabled if the hart’s current privilege mode equals the
        // delegated privilege mode (S or U) and that mode’s interrupt enable bit (SIE or UIE in
        // mstatus) is set, or if the current privilege mode is less than the delegated privilege
        // mode."
        let pending = self.state.read(MIE) & self.state.read(MIP);

        if (pending & MEIP_BIT) != 0 {
            self.state.write(MIP, self.state.read(MIP) & !MEIP_BIT);
            return Some(Interrupt::MachineExternalInterrupt);
        }
        if (pending & MSIP_BIT) != 0 {
            self.state.write(MIP, self.state.read(MIP) & !MSIP_BIT);
            return Some(Interrupt::MachineSoftwareInterrupt);
        }
        if (pending & MTIP_BIT) != 0 {
            self.state.write(MIP, self.state.read(MIP) & !MTIP_BIT);
            return Some(Interrupt::MachineTimerInterrupt);
        }
        if (pending & SEIP_BIT) != 0 {
            self.state.write(MIP, self.state.read(MIP) & !SEIP_BIT);
            return Some(Interrupt::SupervisorExternalInterrupt);
        }
        if (pending & SSIP_BIT) != 0 {
            self.state.write(MIP, self.state.read(MIP) & !SSIP_BIT);
            return Some(Interrupt::SupervisorSoftwareInterrupt);
        }
        if (pending & STIP_BIT) != 0 {
            self.state.write(MIP, self.state.read(MIP) & !STIP_BIT);
            return Some(Interrupt::SupervisorTimerInterrupt);
        }

        return None;
    }

    /// Update the physical page number (PPN) and the addressing mode.
    pub fn update_paging(&mut self) {
        // Read the physical page number (PPN) of the root page table, i.e., its
        // supervisor physical address divided by 4 KiB.
        self.page_table = self.state.read_bits(SATP, ..44) * PAGE_SIZE;

        // Read the MODE field, which selects the current address-translation scheme.
        let mode = self.state.read_bits(SATP, 60..);

        // Enable the SV39 paging if the value of the mode field is 8.
        if mode == 8 {
            self.enable_paging = true;
        } else {
            self.enable_paging = false;
        }
    }

    /// Translate a virtual address to a physical address for the paged virtual-memory system.
    fn translate(&mut self, addr: u64, access_type: AccessType) -> Result<u64, Exception> {
        if !self.enable_paging || self.mode == Mode::Machine {
            return Ok(addr);
        }

        // 4.3.2 Virtual Address Translation Process
        // (The RISC-V Instruction Set Manual Volume II-Privileged Architecture_20190608)
        // A virtual address va is translated into a physical address pa as follows:
        let levels = 3;
        let vpn = [
            (addr >> 12) & 0x1ff,
            (addr >> 21) & 0x1ff,
            (addr >> 30) & 0x1ff,
        ];

        // 1. Let a be satp.ppn × PAGESIZE, and let i = LEVELS − 1. (For Sv32, PAGESIZE=212
        //    and LEVELS=2.)
        let mut a = self.page_table;
        let mut i: i64 = levels - 1;
        let mut pte;
        loop {
            // 2. Let pte be the value of the PTE at address a+va.vpn[i]×PTESIZE. (For Sv32,
            //    PTESIZE=4.) If accessing pte violates a PMA or PMP check, raise an access
            //    exception corresponding to the original access type.
            pte = self.bus.read(a + vpn[i as usize] * 8, DOUBLEWORD)?;

            // 3. If pte.v = 0, or if pte.r = 0 and pte.w = 1, stop and raise a page-fault
            //    exception corresponding to the original access type.
            let v = pte & 1;
            let r = (pte >> 1) & 1;
            let w = (pte >> 2) & 1;
            let x = (pte >> 3) & 1;
            if v == 0 || (r == 0 && w == 1) {
                match access_type {
                    AccessType::Instruction => return Err(Exception::InstructionPageFault(addr)),
                    AccessType::Load => return Err(Exception::LoadPageFault(addr)),
                    AccessType::Store => return Err(Exception::StoreAMOPageFault(addr)),
                }
            }

            // 4. Otherwise, the PTE is valid. If pte.r = 1 or pte.x = 1, go to step 5.
            //    Otherwise, this PTE is a pointer to the next level of the page table.
            //    Let i = i − 1. If i < 0, stop and raise a page-fault exception
            //    corresponding to the original access type. Otherwise,
            //    let a = pte.ppn × PAGESIZE and go to step 2.
            if r == 1 || x == 1 {
                break;
            }
            i -= 1;
            let ppn = (pte >> 10) & 0x0fff_ffff_ffff;
            a = ppn * PAGE_SIZE;
            if i < 0 {
                match access_type {
                    AccessType::Instruction => return Err(Exception::InstructionPageFault(addr)),
                    AccessType::Load => return Err(Exception::LoadPageFault(addr)),
                    AccessType::Store => return Err(Exception::StoreAMOPageFault(addr)),
                }
            }
        }
        // TODO: implement step 5
        // 5. A leaf PTE has been found. Determine if the requested memory access is
        //    allowed by the pte.r, pte.w, pte.x, and pte.u bits, given the current
        //    privilege mode and the value of the SUM and MXR fields of the mstatus
        //    register. If not, stop and raise a page-fault exception corresponding
        //    to the original access type.

        // 3.1.6.3 Memory Privilege in mstatus Register
        // "The MXR (Make eXecutable Readable) bit modifies the privilege with which loads access
        // virtual memory. When MXR=0, only loads from pages marked readable (R=1 in Figure 4.15)
        // will succeed. When MXR=1, loads from pages marked either readable or executable
        // (R=1 or X=1) will succeed. MXR has no effect when page-based virtual memory is not in
        // effect. MXR is hardwired to 0 if S-mode is not supported."

        // "The SUM (permit Supervisor User Memory access) bit modifies the privilege with which
        // S-mode loads and stores access virtual memory. When SUM=0, S-mode memory accesses to
        // pages that are accessible by U-mode (U=1 in Figure 4.15) will fault. When SUM=1, these
        // accesses are permitted.  SUM has no effect when page-based virtual memory is not in
        // effect. Note that, while SUM is ordinarily ignored when not executing in S-mode, it is
        // in effect when MPRV=1 and MPP=S. SUM is hardwired to 0 if S-mode is not supported."

        // 6. If i > 0 and pte.ppn[i−1:0] != 0, this is a misaligned superpage; stop and
        //    raise a page-fault exception corresponding to the original access type.
        let ppn = [
            (pte >> 10) & 0x1ff,
            (pte >> 19) & 0x1ff,
            (pte >> 28) & 0x03ff_ffff,
        ];
        if i > 0 {
            for j in (0..i).rev() {
                if ppn[j as usize] != 0 {
                    // A misaligned superpage.
                    match access_type {
                        AccessType::Instruction => {
                            return Err(Exception::InstructionPageFault(addr))
                        }
                        AccessType::Load => return Err(Exception::LoadPageFault(addr)),
                        AccessType::Store => return Err(Exception::StoreAMOPageFault(addr)),
                    }
                }
            }
        }

        // 7. If pte.a = 0, or if the memory access is a store and pte.d = 0, either raise
        //    a page-fault exception corresponding to the original access type, or:
        //    • Set pte.a to 1 and, if the memory access is a store, also set pte.d to 1.
        //    • If this access violates a PMA or PMP check, raise an access exception
        //    corresponding to the original access type.
        //    • This update and the loading of pte in step 2 must be atomic; in particular,
        //    no intervening store to the PTE may be perceived to have occurred in-between.
        let a = (pte >> 6) & 1;
        let d = (pte >> 7) & 1;
        if a == 0 || (access_type == AccessType::Store && d == 0) {
            // Set pte.a to 1 and, if the memory access is a store, also set pte.d to 1.
            pte = pte
                | (1 << 6)
                | if access_type == AccessType::Store {
                    1 << 7
                } else {
                    0
                };

            // TODO: PMA or PMP check.

            // Update the value of address satp.ppn × PAGESIZE + va.vpn[i] × PTESIZE with new pte
            // value.
            // TODO: If this is enabled, running xv6 fails.
            //self.bus.write(self.page_table + vpn[i as usize] * 8, pte, 64)?;
        }

        // 8. The translation is successful. The translated physical address is given as
        //    follows:
        //    • pa.pgoff = va.pgoff.
        //    • If i > 0, then this is a superpage translation and pa.ppn[i−1:0] =
        //    va.vpn[i−1:0].
        //    • pa.ppn[LEVELS−1:i] = pte.ppn[LEVELS−1:i].
        let offset = addr & 0xfff;
        match i {
            0 => {
                let ppn = (pte >> 10) & 0x0fff_ffff_ffff;
                Ok((ppn << 12) | offset)
            }
            1 => {
                // Superpage translation. A superpage is a memory page of larger size than an
                // ordinary page (4 KiB). It reduces TLB misses and improves performance.
                Ok((ppn[2] << 30) | (ppn[1] << 21) | (vpn[0] << 12) | offset)
            }
            2 => {
                // Superpage translation. A superpage is a memory page of larger size than an
                // ordinary page (4 KiB). It reduces TLB misses and improves performance.
                Ok((ppn[2] << 30) | (vpn[1] << 21) | (vpn[0] << 12) | offset)
            }
            _ => match access_type {
                AccessType::Instruction => return Err(Exception::InstructionPageFault(addr)),
                AccessType::Load => return Err(Exception::LoadPageFault(addr)),
                AccessType::Store => return Err(Exception::StoreAMOPageFault(addr)),
            },
        }
    }

    /// Read `size`-bit data from the system bus with the translation a virtual address to a physical address
    /// if it is enabled.
    fn read(&mut self, v_addr: u64, size: u8) -> Result<u64, Exception> {
        let previous_mode = self.mode;

        // 3.1.6.3 Memory Privilege in mstatus Register
        // "When MPRV=1, load and store memory addresses are translated and protected, and
        // endianness is applied, as though the current privilege mode were set to MPP."
        if self.state.read_mstatus(MSTATUS_MPRV) == 1 {
            self.mode = match self.state.read_mstatus(MSTATUS_MPP) {
                0b00 => Mode::User,
                0b01 => Mode::Supervisor,
                0b11 => Mode::Machine,
                _ => Mode::Debug,
            };
        }

        let p_addr = self.translate(v_addr, AccessType::Load)?;
        let result = self.bus.read(p_addr, size);

        if self.state.read_mstatus(MSTATUS_MPRV) == 1 {
            self.mode = previous_mode;
        }

        result
    }

    /// Write `size`-bit data to the system bus with the translation a virtual address to a physical
    /// address if it is enabled.
    fn write(&mut self, v_addr: u64, value: u64, size: u8) -> Result<(), Exception> {
        let previous_mode = self.mode;

        // 3.1.6.3 Memory Privilege in mstatus Register
        // "When MPRV=1, load and store memory addresses are translated and protected, and
        // endianness is applied, as though the current privilege mode were set to MPP."
        if self.state.read_mstatus(MSTATUS_MPRV) == 1 {
            self.mode = match self.state.read_mstatus(MSTATUS_MPP) {
                0b00 => Mode::User,
                0b01 => Mode::Supervisor,
                0b11 => Mode::Machine,
                _ => Mode::Debug,
            };
        }

        // "The SC must fail if a write from some other device to the bytes accessed by the LR can
        // be observed to occur between the LR and SC."
        if self.reservation_set.contains(&v_addr) {
            self.reservation_set.retain(|&x| x != v_addr);
        }

        let p_addr = self.translate(v_addr, AccessType::Store)?;
        let result = self.bus.write(p_addr, value, size);

        if self.state.read_mstatus(MSTATUS_MPRV) == 1 {
            self.mode = previous_mode;
        }

        result
    }

    /// Fetch the `size`-bit next instruction from the memory at the current program counter.
    pub fn fetch(&mut self, size: u8) -> Result<u64, Exception> {
        if size != HALFWORD && size != WORD {
            return Err(Exception::InstructionAccessFault);
        }

        let p_pc = self.translate(self.pc, AccessType::Instruction)?;

        // The result of the read method can be `Exception::LoadAccessFault`. In fetch(), an error
        // should be `Exception::InstructionAccessFault`.
        match self.bus.read(p_pc, size) {
            Ok(value) => Ok(value),
            Err(_) => Err(Exception::InstructionAccessFault),
        }
    }

    /// Execute a cycle on peripheral devices.
    pub fn devices_increment(&mut self) {
        // TODO: mtime in Clint and TIME in CSR should be the same value.
        // Increment the timer register (mtimer) in Clint.
        self.bus.clint.increment(&mut self.state);
        // Increment the value in the TIME and CYCLE registers in CSR.
        self.state.increment_time();
    }

    /// Execute an instruction. Raises an exception if something is wrong, otherwise, returns
    /// the instruction executed in this cycle.
    pub fn execute(&mut self) -> Result<u64, Exception> {
        // WFI is called and pending interrupts don't exist.
        if self.idle {
            return Ok(0);
        }

        // Fetch.
        let inst16 = self.fetch(HALFWORD)?;
        let inst;
        match inst16 & 0b11 {
            0 | 1 | 2 => {
                if inst16 == 0 {
                    // Unimplemented instruction, since all bits are 0.
                    return Err(Exception::IllegalInstruction(inst16));
                }
                inst = inst16;
                self.execute_compressed(inst)?;
                // Add 2 bytes to the program counter.
                self.pc += 2;
            }
            _ => {
                inst = self.fetch(WORD)?;
                self.execute_general(inst)?;
                // Add 4 bytes to the program counter.
                self.pc += 4;
            }
        }
        self.pre_inst = inst;
        Ok(inst)
    }

    /// Execute a compressed instruction. Raised an exception if something is wrong, otherwise,
    /// returns a fetched instruction. It also increments the program counter by 2 bytes.
    pub fn execute_compressed(&mut self, inst: u64) -> Result<(), Exception> {
        execute_compressed_inner(self, inst)
    }

    /// Execute a general-purpose instruction. Raises an exception if something is wrong,
    /// otherwise, returns a fetched instruction. It also increments the program counter by 4 bytes.
    fn execute_general(&mut self, inst: u64) -> Result<(), Exception> {
        execute_general_inner(self, inst)
    }
}

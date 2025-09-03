//! CPU module for the RV64 emulator.
use crate::{
    bus::Bus,
    devices::{
        uart_cli::UART_IRQ,
        virtio_blk::{Virtio, VIRTIO_IRQ},
    },
    primitives::constants::{Mode, NUM_REGISTERS},
    reg::{
        csr::{
            state::State, MEIP_BIT, MIE, MIP, MSIP_BIT, MSTATUS_MIE, MTIP_BIT, SEIP_BIT, SSIP_BIT,
            STIP_BIT, XSTATUS_SIE,
        },
        f_reg::FloatRegister,
        i_reg::IntRegister,
    },
};
use std::collections::BTreeMap;

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
        for i in 0..NUM_REGISTERS {
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
}

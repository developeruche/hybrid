//! Control and status registers
use core::ops::{Bound, Range, RangeBounds, RangeInclusive};

pub mod state;
pub mod utils;

pub type CsrAddress = u16;
pub type CsrFieldRange = RangeInclusive<usize>;

pub const MXLEN: usize = 64;
/// The number of CSRs. The field is 12 bits so the maximum kind of CSRs is 4096 (2**12).
pub const CSR_SIZE: usize = 4096;

// ==========================
// User-level CSR addresses
// ==========================

// User trap setup.
/// User status register.
pub const USTATUS: CsrAddress = 0x000;
/// User trap handler base address.
pub const UTVEC: CsrAddress = 0x005;

// User trap handling.
/// User exception program counter.
pub const UEPC: CsrAddress = 0x041;
/// User trap cause.
pub const UCAUSE: CsrAddress = 0x042;
/// User bad address or instruction.
pub const _UTVAL: CsrAddress = 0x043;

// User floating-point CSRs.
/// Flating-point accrued exceptions.
pub const _FFLAGS: CsrAddress = 0x001;
/// Floating-point dynamic rounding mode.
pub const _FRB: CsrAddress = 0x002;
/// Floating-point control and status register (frm + fflags).
pub const FCSR: CsrAddress = 0x003;

// User Counter/Timers.
/// Timer for RDTIME instruction.
pub const TIME: CsrAddress = 0xc01;

// =================================
// Supervisor-level CSR addresses
// =================================

// Supervisor trap setup.
/// Supervisor status register.
pub const SSTATUS: CsrAddress = 0x100;
/// Supervisor exception delegation register.
pub const SEDELEG: CsrAddress = 0x102;
/// Supervisor interrupt delegation register.
pub const SIDELEG: CsrAddress = 0x103;
/// Supervisor interrupt-enable register.
pub const SIE: CsrAddress = 0x104;
/// Supervisor trap handler base address.
pub const STVEC: CsrAddress = 0x105;

// Supervisor trap handling.
/// Scratch register for supervisor trap handlers.
pub const _SSCRATCH: CsrAddress = 0x140;
/// Supervisor exception program counter.
pub const SEPC: CsrAddress = 0x141;
/// Supervisor trap cause.
pub const SCAUSE: CsrAddress = 0x142;
/// Supervisor bad address or instruction.
pub const STVAL: CsrAddress = 0x143;
/// Supervisor interrupt pending.
pub const SIP: CsrAddress = 0x144;

// Supervisor protection and translation.
/// Supervisor address translation and protection.
pub const SATP: CsrAddress = 0x180;

// SSTATUS fields.
pub const SSTATUS_SIE_MASK: u64 = 0x2; // sstatus[1]
pub const SSTATUS_SPIE_MASK: u64 = 0x20; // sstatus[5]
pub const SSTATUS_UBE_MASK: u64 = 0x40; // sstatus[6]
pub const SSTATUS_SPP_MASK: u64 = 0x100; // sstatus[8]
pub const SSTATUS_FS_MASK: u64 = 0x6000; // sstatus[14:13]
pub const SSTATUS_XS_MASK: u64 = 0x18000; // sstatus[16:15]
pub const SSTATUS_SUM_MASK: u64 = 0x40000; // sstatus[18]
pub const SSTATUS_MXR_MASK: u64 = 0x80000; // sstatus[19]
pub const SSTATUS_UXL_MASK: u64 = 0x3_00000000; // sstatus[33:32]
pub const SSTATUS_SD_MASK: u64 = 0x80000000_00000000; // sstatus[63]
pub const SSTATUS_MASK: u64 = SSTATUS_SIE_MASK
    | SSTATUS_SPIE_MASK
    | SSTATUS_UBE_MASK
    | SSTATUS_SPP_MASK
    | SSTATUS_FS_MASK
    | SSTATUS_XS_MASK
    | SSTATUS_SUM_MASK
    | SSTATUS_MXR_MASK
    | SSTATUS_UXL_MASK
    | SSTATUS_SD_MASK;
/// Global interrupt-enable bit for supervisor mode.
pub const XSTATUS_SIE: CsrFieldRange = 1..=1;
/// Previous interrupt-enable bit for supervisor mode.
pub const XSTATUS_SPIE: CsrFieldRange = 5..=5;
/// Previous privilege mode for supervisor mode.
pub const XSTATUS_SPP: CsrFieldRange = 8..=8;

// =============================
// Machine-level CSR addresses
// =============================

// Machine information registers.
/// Vendor ID.
pub const MVENDORID: CsrAddress = 0xf11;
/// Architecture ID.
pub const MARCHID: CsrAddress = 0xf12;
/// Implementation ID.
pub const MIMPID: CsrAddress = 0xf13;
/// Hardware thread ID.
pub const MHARTID: CsrAddress = 0xf14;

// Machine trap setup.
/// Machine status register.
pub const MSTATUS: CsrAddress = 0x300;
/// ISA and extensions.
pub const MISA: CsrAddress = 0x301;
/// Machine exception delefation register.
pub const MEDELEG: CsrAddress = 0x302;
/// Machine interrupt delefation register.
pub const MIDELEG: CsrAddress = 0x303;
/// Machine interrupt-enable register.
pub const MIE: CsrAddress = 0x304;
/// Machine trap-handler base address.
pub const MTVEC: CsrAddress = 0x305;
/// Machine counter enable.
pub const _MCOUNTEREN: CsrAddress = 0x306;

// Machine trap handling.
/// Scratch register for machine trap handlers.
pub const _MSCRATCH: CsrAddress = 0x340;
/// Machine exception program counter.
pub const MEPC: CsrAddress = 0x341;
/// Machine trap cause.
pub const MCAUSE: CsrAddress = 0x342;
/// Machine bad address or instruction.
pub const MTVAL: CsrAddress = 0x343;
/// Machine interrupt pending.
pub const MIP: CsrAddress = 0x344;

// Machine memory protection.
/// Physical memory protection configuration.
pub const _PMPCFG0: CsrAddress = 0x3a0;
/// Physical memory protection address register.
pub const _PMPADDR0: CsrAddress = 0x3b0;

// MSTATUS fields.
/// Global interrupt-enable bit for machine mode.
pub const MSTATUS_MIE: CsrFieldRange = 3..=3;
/// Previous interrupt-enable bit for machine mode.
pub const MSTATUS_MPIE: CsrFieldRange = 7..=7;
/// Previous privilege mode for machine mode.
pub const MSTATUS_MPP: CsrFieldRange = 11..=12;
/// Modify privilege bit.
pub const MSTATUS_MPRV: CsrFieldRange = 17..=17;

// MIP fields.
/// Supervisor software interrupt.
pub const SSIP_BIT: u64 = 1 << 1;
/// Machine software interrupt.
pub const MSIP_BIT: u64 = 1 << 3;
/// Supervisor timer interrupt.
pub const STIP_BIT: u64 = 1 << 5;
/// Machine timer interrupt.
pub const MTIP_BIT: u64 = 1 << 7;
/// Supervisor external interrupt.
pub const SEIP_BIT: u64 = 1 << 9;
/// Machine external interrupt.
pub const MEIP_BIT: u64 = 1 << 11;

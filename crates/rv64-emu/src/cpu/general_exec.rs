use crate::cpu::{Mode, BYTE, DOUBLEWORD, HALFWORD, WORD};
use crate::reg::csr::{
    MEPC, MSTATUS_MIE, MSTATUS_MPIE, MSTATUS_MPP, MSTATUS_MPRV, SATP, SEPC, XSTATUS_SIE,
    XSTATUS_SPIE, XSTATUS_SPP,
};
use crate::{cpu::CPU, exception::Exception, reg::csr::FCSR};
use std::cmp;
use std::num::FpCategory;

/// Execute a general-purpose instruction. Raises an exception if something is wrong,
/// otherwise, returns a fetched instruction. It also increments the program counter by 4 bytes.
pub fn execute_general_inner(cpu: &mut CPU, inst: u64) -> Result<(), Exception> {
    // 2. Decode.
    let opcode = inst & 0x0000007f;
    let rd = (inst & 0x00000f80) >> 7;
    let rs1 = (inst & 0x000f8000) >> 15;
    let rs2 = (inst & 0x01f00000) >> 20;
    let funct3 = (inst & 0x00007000) >> 12;
    let funct7 = (inst & 0xfe000000) >> 25;

    // 3. Execute.
    match opcode {
        0x03 => {
            // RV32I and RV64I
            // imm[11:0] = inst[31:20]
            let offset = ((inst as i32 as i64) >> 20) as u64;
            let addr = cpu.int_regs.read(rs1).wrapping_add(offset);
            match funct3 {
                0x0 => {
                    // lb

                    let val = cpu.read(addr, BYTE)?;
                    cpu.int_regs.write(rd, val as i8 as i64 as u64);
                }
                0x1 => {
                    // lh

                    let val = cpu.read(addr, HALFWORD)?;
                    cpu.int_regs.write(rd, val as i16 as i64 as u64);
                }
                0x2 => {
                    // lw

                    let val = cpu.read(addr, WORD)?;
                    cpu.int_regs.write(rd, val as i32 as i64 as u64);
                }
                0x3 => {
                    // ld

                    let val = cpu.read(addr, DOUBLEWORD)?;
                    cpu.int_regs.write(rd, val);
                }
                0x4 => {
                    // lbu

                    let val = cpu.read(addr, BYTE)?;
                    cpu.int_regs.write(rd, val);
                }
                0x5 => {
                    // lhu

                    let val = cpu.read(addr, HALFWORD)?;
                    cpu.int_regs.write(rd, val);
                }
                0x6 => {
                    // lwu

                    let val = cpu.read(addr, WORD)?;
                    cpu.int_regs.write(rd, val);
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }
        }
        0x07 => {
            // RV32D and RV64D
            // imm[11:0] = inst[31:20]
            let offset = ((inst as i32 as i64) >> 20) as u64;
            let addr = cpu.int_regs.read(rs1).wrapping_add(offset);
            match funct3 {
                0x2 => {
                    // flw

                    let val = f32::from_bits(cpu.read(addr, WORD)? as u32);
                    cpu.float_regs.write(rd, val as f64);
                }
                0x3 => {
                    // fld

                    let val = f64::from_bits(cpu.read(addr, DOUBLEWORD)?);
                    cpu.float_regs.write(rd, val);
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }
        }
        0x0f => {
            // RV32I and RV64I
            // fence instructions are not supported yet because this emulator executes an
            // instruction sequentially on a single thread.
            // fence.i is a part of the Zifencei extension.
            match funct3 {
                0x0 => {
                    // fence
                }
                0x1 => {
                    // fence.i
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }
        }
        0x13 => {
            // RV32I and RV64I
            // imm[11:0] = inst[31:20]
            let imm = ((inst as i32 as i64) >> 20) as u64;
            let funct6 = funct7 >> 1;
            match funct3 {
                0x0 => {
                    // addi

                    cpu.int_regs
                        .write(rd, cpu.int_regs.read(rs1).wrapping_add(imm));
                }
                0x1 => {
                    // slli

                    // shamt size is 5 bits for RV32I and 6 bits for RV64I.
                    let shamt = (inst >> 20) & 0x3f;
                    cpu.int_regs.write(rd, cpu.int_regs.read(rs1) << shamt);
                }
                0x2 => {
                    // slti

                    cpu.int_regs.write(
                        rd,
                        if (cpu.int_regs.read(rs1) as i64) < (imm as i64) {
                            1
                        } else {
                            0
                        },
                    );
                }
                0x3 => {
                    // sltiu

                    cpu.int_regs
                        .write(rd, if cpu.int_regs.read(rs1) < imm { 1 } else { 0 });
                }
                0x4 => {
                    // xori

                    cpu.int_regs.write(rd, cpu.int_regs.read(rs1) ^ imm);
                }
                0x5 => {
                    match funct6 {
                        0x00 => {
                            // srli

                            // shamt size is 5 bits for RV32I and 6 bits for RV64I.
                            let shamt = (inst >> 20) & 0x3f;
                            cpu.int_regs.write(rd, cpu.int_regs.read(rs1) >> shamt);
                        }
                        0x10 => {
                            // srai

                            // shamt size is 5 bits for RV32I and 6 bits for RV64I.
                            let shamt = (inst >> 20) & 0x3f;
                            cpu.int_regs
                                .write(rd, ((cpu.int_regs.read(rs1) as i64) >> shamt) as u64);
                        }
                        _ => {
                            return Err(Exception::IllegalInstruction(inst));
                        }
                    }
                }
                0x6 => {
                    // ori

                    cpu.int_regs.write(rd, cpu.int_regs.read(rs1) | imm);
                }
                0x7 => {
                    // andi

                    cpu.int_regs.write(rd, cpu.int_regs.read(rs1) & imm);
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }
        }
        0x17 => {
            // RV32I
            // auipc

            // AUIPC forms a 32-bit offset from the 20-bit U-immediate, filling
            // in the lowest 12 bits with zeros.
            // imm[31:12] = inst[31:12]
            let imm = (inst & 0xfffff000) as i32 as i64 as u64;
            cpu.int_regs.write(rd, cpu.pc.wrapping_add(imm));
        }
        0x1b => {
            // RV64I
            // imm[11:0] = inst[31:20]
            let imm = ((inst as i32 as i64) >> 20) as u64;
            match funct3 {
                0x0 => {
                    // addiw

                    cpu.int_regs.write(
                        rd,
                        cpu.int_regs.read(rs1).wrapping_add(imm) as i32 as i64 as u64,
                    );
                }
                0x1 => {
                    // slliw

                    // "SLLIW, SRLIW, and SRAIW encodings with imm[5] ̸= 0 are reserved."
                    let shamt = (imm & 0x1f) as u32;
                    cpu.int_regs
                        .write(rd, (cpu.int_regs.read(rs1) << shamt) as i32 as i64 as u64);
                }
                0x5 => {
                    match funct7 {
                        0x00 => {
                            // srliw

                            // "SLLIW, SRLIW, and SRAIW encodings with imm[5] ̸= 0 are reserved."
                            let shamt = (imm & 0x1f) as u32;
                            cpu.int_regs.write(
                                rd,
                                ((cpu.int_regs.read(rs1) as u32) >> shamt) as i32 as i64 as u64,
                            )
                        }
                        0x20 => {
                            // sraiw

                            // "SLLIW, SRLIW, and SRAIW encodings with imm[5] ̸= 0 are reserved."
                            let shamt = (imm & 0x1f) as u32;
                            cpu.int_regs.write(
                                rd,
                                ((cpu.int_regs.read(rs1) as i32) >> shamt) as i64 as u64,
                            );
                        }
                        _ => {
                            return Err(Exception::IllegalInstruction(inst));
                        }
                    }
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }
        }
        0x23 => {
            // RV32I
            // offset[11:5|4:0] = inst[31:25|11:7]
            let offset = (((inst & 0xfe000000) as i32 as i64 >> 20) as u64) | ((inst >> 7) & 0x1f);
            let addr = cpu.int_regs.read(rs1).wrapping_add(offset);
            match funct3 {
                0x0 => {
                    // sb

                    cpu.write(addr, cpu.int_regs.read(rs2), BYTE)?
                }
                0x1 => {
                    // sh

                    cpu.write(addr, cpu.int_regs.read(rs2), HALFWORD)?
                }
                0x2 => {
                    // sw

                    cpu.write(addr, cpu.int_regs.read(rs2), WORD)?
                }
                0x3 => {
                    // sd

                    cpu.write(addr, cpu.int_regs.read(rs2), DOUBLEWORD)?
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }
        }
        0x27 => {
            // RV32F and RV64F
            // offset[11:5|4:0] = inst[31:25|11:7]
            let offset = ((((inst as i32 as i64) >> 20) as u64) & 0xfe0) | ((inst >> 7) & 0x1f);
            let addr = cpu.int_regs.read(rs1).wrapping_add(offset);
            match funct3 {
                0x2 => {
                    // fsw

                    cpu.write(
                        addr,
                        (cpu.float_regs.read(rs2) as f32).to_bits() as u64,
                        WORD,
                    )?
                }
                0x3 => {
                    // fsd

                    cpu.write(addr, cpu.float_regs.read(rs2).to_bits() as u64, DOUBLEWORD)?
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }
        }
        0x2f => {
            // RV32A and RV64A
            let funct5 = (funct7 & 0b1111100) >> 2;
            // TODO: Handle `aq` and `rl`.
            let _aq = (funct7 & 0b0000010) >> 1; // acquire access
            let _rl = funct7 & 0b0000001; // release access
            match (funct3, funct5) {
                (0x2, 0x00) => {
                    // amoadd.w

                    let addr = cpu.int_regs.read(rs1);
                    // "For AMOs, the A extension requires that the address held in rs1 be
                    // naturally aligned to the size of the operand (i.e., eight-byte aligned
                    // for 64-bit words and four-byte aligned for 32-bit words). If the
                    // address is not naturally aligned, an address-misaligned exception or
                    // an access-fault exception will be generated."
                    if addr % 4 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let t = cpu.read(addr, WORD)?;
                    cpu.write(addr, t.wrapping_add(cpu.int_regs.read(rs2)), WORD)?;
                    cpu.int_regs.write(rd, t as i32 as i64 as u64);
                }
                (0x3, 0x00) => {
                    // amoadd.d

                    let addr = cpu.int_regs.read(rs1);
                    if addr % 8 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let t = cpu.read(addr, DOUBLEWORD)?;
                    cpu.write(addr, t.wrapping_add(cpu.int_regs.read(rs2)), DOUBLEWORD)?;
                    cpu.int_regs.write(rd, t);
                }
                (0x2, 0x01) => {
                    // amoswap.w

                    let addr = cpu.int_regs.read(rs1);
                    if addr % 4 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let t = cpu.read(addr, WORD)?;
                    cpu.write(addr, cpu.int_regs.read(rs2), WORD)?;
                    cpu.int_regs.write(rd, t as i32 as i64 as u64);
                }
                (0x3, 0x01) => {
                    // amoswap.d

                    let addr = cpu.int_regs.read(rs1);
                    if addr % 8 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let t = cpu.read(addr, DOUBLEWORD)?;
                    cpu.write(addr, cpu.int_regs.read(rs2), DOUBLEWORD)?;
                    cpu.int_regs.write(rd, t);
                }
                (0x2, 0x02) => {
                    // lr.w

                    let addr = cpu.int_regs.read(rs1);
                    // "For LR and SC, the A extension requires that the address held in rs1 be
                    // naturally aligned to the size of the operand (i.e., eight-byte aligned
                    // for 64-bit words and four-byte aligned for 32-bit words)."
                    if addr % 4 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let value = cpu.read(addr, WORD)?;
                    cpu.int_regs.write(rd, value as i32 as i64 as u64);
                    cpu.reservation_set.push(addr);
                }
                (0x3, 0x02) => {
                    // lr.d

                    let addr = cpu.int_regs.read(rs1);
                    // "For LR and SC, the A extension requires that the address held in rs1 be
                    // naturally aligned to the size of the operand (i.e., eight-byte aligned for
                    // 64-bit words and four-byte aligned for 32-bit words)."
                    if addr % 8 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let value = cpu.read(addr, DOUBLEWORD)?;
                    cpu.int_regs.write(rd, value);
                    cpu.reservation_set.push(addr);
                }
                (0x2, 0x03) => {
                    // sc.w

                    let addr = cpu.int_regs.read(rs1);
                    // "For LR and SC, the A extension requires that the address held in rs1 be
                    // naturally aligned to the size of the operand (i.e., eight-byte aligned for
                    // 64-bit words and four-byte aligned for 32-bit words)."
                    if addr % 4 != 0 {
                        return Err(Exception::StoreAMOAddressMisaligned);
                    }
                    if cpu.reservation_set.contains(&addr) {
                        // "Regardless of success or failure, executing an SC.W instruction
                        // invalidates any reservation held by this hart. "
                        cpu.reservation_set.retain(|&x| x != addr);
                        cpu.write(addr, cpu.int_regs.read(rs2), WORD)?;
                        cpu.int_regs.write(rd, 0);
                    } else {
                        cpu.reservation_set.retain(|&x| x != addr);
                        cpu.int_regs.write(rd, 1);
                    };
                }
                (0x3, 0x03) => {
                    // sc.d

                    let addr = cpu.int_regs.read(rs1);
                    // "For LR and SC, the A extension requires that the address held in rs1 be
                    // naturally aligned to the size of the operand (i.e., eight-byte aligned for
                    // 64-bit words and four-byte aligned for 32-bit words)."
                    if addr % 8 != 0 {
                        return Err(Exception::StoreAMOAddressMisaligned);
                    }
                    if cpu.reservation_set.contains(&addr) {
                        cpu.reservation_set.retain(|&x| x != addr);
                        cpu.write(addr, cpu.int_regs.read(rs2), DOUBLEWORD)?;
                        cpu.int_regs.write(rd, 0);
                    } else {
                        cpu.reservation_set.retain(|&x| x != addr);
                        cpu.int_regs.write(rd, 1);
                    }
                }
                (0x2, 0x04) => {
                    // amoxor.w

                    let addr = cpu.int_regs.read(rs1);
                    if addr % 4 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let t = cpu.read(addr, WORD)?;
                    cpu.write(
                        addr,
                        (t as i32 ^ (cpu.int_regs.read(rs2) as i32)) as i64 as u64,
                        WORD,
                    )?;
                    cpu.int_regs.write(rd, t as i32 as i64 as u64);
                }
                (0x3, 0x04) => {
                    // amoxor.d

                    let addr = cpu.int_regs.read(rs1);
                    if addr % 8 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let t = cpu.read(addr, DOUBLEWORD)?;
                    cpu.write(addr, t ^ cpu.int_regs.read(rs2), DOUBLEWORD)?;
                    cpu.int_regs.write(rd, t);
                }
                (0x2, 0x08) => {
                    // amoor.w

                    let addr = cpu.int_regs.read(rs1);
                    if addr % 4 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let t = cpu.read(addr, WORD)?;
                    cpu.write(
                        addr,
                        (t as i32 | (cpu.int_regs.read(rs2) as i32)) as i64 as u64,
                        WORD,
                    )?;
                    cpu.int_regs.write(rd, t as i32 as i64 as u64);
                }
                (0x3, 0x08) => {
                    // amoor.d

                    let addr = cpu.int_regs.read(rs1);
                    if addr % 8 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let t = cpu.read(addr, DOUBLEWORD)?;
                    cpu.write(addr, t | cpu.int_regs.read(rs2), DOUBLEWORD)?;
                    cpu.int_regs.write(rd, t);
                }
                (0x2, 0x0c) => {
                    // amoand.w

                    let addr = cpu.int_regs.read(rs1);
                    if addr % 4 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let t = cpu.read(addr, WORD)?;
                    cpu.write(
                        addr,
                        (t as i32 & (cpu.int_regs.read(rs2) as i32)) as u32 as u64,
                        WORD,
                    )?;
                    cpu.int_regs.write(rd, t as i32 as i64 as u64);
                }
                (0x3, 0x0c) => {
                    // amoand.d

                    let addr = cpu.int_regs.read(rs1);
                    if addr % 8 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let t = cpu.read(addr, DOUBLEWORD)?;
                    cpu.write(addr, t & cpu.int_regs.read(rs2), DOUBLEWORD)?;
                    cpu.int_regs.write(rd, t);
                }
                (0x2, 0x10) => {
                    // amomin.w

                    let addr = cpu.int_regs.read(rs1);
                    if addr % 4 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let t = cpu.read(addr, WORD)?;
                    cpu.write(
                        addr,
                        cmp::min(t as i32, cpu.int_regs.read(rs2) as i32) as i64 as u64,
                        WORD,
                    )?;
                    cpu.int_regs.write(rd, t as i32 as i64 as u64);
                }
                (0x3, 0x10) => {
                    // amomin.d

                    let addr = cpu.int_regs.read(rs1);
                    if addr % 8 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let t = cpu.read(addr, DOUBLEWORD)?;
                    cpu.write(
                        addr,
                        cmp::min(t as i64, cpu.int_regs.read(rs2) as i64) as u64,
                        DOUBLEWORD,
                    )?;
                    cpu.int_regs.write(rd, t as u64);
                }
                (0x2, 0x14) => {
                    // amomax.w

                    let addr = cpu.int_regs.read(rs1);
                    if addr % 4 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let t = cpu.read(addr, WORD)?;
                    cpu.write(
                        addr,
                        cmp::max(t as i32, cpu.int_regs.read(rs2) as i32) as i64 as u64,
                        WORD,
                    )?;
                    cpu.int_regs.write(rd, t as i32 as i64 as u64);
                }
                (0x3, 0x14) => {
                    // amomax.d

                    let addr = cpu.int_regs.read(rs1);
                    if addr % 8 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let t = cpu.read(addr, DOUBLEWORD)?;
                    cpu.write(
                        addr,
                        cmp::max(t as i64, cpu.int_regs.read(rs2) as i64) as u64,
                        DOUBLEWORD,
                    )?;
                    cpu.int_regs.write(rd, t);
                }
                (0x2, 0x18) => {
                    // amominu.w

                    let addr = cpu.int_regs.read(rs1);
                    if addr % 4 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let t = cpu.read(addr, WORD)?;
                    cpu.write(
                        addr,
                        cmp::min(t as u32, cpu.int_regs.read(rs2) as u32) as u64,
                        WORD,
                    )?;
                    cpu.int_regs.write(rd, t as i32 as i64 as u64);
                }
                (0x3, 0x18) => {
                    // amominu.d

                    let addr = cpu.int_regs.read(rs1);
                    if addr % 8 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let t = cpu.read(addr, DOUBLEWORD)?;
                    cpu.write(addr, cmp::min(t, cpu.int_regs.read(rs2)), DOUBLEWORD)?;
                    cpu.int_regs.write(rd, t);
                }
                (0x2, 0x1c) => {
                    // amomaxu.w

                    let addr = cpu.int_regs.read(rs1);
                    if addr % 4 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let t = cpu.read(addr, WORD)?;
                    cpu.write(
                        addr,
                        cmp::max(t as u32, cpu.int_regs.read(rs2) as u32) as u64,
                        WORD,
                    )?;
                    cpu.int_regs.write(rd, t as i32 as i64 as u64);
                }
                (0x3, 0x1c) => {
                    // amomaxu.d

                    let addr = cpu.int_regs.read(rs1);
                    if addr % 8 != 0 {
                        return Err(Exception::LoadAddressMisaligned);
                    }
                    let t = cpu.read(addr, DOUBLEWORD)?;
                    cpu.write(addr, cmp::max(t, cpu.int_regs.read(rs2)), DOUBLEWORD)?;
                    cpu.int_regs.write(rd, t);
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }
        }
        0x33 => {
            // RV64I and RV64M
            match (funct3, funct7) {
                (0x0, 0x00) => {
                    // add

                    cpu.int_regs.write(
                        rd,
                        cpu.int_regs.read(rs1).wrapping_add(cpu.int_regs.read(rs2)),
                    );
                }
                (0x0, 0x01) => {
                    // mul

                    cpu.int_regs.write(
                        rd,
                        (cpu.int_regs.read(rs1) as i64).wrapping_mul(cpu.int_regs.read(rs2) as i64)
                            as u64,
                    );
                }
                (0x0, 0x20) => {
                    // sub

                    cpu.int_regs.write(
                        rd,
                        cpu.int_regs.read(rs1).wrapping_sub(cpu.int_regs.read(rs2)),
                    );
                }
                (0x1, 0x00) => {
                    // sll

                    // "SLL, SRL, and SRA perform logical left, logical right, and arithmetic
                    // right shifts on the value in register rs1 by the shift amount held in
                    // register rs2. In RV64I, only the low 6 bits of rs2 are considered for the
                    // shift amount."
                    let shamt = cpu.int_regs.read(rs2) & 0x3f;
                    cpu.int_regs.write(rd, cpu.int_regs.read(rs1) << shamt);
                }
                (0x1, 0x01) => {
                    // mulh

                    // signed × signed
                    cpu.int_regs.write(
                        rd,
                        ((cpu.int_regs.read(rs1) as i64 as i128)
                            .wrapping_mul(cpu.int_regs.read(rs2) as i64 as i128)
                            >> 64) as u64,
                    );
                }
                (0x2, 0x00) => {
                    // slt

                    cpu.int_regs.write(
                        rd,
                        if (cpu.int_regs.read(rs1) as i64) < (cpu.int_regs.read(rs2) as i64) {
                            1
                        } else {
                            0
                        },
                    );
                }
                (0x2, 0x01) => {
                    // mulhsu

                    // signed × unsigned
                    cpu.int_regs.write(
                        rd,
                        ((cpu.int_regs.read(rs1) as i64 as i128 as u128)
                            .wrapping_mul(cpu.int_regs.read(rs2) as u128)
                            >> 64) as u64,
                    );
                }
                (0x3, 0x00) => {
                    // sltu

                    cpu.int_regs.write(
                        rd,
                        if cpu.int_regs.read(rs1) < cpu.int_regs.read(rs2) {
                            1
                        } else {
                            0
                        },
                    );
                }
                (0x3, 0x01) => {
                    // mulhu

                    // unsigned × unsigned
                    cpu.int_regs.write(
                        rd,
                        ((cpu.int_regs.read(rs1) as u128)
                            .wrapping_mul(cpu.int_regs.read(rs2) as u128)
                            >> 64) as u64,
                    );
                }
                (0x4, 0x00) => {
                    // xor

                    cpu.int_regs
                        .write(rd, cpu.int_regs.read(rs1) ^ cpu.int_regs.read(rs2));
                }
                (0x4, 0x01) => {
                    // div

                    let dividend = cpu.int_regs.read(rs1) as i64;
                    let divisor = cpu.int_regs.read(rs2) as i64;
                    cpu.int_regs.write(
                        rd,
                        if divisor == 0 {
                            // Division by zero
                            // Set DZ (Divide by Zero) flag to 1.
                            cpu.state.write_bit(FCSR, 3, 1);
                            // "The quotient of division by zero has all bits set"
                            u64::MAX
                        } else if dividend == i64::MIN && divisor == -1 {
                            // Overflow
                            // "The quotient of a signed division with overflow is equal to the
                            // dividend"
                            dividend as u64
                        } else {
                            // "division of rs1 by rs2, rounding towards zero"
                            dividend.wrapping_div(divisor) as u64
                        },
                    );
                }
                (0x5, 0x00) => {
                    // srl

                    // "SLL, SRL, and SRA perform logical left, logical right, and arithmetic
                    // right shifts on the value in register rs1 by the shift amount held in
                    // register rs2. In RV64I, only the low 6 bits of rs2 are considered for the
                    // shift amount."
                    let shamt = cpu.int_regs.read(rs2) & 0x3f;
                    cpu.int_regs.write(rd, cpu.int_regs.read(rs1) >> shamt);
                }
                (0x5, 0x01) => {
                    // divu

                    let dividend = cpu.int_regs.read(rs1);
                    let divisor = cpu.int_regs.read(rs2);
                    cpu.int_regs.write(
                        rd,
                        if divisor == 0 {
                            // Division by zero
                            // Set DZ (Divide by Zero) flag to 1.
                            cpu.state.write_bit(FCSR, 3, 1);
                            // "The quotient of division by zero has all bits set"
                            u64::MAX
                        } else {
                            // "division of rs1 by rs2, rounding towards zero"
                            dividend.wrapping_div(divisor)
                        },
                    );
                }
                (0x5, 0x20) => {
                    // sra

                    // "SLL, SRL, and SRA perform logical left, logical right, and arithmetic
                    // right shifts on the value in register rs1 by the shift amount held in
                    // register rs2. In RV64I, only the low 6 bits of rs2 are considered for the
                    // shift amount."
                    let shamt = cpu.int_regs.read(rs2) & 0x3f;
                    cpu.int_regs
                        .write(rd, ((cpu.int_regs.read(rs1) as i64) >> shamt) as u64);
                }
                (0x6, 0x00) => {
                    // or

                    cpu.int_regs
                        .write(rd, cpu.int_regs.read(rs1) | cpu.int_regs.read(rs2));
                }
                (0x6, 0x01) => {
                    // rem

                    let dividend = cpu.int_regs.read(rs1) as i64;
                    let divisor = cpu.int_regs.read(rs2) as i64;
                    cpu.int_regs.write(
                        rd,
                        if divisor == 0 {
                            // Division by zero
                            // "the remainder of division by zero equals the dividend"
                            dividend as u64
                        } else if dividend == i64::MIN && divisor == -1 {
                            // Overflow
                            // "the remainder is zero"
                            0
                        } else {
                            // "provide the remainder of the corresponding division
                            // operation"
                            dividend.wrapping_rem(divisor) as u64
                        },
                    );
                }
                (0x7, 0x00) => {
                    // and

                    cpu.int_regs
                        .write(rd, cpu.int_regs.read(rs1) & cpu.int_regs.read(rs2));
                }
                (0x7, 0x01) => {
                    // remu

                    let dividend = cpu.int_regs.read(rs1);
                    let divisor = cpu.int_regs.read(rs2);
                    cpu.int_regs.write(
                        rd,
                        if divisor == 0 {
                            // Division by zero
                            // "the remainder of division by zero equals the dividend"
                            dividend
                        } else {
                            // "provide the remainder of the corresponding division
                            // operation"
                            dividend.wrapping_rem(divisor)
                        },
                    );
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            };
        }
        0x37 => {
            // RV32I
            // lui

            // "LUI places the U-immediate value in the top 20 bits of the destination
            // register rd, filling in the lowest 12 bits with zeros."
            cpu.int_regs
                .write(rd, (inst & 0xfffff000) as i32 as i64 as u64);
        }
        0x3b => {
            // RV64I and RV64M
            match (funct3, funct7) {
                (0x0, 0x00) => {
                    // addw

                    cpu.int_regs.write(
                        rd,
                        cpu.int_regs.read(rs1).wrapping_add(cpu.int_regs.read(rs2)) as i32 as i64
                            as u64,
                    );
                }
                (0x0, 0x01) => {
                    // mulw

                    let n1 = cpu.int_regs.read(rs1) as i32;
                    let n2 = cpu.int_regs.read(rs2) as i32;
                    let result = n1.wrapping_mul(n2);
                    cpu.int_regs.write(rd, result as i64 as u64);
                }
                (0x0, 0x20) => {
                    // subw

                    cpu.int_regs.write(
                        rd,
                        ((cpu.int_regs.read(rs1).wrapping_sub(cpu.int_regs.read(rs2))) as i32)
                            as u64,
                    );
                }
                (0x1, 0x00) => {
                    // sllw

                    // The shift amount is given by rs2[4:0].
                    let shamt = cpu.int_regs.read(rs2) & 0x1f;
                    cpu.int_regs
                        .write(rd, ((cpu.int_regs.read(rs1)) << shamt) as i32 as i64 as u64);
                }
                (0x4, 0x01) => {
                    // divw

                    let dividend = cpu.int_regs.read(rs1) as i32;
                    let divisor = cpu.int_regs.read(rs2) as i32;
                    cpu.int_regs.write(
                        rd,
                        if divisor == 0 {
                            // Division by zero
                            // Set DZ (Divide by Zero) flag to 1.
                            cpu.state.write_bit(FCSR, 3, 1);
                            // "The quotient of division by zero has all bits set"
                            u64::MAX
                        } else if dividend == i32::MIN && divisor == -1 {
                            // Overflow
                            // "The quotient of a signed division with overflow is equal to the
                            // dividend"
                            dividend as i64 as u64
                        } else {
                            // "division of rs1 by rs2, rounding towards zero"
                            dividend.wrapping_div(divisor) as i64 as u64
                        },
                    );
                }
                (0x5, 0x00) => {
                    // srlw

                    // The shift amount is given by rs2[4:0].
                    let shamt = cpu.int_regs.read(rs2) & 0x1f;
                    cpu.int_regs.write(
                        rd,
                        ((cpu.int_regs.read(rs1) as u32) >> shamt) as i32 as i64 as u64,
                    );
                }
                (0x5, 0x01) => {
                    // divuw

                    let dividend = cpu.int_regs.read(rs1) as u32;
                    let divisor = cpu.int_regs.read(rs2) as u32;
                    cpu.int_regs.write(
                        rd,
                        if divisor == 0 {
                            // Division by zero
                            // Set DZ (Divide by Zero) flag to 1.
                            cpu.state.write_bit(FCSR, 3, 1);
                            // "The quotient of division by zero has all bits set"
                            u64::MAX
                        } else {
                            // "division of rs1 by rs2, rounding towards zero"
                            dividend.wrapping_div(divisor) as i32 as i64 as u64
                        },
                    );
                }
                (0x5, 0x20) => {
                    // sraw

                    // The shift amount is given by rs2[4:0].
                    let shamt = cpu.int_regs.read(rs2) & 0x1f;
                    cpu.int_regs
                        .write(rd, ((cpu.int_regs.read(rs1) as i32) >> shamt) as i64 as u64);
                }
                (0x6, 0x01) => {
                    // remw

                    let dividend = cpu.int_regs.read(rs1) as i32;
                    let divisor = cpu.int_regs.read(rs2) as i32;
                    cpu.int_regs.write(
                        rd,
                        if divisor == 0 {
                            // Division by zero
                            // "the remainder of division by zero equals the dividend"
                            dividend as i64 as u64
                        } else if dividend == i32::MIN && divisor == -1 {
                            // Overflow
                            // "the remainder is zero"
                            0
                        } else {
                            // "provide the remainder of the corresponding division
                            // operation"
                            dividend.wrapping_rem(divisor) as i64 as u64
                        },
                    );
                }
                (0x7, 0x01) => {
                    // remuw

                    let dividend = cpu.int_regs.read(rs1) as u32;
                    let divisor = cpu.int_regs.read(rs2) as u32;
                    cpu.int_regs.write(
                        rd,
                        if divisor == 0 {
                            // Division by zero
                            // "the remainder of division by zero equals the dividend"
                            dividend as i32 as i64 as u64
                        } else {
                            // "provide the remainder of the corresponding division
                            // operation"
                            dividend.wrapping_rem(divisor) as i32 as i64 as u64
                        },
                    );
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }
        }
        0x43 => {
            // RV32F and RV64F
            // TODO: support the rounding mode encoding (rm).
            let rs3 = ((inst & 0xf8000000) >> 27) as u64;
            let funct2 = (inst & 0x03000000) >> 25;
            match funct2 {
                0x0 => {
                    // fmadd.s

                    cpu.float_regs.write(
                        rd,
                        (cpu.float_regs.read(rs1) as f32).mul_add(
                            cpu.float_regs.read(rs2) as f32,
                            cpu.float_regs.read(rs3) as f32,
                        ) as f64,
                    );
                }
                0x1 => {
                    // fmadd.d

                    cpu.float_regs.write(
                        rd,
                        cpu.float_regs
                            .read(rs1)
                            .mul_add(cpu.float_regs.read(rs2), cpu.float_regs.read(rs3)),
                    );
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }
        }
        0x47 => {
            // RV32F and RV64F
            // TODO: support the rounding mode encoding (rm).
            let rs3 = ((inst & 0xf8000000) >> 27) as u64;
            let funct2 = (inst & 0x03000000) >> 25;
            match funct2 {
                0x0 => {
                    // fmsub.s

                    cpu.float_regs.write(
                        rd,
                        (cpu.float_regs.read(rs1) as f32).mul_add(
                            cpu.float_regs.read(rs2) as f32,
                            -cpu.float_regs.read(rs3) as f32,
                        ) as f64,
                    );
                }
                0x1 => {
                    // fmsub.d

                    cpu.float_regs.write(
                        rd,
                        cpu.float_regs
                            .read(rs1)
                            .mul_add(cpu.float_regs.read(rs2), -cpu.float_regs.read(rs3)),
                    );
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }
        }
        0x4b => {
            // RV32F and RV64F
            // TODO: support the rounding mode encoding (rm).
            let rs3 = ((inst & 0xf8000000) >> 27) as u64;
            let funct2 = (inst & 0x03000000) >> 25;
            match funct2 {
                0x0 => {
                    // fnmadd.s

                    cpu.float_regs.write(
                        rd,
                        (-cpu.float_regs.read(rs1) as f32).mul_add(
                            cpu.float_regs.read(rs2) as f32,
                            cpu.float_regs.read(rs3) as f32,
                        ) as f64,
                    );
                }
                0x1 => {
                    // fnmadd.d

                    cpu.float_regs.write(
                        rd,
                        (-cpu.float_regs.read(rs1))
                            .mul_add(cpu.float_regs.read(rs2), cpu.float_regs.read(rs3)),
                    );
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }
        }
        0x4f => {
            // RV32F and RV64F
            // TODO: support the rounding mode encoding (rm).
            let rs3 = ((inst & 0xf8000000) >> 27) as u64;
            let funct2 = (inst & 0x03000000) >> 25;
            match funct2 {
                0x0 => {
                    // fnmsub.s

                    cpu.float_regs.write(
                        rd,
                        (-cpu.float_regs.read(rs1) as f32).mul_add(
                            cpu.float_regs.read(rs2) as f32,
                            -cpu.float_regs.read(rs3) as f32,
                        ) as f64,
                    );
                }
                0x1 => {
                    // fnmsub.d

                    cpu.float_regs.write(
                        rd,
                        (-cpu.float_regs.read(rs1))
                            .mul_add(cpu.float_regs.read(rs2), -cpu.float_regs.read(rs3)),
                    );
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }
        }
        0x53 => {
            // RV32F and RV64F
            // TODO: support the rounding mode encoding (rm).
            // TODO: NaN Boxing of Narrower Values (Spec 12.2).
            // TODO: set exception flags.

            /*
             * Floating-point instructions align with the IEEE 754 (1985).
             * The format consist of three fields: a sign bit, a biased exponent, and a fraction.
             *
             * | sign(1) | exponent(8) | fraction(23) |
             * Ok => {}
             * 31                                     0
             *
             */

            // Check the frm field is valid.
            match cpu.state.read_bits(FCSR, 5..8) {
                0b000 => {}
                0b001 => {}
                0b010 => {}
                0b011 => {}
                0b100 => {}
                0b111 => {}
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }

            match funct7 {
                0x00 => {
                    // fadd.s

                    cpu.float_regs.write(
                        rd,
                        (cpu.float_regs.read(rs1) as f32 + cpu.float_regs.read(rs2) as f32) as f64,
                    )
                }
                0x01 => {
                    // fadd.d

                    cpu.float_regs
                        .write(rd, cpu.float_regs.read(rs1) + cpu.float_regs.read(rs2));
                }
                0x04 => {
                    // fsub.s

                    cpu.float_regs.write(
                        rd,
                        (cpu.float_regs.read(rs1) as f32 - cpu.float_regs.read(rs2) as f32) as f64,
                    )
                }
                0x05 => {
                    // fsub.d

                    cpu.float_regs
                        .write(rd, cpu.float_regs.read(rs1) - cpu.float_regs.read(rs2));
                }
                0x08 => {
                    // fmul.s

                    cpu.float_regs.write(
                        rd,
                        (cpu.float_regs.read(rs1) as f32 * cpu.float_regs.read(rs2) as f32) as f64,
                    )
                }
                0x09 => {
                    // fmul.d

                    cpu.float_regs
                        .write(rd, cpu.float_regs.read(rs1) * cpu.float_regs.read(rs2));
                }
                0x0c => {
                    // fdiv.s

                    cpu.float_regs.write(
                        rd,
                        (cpu.float_regs.read(rs1) as f32 / cpu.float_regs.read(rs2) as f32) as f64,
                    )
                }
                0x0d => {
                    // fdiv.d

                    cpu.float_regs
                        .write(rd, cpu.float_regs.read(rs1) / cpu.float_regs.read(rs2));
                }
                0x10 => {
                    match funct3 {
                        0x0 => {
                            // fsgnj.s

                            cpu.float_regs.write(
                                rd,
                                cpu.float_regs.read(rs1).copysign(cpu.float_regs.read(rs2)),
                            );
                        }
                        0x1 => {
                            // fsgnjn.s

                            cpu.float_regs.write(
                                rd,
                                cpu.float_regs.read(rs1).copysign(-cpu.float_regs.read(rs2)),
                            );
                        }
                        0x2 => {
                            // fsgnjx.s

                            let sign1 = (cpu.float_regs.read(rs1) as f32).to_bits() & 0x80000000;
                            let sign2 = (cpu.float_regs.read(rs2) as f32).to_bits() & 0x80000000;
                            let other = (cpu.float_regs.read(rs1) as f32).to_bits() & 0x7fffffff;
                            cpu.float_regs
                                .write(rd, f32::from_bits((sign1 ^ sign2) | other) as f64);
                        }
                        _ => {
                            return Err(Exception::IllegalInstruction(inst));
                        }
                    }
                }
                0x11 => {
                    match funct3 {
                        0x0 => {
                            // fsgnj.d

                            cpu.float_regs.write(
                                rd,
                                cpu.float_regs.read(rs1).copysign(cpu.float_regs.read(rs2)),
                            );
                        }
                        0x1 => {
                            // fsgnjn.d

                            cpu.float_regs.write(
                                rd,
                                cpu.float_regs.read(rs1).copysign(-cpu.float_regs.read(rs2)),
                            );
                        }
                        0x2 => {
                            // fsgnjx.d

                            let sign1 = cpu.float_regs.read(rs1).to_bits() & 0x80000000_00000000;
                            let sign2 = cpu.float_regs.read(rs2).to_bits() & 0x80000000_00000000;
                            let other = cpu.float_regs.read(rs1).to_bits() & 0x7fffffff_ffffffff;
                            cpu.float_regs
                                .write(rd, f64::from_bits((sign1 ^ sign2) | other));
                        }
                        _ => {
                            return Err(Exception::IllegalInstruction(inst));
                        }
                    }
                }
                0x14 => {
                    match funct3 {
                        0x0 => {
                            // fmin.s

                            cpu.float_regs
                                .write(rd, cpu.float_regs.read(rs1).min(cpu.float_regs.read(rs2)));
                        }
                        0x1 => {
                            // fmax.s

                            cpu.float_regs
                                .write(rd, cpu.float_regs.read(rs1).max(cpu.float_regs.read(rs2)));
                        }
                        _ => {
                            return Err(Exception::IllegalInstruction(inst));
                        }
                    }
                }
                0x15 => {
                    match funct3 {
                        0x0 => {
                            // fmin.d

                            cpu.float_regs
                                .write(rd, cpu.float_regs.read(rs1).min(cpu.float_regs.read(rs2)));
                        }
                        0x1 => {
                            // fmax.d

                            cpu.float_regs
                                .write(rd, cpu.float_regs.read(rs1).max(cpu.float_regs.read(rs2)));
                        }
                        _ => {
                            return Err(Exception::IllegalInstruction(inst));
                        }
                    }
                }
                0x20 => {
                    // fcvt.s.d

                    cpu.float_regs.write(rd, cpu.float_regs.read(rs1));
                }
                0x21 => {
                    // fcvt.d.s

                    cpu.float_regs
                        .write(rd, (cpu.float_regs.read(rs1) as f32) as f64);
                }
                0x2c => {
                    // fsqrt.s

                    cpu.float_regs
                        .write(rd, (cpu.float_regs.read(rs1) as f32).sqrt() as f64);
                }
                0x2d => {
                    // fsqrt.d

                    cpu.float_regs.write(rd, cpu.float_regs.read(rs1).sqrt());
                }
                0x50 => {
                    match funct3 {
                        0x0 => {
                            // fle.s

                            cpu.int_regs.write(
                                rd,
                                if cpu.float_regs.read(rs1) <= cpu.float_regs.read(rs2) {
                                    1
                                } else {
                                    0
                                },
                            );
                        }
                        0x1 => {
                            // flt.s

                            cpu.int_regs.write(
                                rd,
                                if cpu.float_regs.read(rs1) < cpu.float_regs.read(rs2) {
                                    1
                                } else {
                                    0
                                },
                            );
                        }
                        0x2 => {
                            // feq.s

                            cpu.int_regs.write(
                                rd,
                                if cpu.float_regs.read(rs1) == cpu.float_regs.read(rs2) {
                                    1
                                } else {
                                    0
                                },
                            );
                        }
                        _ => {
                            return Err(Exception::IllegalInstruction(inst));
                        }
                    }
                }
                0x51 => {
                    match funct3 {
                        0x0 => {
                            // fle.d

                            cpu.int_regs.write(
                                rd,
                                if cpu.float_regs.read(rs1) <= cpu.float_regs.read(rs2) {
                                    1
                                } else {
                                    0
                                },
                            );
                        }
                        0x1 => {
                            // flt.d

                            cpu.int_regs.write(
                                rd,
                                if cpu.float_regs.read(rs1) < cpu.float_regs.read(rs2) {
                                    1
                                } else {
                                    0
                                },
                            );
                        }
                        0x2 => {
                            // feq.d

                            cpu.int_regs.write(
                                rd,
                                if cpu.float_regs.read(rs1) == cpu.float_regs.read(rs2) {
                                    1
                                } else {
                                    0
                                },
                            );
                        }
                        _ => {
                            return Err(Exception::IllegalInstruction(inst));
                        }
                    }
                }
                0x60 => {
                    match rs2 {
                        0x0 => {
                            // fcvt.w.s

                            cpu.int_regs.write(
                                rd,
                                ((cpu.float_regs.read(rs1) as f32).round() as i32) as u64,
                            );
                        }
                        0x1 => {
                            // fcvt.wu.s

                            cpu.int_regs.write(
                                rd,
                                (((cpu.float_regs.read(rs1) as f32).round() as u32) as i32) as u64,
                            );
                        }
                        0x2 => {
                            // fcvt.l.s

                            cpu.int_regs
                                .write(rd, (cpu.float_regs.read(rs1) as f32).round() as u64);
                        }
                        0x3 => {
                            // fcvt.lu.s

                            cpu.int_regs
                                .write(rd, (cpu.float_regs.read(rs1) as f32).round() as u64);
                        }
                        _ => {
                            return Err(Exception::IllegalInstruction(inst));
                        }
                    }
                }
                0x61 => {
                    match rs2 {
                        0x0 => {
                            // fcvt.w.d

                            cpu.int_regs
                                .write(rd, (cpu.float_regs.read(rs1).round() as i32) as u64);
                        }
                        0x1 => {
                            // fcvt.wu.d

                            cpu.int_regs.write(
                                rd,
                                ((cpu.float_regs.read(rs1).round() as u32) as i32) as u64,
                            );
                        }
                        0x2 => {
                            // fcvt.l.d

                            cpu.int_regs
                                .write(rd, cpu.float_regs.read(rs1).round() as u64);
                        }
                        0x3 => {
                            // fcvt.lu.d

                            cpu.int_regs
                                .write(rd, cpu.float_regs.read(rs1).round() as u64);
                        }
                        _ => {
                            return Err(Exception::IllegalInstruction(inst));
                        }
                    }
                }
                0x68 => {
                    match rs2 {
                        0x0 => {
                            // fcvt.s.w

                            cpu.float_regs
                                .write(rd, ((cpu.int_regs.read(rs1) as i32) as f32) as f64);
                        }
                        0x1 => {
                            // fcvt.s.wu

                            cpu.float_regs
                                .write(rd, ((cpu.int_regs.read(rs1) as u32) as f32) as f64);
                        }
                        0x2 => {
                            // fcvt.s.l

                            cpu.float_regs
                                .write(rd, (cpu.int_regs.read(rs1) as f32) as f64);
                        }
                        0x3 => {
                            // fcvt.s.lu

                            cpu.float_regs
                                .write(rd, ((cpu.int_regs.read(rs1) as u64) as f32) as f64);
                        }
                        _ => {
                            return Err(Exception::IllegalInstruction(inst));
                        }
                    }
                }
                0x69 => {
                    match rs2 {
                        0x0 => {
                            // fcvt.d.w

                            cpu.float_regs
                                .write(rd, (cpu.int_regs.read(rs1) as i32) as f64);
                        }
                        0x1 => {
                            // fcvt.d.wu

                            cpu.float_regs
                                .write(rd, (cpu.int_regs.read(rs1) as u32) as f64);
                        }
                        0x2 => {
                            // fcvt.d.l

                            cpu.float_regs.write(rd, cpu.int_regs.read(rs1) as f64);
                        }
                        0x3 => {
                            // fcvt.d.lu

                            cpu.float_regs.write(rd, cpu.int_regs.read(rs1) as f64);
                        }
                        _ => {
                            return Err(Exception::IllegalInstruction(inst));
                        }
                    }
                }
                0x70 => {
                    match funct3 {
                        0x0 => {
                            // fmv.x.w

                            cpu.int_regs.write(
                                rd,
                                (cpu.float_regs.read(rs1).to_bits() & 0xffffffff) as i32 as i64
                                    as u64,
                            );
                        }
                        0x1 => {
                            // fclass.s

                            let f = cpu.float_regs.read(rs1);
                            match f.classify() {
                                FpCategory::Infinite => {
                                    cpu.int_regs
                                        .write(rd, if f.is_sign_negative() { 0 } else { 7 });
                                }
                                FpCategory::Normal => {
                                    cpu.int_regs
                                        .write(rd, if f.is_sign_negative() { 1 } else { 6 });
                                }
                                FpCategory::Subnormal => {
                                    cpu.int_regs
                                        .write(rd, if f.is_sign_negative() { 2 } else { 5 });
                                }
                                FpCategory::Zero => {
                                    cpu.int_regs
                                        .write(rd, if f.is_sign_negative() { 3 } else { 4 });
                                }
                                // don't support a signaling nan, only support a quiet nan.
                                FpCategory::Nan => cpu.int_regs.write(rd, 9),
                            }
                        }
                        _ => {
                            return Err(Exception::IllegalInstruction(inst));
                        }
                    }
                }
                0x71 => {
                    match funct3 {
                        0x0 => {
                            // fmv.x.d

                            // "FMV.X.D and FMV.D.X do not modify the bits being transferred"
                            cpu.int_regs.write(rd, cpu.float_regs.read(rs1).to_bits());
                        }
                        0x1 => {
                            // fclass.d

                            let f = cpu.float_regs.read(rs1);
                            match f.classify() {
                                FpCategory::Infinite => {
                                    cpu.int_regs
                                        .write(rd, if f.is_sign_negative() { 0 } else { 7 });
                                }
                                FpCategory::Normal => {
                                    cpu.int_regs
                                        .write(rd, if f.is_sign_negative() { 1 } else { 6 });
                                }
                                FpCategory::Subnormal => {
                                    cpu.int_regs
                                        .write(rd, if f.is_sign_negative() { 2 } else { 5 });
                                }
                                FpCategory::Zero => {
                                    cpu.int_regs
                                        .write(rd, if f.is_sign_negative() { 3 } else { 4 });
                                }
                                // don't support a signaling nan, only support a quiet nan.
                                FpCategory::Nan => cpu.int_regs.write(rd, 9),
                            }
                        }
                        _ => {
                            return Err(Exception::IllegalInstruction(inst));
                        }
                    }
                }
                0x78 => {
                    // fmv.w.x

                    cpu.float_regs
                        .write(rd, f64::from_bits(cpu.int_regs.read(rs1) & 0xffffffff));
                }
                0x79 => {
                    // fmv.d.x

                    // "FMV.X.D and FMV.D.X do not modify the bits being transferred"
                    cpu.float_regs
                        .write(rd, f64::from_bits(cpu.int_regs.read(rs1)));
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }
        }
        0x63 => {
            // RV32I
            // imm[12|10:5|4:1|11] = inst[31|30:25|11:8|7]
            let imm = (((inst & 0x80000000) as i32 as i64 >> 19) as u64)
                | ((inst & 0x80) << 4) // imm[11]
                | ((inst >> 20) & 0x7e0) // imm[10:5]
                | ((inst >> 7) & 0x1e); // imm[4:1]

            match funct3 {
                0x0 => {
                    // beq

                    if cpu.int_regs.read(rs1) == cpu.int_regs.read(rs2) {
                        cpu.pc = cpu.pc.wrapping_add(imm).wrapping_sub(4);
                    }
                }
                0x1 => {
                    // bne

                    if cpu.int_regs.read(rs1) != cpu.int_regs.read(rs2) {
                        cpu.pc = cpu.pc.wrapping_add(imm).wrapping_sub(4);
                    }
                }
                0x4 => {
                    // blt

                    if (cpu.int_regs.read(rs1) as i64) < (cpu.int_regs.read(rs2) as i64) {
                        cpu.pc = cpu.pc.wrapping_add(imm).wrapping_sub(4);
                    }
                }
                0x5 => {
                    // bge

                    if (cpu.int_regs.read(rs1) as i64) >= (cpu.int_regs.read(rs2) as i64) {
                        cpu.pc = cpu.pc.wrapping_add(imm).wrapping_sub(4);
                    }
                }
                0x6 => {
                    // bltu

                    if cpu.int_regs.read(rs1) < cpu.int_regs.read(rs2) {
                        cpu.pc = cpu.pc.wrapping_add(imm).wrapping_sub(4);
                    }
                }
                0x7 => {
                    // bgeu

                    if cpu.int_regs.read(rs1) >= cpu.int_regs.read(rs2) {
                        cpu.pc = cpu.pc.wrapping_add(imm).wrapping_sub(4);
                    }
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }
        }
        0x67 => {
            // jalr

            let t = cpu.pc.wrapping_add(4);

            let offset = (inst as i32 as i64) >> 20;
            let target = ((cpu.int_regs.read(rs1) as i64).wrapping_add(offset)) & !1;

            cpu.pc = (target as u64).wrapping_sub(4);

            cpu.int_regs.write(rd, t);
        }
        0x6F => {
            // jal

            cpu.int_regs.write(rd, cpu.pc.wrapping_add(4));

            // imm[20|10:1|11|19:12] = inst[31|30:21|20|19:12]
            let offset = (((inst & 0x80000000) as i32 as i64 >> 11) as u64) // imm[20]
                | (inst & 0xff000) // imm[19:12]
                | ((inst >> 9) & 0x800) // imm[11]
                | ((inst >> 20) & 0x7fe); // imm[10:1]

            cpu.pc = cpu.pc.wrapping_add(offset).wrapping_sub(4);
        }
        0x73 => {
            // RV32I, RVZicsr, and supervisor ISA
            let csr_addr = ((inst >> 20) & 0xfff) as u16;
            match funct3 {
                0x0 => {
                    match (rs2, funct7) {
                        (0x0, 0x0) => {
                            // ecall

                            // Makes a request of the execution environment by raising an
                            // environment call exception.
                            match cpu.mode {
                                Mode::User => {
                                    return Err(Exception::EnvironmentCallFromUMode);
                                }
                                Mode::Supervisor => {
                                    return Err(Exception::EnvironmentCallFromSMode);
                                }
                                Mode::Machine => {
                                    return Err(Exception::EnvironmentCallFromMMode);
                                }
                                _ => {
                                    return Err(Exception::IllegalInstruction(inst));
                                }
                            }
                        }
                        (0x1, 0x0) => {
                            // ebreak

                            // Makes a request of the debugger bu raising a Breakpoint
                            // exception.
                            return Err(Exception::Breakpoint);
                        }
                        (0x2, 0x0) => {
                            // uret
                            panic!("uret: not implemented yet. pc {}", cpu.pc);
                        }
                        (0x2, 0x8) => {
                            // sret

                            // "The RISC-V Reader" book says:
                            // "Returns from a supervisor-mode exception handler. Sets the pc to
                            // CSRs[sepc], the privilege mode to CSRs[sstatus].SPP,
                            // CSRs[sstatus].SIE to CSRs[sstatus].SPIE, CSRs[sstatus].SPIE to
                            // 1, and CSRs[sstatus].SPP to 0.", but the implementation in QEMU
                            // and Spike use `mstatus` instead of `sstatus`.

                            // Set the program counter to the supervisor exception program
                            // counter (SEPC).
                            cpu.pc = cpu.state.read(SEPC).wrapping_sub(4);

                            // TODO: Check TSR field

                            // Set the current privileged mode depending on a previous
                            // privilege mode for supervisor mode (SPP, 8).
                            cpu.mode = match cpu.state.read_sstatus(XSTATUS_SPP) {
                                0b0 => Mode::User,
                                0b1 => {
                                    // If SPP != M-mode, SRET also sets MPRV=0.
                                    cpu.state.write_mstatus(MSTATUS_MPRV, 0);
                                    Mode::Supervisor
                                }
                                _ => Mode::Debug,
                            };

                            // Read a previous interrupt-enable bit for supervisor mode (SPIE,
                            // 5), and set a global interrupt-enable bit for supervisor mode
                            // (SIE, 1) to it.
                            cpu.state
                                .write_sstatus(XSTATUS_SIE, cpu.state.read_sstatus(XSTATUS_SPIE));

                            // Set a previous interrupt-enable bit for supervisor mode (SPIE,
                            // 5) to 1.
                            cpu.state.write_sstatus(XSTATUS_SPIE, 1);
                            // Set a previous privilege mode for supervisor mode (SPP, 8) to 0.
                            cpu.state.write_sstatus(XSTATUS_SPP, 0);
                        }
                        (0x2, 0x18) => {
                            // mret

                            // "The RISC-V Reader" book says:
                            // "Returns from a machine-mode exception handler. Sets the pc to
                            // CSRs[mepc], the privilege mode to CSRs[mstatus].MPP,
                            // CSRs[mstatus].MIE to CSRs[mstatus].MPIE, and CSRs[mstatus].MPIE
                            // to 1; and, if user mode is supported, sets CSRs[mstatus].MPP to
                            // 0".

                            // Set the program counter to the machine exception program
                            // counter (MEPC).
                            cpu.pc = cpu.state.read(MEPC).wrapping_sub(4);

                            // Set the current privileged mode depending on a previous
                            // privilege mode for machine  mode (MPP, 11..13).
                            cpu.mode = match cpu.state.read_mstatus(MSTATUS_MPP) {
                                0b00 => {
                                    // If MPP != M-mode, MRET also sets MPRV=0.
                                    cpu.state.write_mstatus(MSTATUS_MPRV, 0);
                                    Mode::User
                                }
                                0b01 => {
                                    // If MPP != M-mode, MRET also sets MPRV=0.
                                    cpu.state.write_mstatus(MSTATUS_MPRV, 0);
                                    Mode::Supervisor
                                }
                                0b11 => Mode::Machine,
                                _ => Mode::Debug,
                            };

                            // Read a previous interrupt-enable bit for machine mode (MPIE, 7),
                            // and set a global interrupt-enable bit for machine mode (MIE, 3)
                            // to it.
                            cpu.state
                                .write_mstatus(MSTATUS_MIE, cpu.state.read_mstatus(MSTATUS_MPIE));

                            // Set a previous interrupt-enable bit for machine mode (MPIE, 7)
                            // to 1.
                            cpu.state.write_mstatus(MSTATUS_MPIE, 1);

                            // Set a previous privilege mode for machine mode (MPP, 11..13) to
                            // 0.
                            cpu.state.write_mstatus(MSTATUS_MPP, Mode::User as u64);
                        }
                        (0x5, 0x8) => {
                            // wfi
                            // "provides a hint to the implementation that the current
                            // hart can be stalled until an interrupt might need servicing."
                            cpu.idle = true;
                        }
                        (_, 0x9) => {
                            // sfence.vma
                            // "SFENCE.VMA is used to synchronize updates to in-memory
                            // memory-management data structures with current execution"
                        }
                        (_, 0x11) => {
                            // hfence.bvma
                        }
                        (_, 0x51) => {
                            // hfence.gvma
                        }
                        _ => {
                            return Err(Exception::IllegalInstruction(inst));
                        }
                    }
                }
                0x1 => {
                    // csrrw

                    let t = cpu.state.read(csr_addr);
                    cpu.state.write(csr_addr, cpu.int_regs.read(rs1));
                    cpu.int_regs.write(rd, t);

                    if csr_addr == SATP {
                        cpu.update_paging();
                    }
                }
                0x2 => {
                    // csrrs

                    let t = cpu.state.read(csr_addr);
                    cpu.state.write(csr_addr, t | cpu.int_regs.read(rs1));
                    cpu.int_regs.write(rd, t);

                    if csr_addr == SATP {
                        cpu.update_paging();
                    }
                }
                0x3 => {
                    // csrrc

                    let t = cpu.state.read(csr_addr);
                    cpu.state.write(csr_addr, t & (!cpu.int_regs.read(rs1)));
                    cpu.int_regs.write(rd, t);

                    if csr_addr == SATP {
                        cpu.update_paging();
                    }
                }
                0x5 => {
                    // csrrwi

                    let zimm = rs1;
                    cpu.int_regs.write(rd, cpu.state.read(csr_addr));
                    cpu.state.write(csr_addr, zimm);

                    if csr_addr == SATP {
                        cpu.update_paging();
                    }
                }
                0x6 => {
                    // csrrsi

                    let zimm = rs1;
                    let t = cpu.state.read(csr_addr);
                    cpu.state.write(csr_addr, t | zimm);
                    cpu.int_regs.write(rd, t);

                    if csr_addr == SATP {
                        cpu.update_paging();
                    }
                }
                0x7 => {
                    // csrrci

                    let zimm = rs1;
                    let t = cpu.state.read(csr_addr);
                    cpu.state.write(csr_addr, t & (!zimm));
                    cpu.int_regs.write(rd, t);

                    if csr_addr == SATP {
                        cpu.update_paging();
                    }
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }
        }
        _ => {
            return Err(Exception::IllegalInstruction(inst));
        }
    }
    Ok(())
}

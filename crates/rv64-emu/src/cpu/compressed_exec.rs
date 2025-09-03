use crate::{
    cpu::{CPU, DOUBLEWORD, WORD},
    exception::Exception,
};

/// Execute a compressed instruction. Raised an exception if something is wrong, otherwise,
/// returns a fetched instruction. It also increments the program counter by 2 bytes.
pub fn execute_compressed_inner(cpu: &mut CPU, inst: u64) -> Result<(), Exception> {
    // 2. Decode.
    let opcode = inst & 0x3;
    let funct3 = (inst >> 13) & 0x7;

    // 3. Execute.
    // Compressed instructions have 3-bit field for popular registers, which correspond to
    // registers x8 to x15.
    match opcode {
        0 => {
            // Quadrant 0.
            match funct3 {
                0x0 => {
                    // c.addi4spn
                    // Expands to addi rd, x2, nzuimm, where rd=rd'+8.

                    let rd = ((inst >> 2) & 0x7) + 8;
                    // nzuimm[5:4|9:6|2|3] = inst[12:11|10:7|6|5]
                    let nzuimm = ((inst >> 1) & 0x3c0) // znuimm[9:6]
                            | ((inst >> 7) & 0x30) // znuimm[5:4]
                            | ((inst >> 2) & 0x8) // znuimm[3]
                            | ((inst >> 4) & 0x4); // znuimm[2]
                    if nzuimm == 0 {
                        return Err(Exception::IllegalInstruction(inst));
                    }
                    cpu.int_regs
                        .write(rd, cpu.int_regs.read(2).wrapping_add(nzuimm));
                }
                0x1 => {
                    // c.fld
                    // Expands to fld rd, offset(rs1), where rd=rd'+8 and rs1=rs1'+8.

                    let rd = ((inst >> 2) & 0x7) + 8;
                    let rs1 = ((inst >> 7) & 0x7) + 8;
                    // offset[5:3|7:6] = isnt[12:10|6:5]
                    let offset = ((inst << 1) & 0xc0) // imm[7:6]
                            | ((inst >> 7) & 0x38); // imm[5:3]
                    let val = f64::from_bits(
                        cpu.read(cpu.int_regs.read(rs1).wrapping_add(offset), DOUBLEWORD)?,
                    );
                    cpu.float_regs.write(rd, val);
                }
                0x2 => {
                    // c.lw
                    // Expands to lw rd, offset(rs1), where rd=rd'+8 and rs1=rs1'+8.

                    let rd = ((inst >> 2) & 0x7) + 8;
                    let rs1 = ((inst >> 7) & 0x7) + 8;
                    // offset[5:3|2|6] = isnt[12:10|6|5]
                    let offset = ((inst << 1) & 0x40) // imm[6]
                            | ((inst >> 7) & 0x38) // imm[5:3]
                            | ((inst >> 4) & 0x4); // imm[2]
                    let addr = cpu.int_regs.read(rs1).wrapping_add(offset);
                    let val = cpu.read(addr, WORD)?;
                    cpu.int_regs.write(rd, val as i32 as i64 as u64);
                }
                0x3 => {
                    // c.ld
                    // Expands to ld rd, offset(rs1), where rd=rd'+8 and rs1=rs1'+8.

                    let rd = ((inst >> 2) & 0x7) + 8;
                    let rs1 = ((inst >> 7) & 0x7) + 8;
                    // offset[5:3|7:6] = isnt[12:10|6:5]
                    let offset = ((inst << 1) & 0xc0) // imm[7:6]
                            | ((inst >> 7) & 0x38); // imm[5:3]
                    let addr = cpu.int_regs.read(rs1).wrapping_add(offset);
                    let val = cpu.read(addr, DOUBLEWORD)?;
                    cpu.int_regs.write(rd, val);
                }
                0x4 => {
                    // Reserved.
                    panic!("reserved");
                }
                0x5 => {
                    // c.fsd
                    // Expands to fsd rs2, offset(rs1), where rs2=rs2'+8 and rs1=rs1'+8.

                    let rs2 = ((inst >> 2) & 0x7) + 8;
                    let rs1 = ((inst >> 7) & 0x7) + 8;
                    // offset[5:3|7:6] = isnt[12:10|6:5]
                    let offset = ((inst << 1) & 0xc0) // imm[7:6]
                            | ((inst >> 7) & 0x38); // imm[5:3]
                    let addr = cpu.int_regs.read(rs1).wrapping_add(offset);
                    cpu.write(addr, cpu.float_regs.read(rs2).to_bits() as u64, DOUBLEWORD)?;
                }
                0x6 => {
                    // c.sw
                    // Expands to sw rs2, offset(rs1), where rs2=rs2'+8 and rs1=rs1'+8.

                    let rs2 = ((inst >> 2) & 0x7) + 8;
                    let rs1 = ((inst >> 7) & 0x7) + 8;
                    // offset[5:3|2|6] = isnt[12:10|6|5]
                    let offset = ((inst << 1) & 0x40) // imm[6]
                            | ((inst >> 7) & 0x38) // imm[5:3]
                            | ((inst >> 4) & 0x4); // imm[2]
                    let addr = cpu.int_regs.read(rs1).wrapping_add(offset);
                    cpu.write(addr, cpu.int_regs.read(rs2), WORD)?;
                }
                0x7 => {
                    // c.sd
                    // Expands to sd rs2, offset(rs1), where rs2=rs2'+8 and rs1=rs1'+8.

                    let rs2 = ((inst >> 2) & 0x7) + 8;
                    let rs1 = ((inst >> 7) & 0x7) + 8;
                    // offset[5:3|7:6] = isnt[12:10|6:5]
                    let offset = ((inst << 1) & 0xc0) // imm[7:6]
                            | ((inst >> 7) & 0x38); // imm[5:3]
                    let addr = cpu.int_regs.read(rs1).wrapping_add(offset);
                    cpu.write(addr, cpu.int_regs.read(rs2), DOUBLEWORD)?;
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }
        }
        1 => {
            // Quadrant 1.
            match funct3 {
                0x0 => {
                    // c.addi
                    // Expands to addi rd, rd, nzimm.

                    let rd = (inst >> 7) & 0x1f;
                    // nzimm[5|4:0] = inst[12|6:2]
                    let mut nzimm = ((inst >> 7) & 0x20) | ((inst >> 2) & 0x1f);
                    // Sign-extended.
                    nzimm = match (nzimm & 0x20) == 0 {
                        true => nzimm,
                        false => (0xc0 | nzimm) as i8 as i64 as u64,
                    };
                    if rd != 0 {
                        cpu.int_regs
                            .write(rd, cpu.int_regs.read(rd).wrapping_add(nzimm));
                    }
                }
                0x1 => {
                    // c.addiw
                    // Expands to addiw rd, rd, imm
                    // "The immediate can be zero for C.ADDIW, where this corresponds to sext.w
                    // rd"

                    let rd = (inst >> 7) & 0x1f;
                    // imm[5|4:0] = inst[12|6:2]
                    let mut imm = ((inst >> 7) & 0x20) | ((inst >> 2) & 0x1f);
                    // Sign-extended.
                    imm = match (imm & 0x20) == 0 {
                        true => imm,
                        false => (0xc0 | imm) as i8 as i64 as u64,
                    };
                    if rd != 0 {
                        cpu.int_regs.write(
                            rd,
                            cpu.int_regs.read(rd).wrapping_add(imm) as i32 as i64 as u64,
                        );
                    }
                }
                0x2 => {
                    // c.li
                    // Expands to addi rd, x0, imm.

                    let rd = (inst >> 7) & 0x1f;
                    // imm[5|4:0] = inst[12|6:2]
                    let mut imm = ((inst >> 7) & 0x20) | ((inst >> 2) & 0x1f);
                    // Sign-extended.
                    imm = match (imm & 0x20) == 0 {
                        true => imm,
                        false => (0xc0 | imm) as i8 as i64 as u64,
                    };
                    if rd != 0 {
                        cpu.int_regs.write(rd, imm);
                    }
                }
                0x3 => {
                    let rd = (inst >> 7) & 0x1f;
                    match rd {
                        0 => {}
                        2 => {
                            // c.addi16sp
                            // Expands to addi x2, x2, nzimm

                            // nzimm[9|4|6|8:7|5] = inst[12|6|5|4:3|2]
                            let mut nzimm = ((inst >> 3) & 0x200) // nzimm[9]
                                    | ((inst >> 2) & 0x10) // nzimm[4]
                                    | ((inst << 1) & 0x40) // nzimm[6]
                                    | ((inst << 4) & 0x180) // nzimm[8:7]
                                    | ((inst << 3) & 0x20); // nzimm[5]
                            nzimm = match (nzimm & 0x200) == 0 {
                                true => nzimm,
                                // Sign-extended.
                                false => (0xfc00 | nzimm) as i16 as i32 as i64 as u64,
                            };
                            if nzimm != 0 {
                                cpu.int_regs
                                    .write(2, cpu.int_regs.read(2).wrapping_add(nzimm));
                            }
                        }
                        _ => {
                            // c.lui
                            // Expands to lui rd, nzimm.

                            // nzimm[17|16:12] = inst[12|6:2]
                            let mut nzimm = ((inst << 5) & 0x20000) | ((inst << 10) & 0x1f000);
                            // Sign-extended.
                            nzimm = match (nzimm & 0x20000) == 0 {
                                true => nzimm,
                                false => (0xfffc0000 | nzimm) as i32 as i64 as u64,
                            };
                            if nzimm != 0 {
                                cpu.int_regs.write(rd, nzimm);
                            }
                        }
                    }
                }
                0x4 => {
                    let funct2 = (inst >> 10) & 0x3;
                    match funct2 {
                        0x0 => {
                            // c.srli
                            // Expands to srli rd, rd, shamt, where rd=rd'+8.

                            let rd = ((inst >> 7) & 0b111) + 8;
                            // shamt[5|4:0] = inst[12|6:2]
                            let shamt = ((inst >> 7) & 0x20) | ((inst >> 2) & 0x1f);
                            cpu.int_regs.write(rd, cpu.int_regs.read(rd) >> shamt);
                        }
                        0x1 => {
                            // c.srai
                            // Expands to srai rd, rd, shamt, where rd=rd'+8.

                            let rd = ((inst >> 7) & 0b111) + 8;
                            // shamt[5|4:0] = inst[12|6:2]
                            let shamt = ((inst >> 7) & 0x20) | ((inst >> 2) & 0x1f);
                            cpu.int_regs
                                .write(rd, ((cpu.int_regs.read(rd) as i64) >> shamt) as u64);
                        }
                        0x2 => {
                            // c.andi
                            // Expands to andi rd, rd, imm, where rd=rd'+8.

                            let rd = ((inst >> 7) & 0b111) + 8;
                            // imm[5|4:0] = inst[12|6:2]
                            let mut imm = ((inst >> 7) & 0x20) | ((inst >> 2) & 0x1f);
                            // Sign-extended.
                            imm = match (imm & 0x20) == 0 {
                                true => imm,
                                false => (0xc0 | imm) as i8 as i64 as u64,
                            };
                            cpu.int_regs.write(rd, cpu.int_regs.read(rd) & imm);
                        }
                        0x3 => {
                            match ((inst >> 12) & 0b1, (inst >> 5) & 0b11) {
                                (0x0, 0x0) => {
                                    // c.sub
                                    // Expands to sub rd, rd, rs2, rd=rd'+8 and rs2=rs2'+8.

                                    let rd = ((inst >> 7) & 0b111) + 8;
                                    let rs2 = ((inst >> 2) & 0b111) + 8;
                                    cpu.int_regs.write(
                                        rd,
                                        cpu.int_regs.read(rd).wrapping_sub(cpu.int_regs.read(rs2)),
                                    );
                                }
                                (0x0, 0x1) => {
                                    // c.xor
                                    // Expands to xor rd, rd, rs2, rd=rd'+8 and rs2=rs2'+8.

                                    let rd = ((inst >> 7) & 0b111) + 8;
                                    let rs2 = ((inst >> 2) & 0b111) + 8;
                                    cpu.int_regs
                                        .write(rd, cpu.int_regs.read(rd) ^ cpu.int_regs.read(rs2));
                                }
                                (0x0, 0x2) => {
                                    // c.or
                                    // Expands to or rd, rd, rs2, rd=rd'+8 and rs2=rs2'+8.

                                    let rd = ((inst >> 7) & 0b111) + 8;
                                    let rs2 = ((inst >> 2) & 0b111) + 8;
                                    cpu.int_regs
                                        .write(rd, cpu.int_regs.read(rd) | cpu.int_regs.read(rs2));
                                }
                                (0x0, 0x3) => {
                                    // c.and
                                    // Expands to and rd, rd, rs2, rd=rd'+8 and rs2=rs2'+8.

                                    let rd = ((inst >> 7) & 0b111) + 8;
                                    let rs2 = ((inst >> 2) & 0b111) + 8;
                                    cpu.int_regs
                                        .write(rd, cpu.int_regs.read(rd) & cpu.int_regs.read(rs2));
                                }
                                (0x1, 0x0) => {
                                    // c.subw
                                    // Expands to subw rd, rd, rs2, rd=rd'+8 and rs2=rs2'+8.

                                    let rd = ((inst >> 7) & 0b111) + 8;
                                    let rs2 = ((inst >> 2) & 0b111) + 8;
                                    cpu.int_regs.write(
                                        rd,
                                        cpu.int_regs.read(rd).wrapping_sub(cpu.int_regs.read(rs2))
                                            as i32 as i64
                                            as u64,
                                    );
                                }
                                (0x1, 0x1) => {
                                    // c.addw
                                    // Expands to addw rd, rd, rs2, rd=rd'+8 and rs2=rs2'+8.

                                    let rd = ((inst >> 7) & 0b111) + 8;
                                    let rs2 = ((inst >> 2) & 0b111) + 8;
                                    cpu.int_regs.write(
                                        rd,
                                        cpu.int_regs.read(rd).wrapping_add(cpu.int_regs.read(rs2))
                                            as i32 as i64
                                            as u64,
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
                0x5 => {
                    // c.j
                    // Expands to jal x0, offset.

                    // offset[11|4|9:8|10|6|7|3:1|5] = inst[12|11|10:9|8|7|6|5:3|2]
                    let mut offset = ((inst >> 1) & 0x800) // offset[11]
                            | ((inst << 2) & 0x400) // offset[10]
                            | ((inst >> 1) & 0x300) // offset[9:8]
                            | ((inst << 1) & 0x80) // offset[7]
                            | ((inst >> 1) & 0x40) // offset[6]
                            | ((inst << 3) & 0x20) // offset[5]
                            | ((inst >> 7) & 0x10) // offset[4]
                            | ((inst >> 2) & 0xe); // offset[3:1]

                    // Sign-extended.
                    offset = match (offset & 0x800) == 0 {
                        true => offset,
                        false => (0xf000 | offset) as i16 as i64 as u64,
                    };
                    cpu.pc = cpu.pc.wrapping_add(offset).wrapping_sub(2);
                }
                0x6 => {
                    // c.beqz
                    // Expands to beq rs1, x0, offset, rs1=rs1'+8.

                    let rs1 = ((inst >> 7) & 0b111) + 8;
                    // offset[8|4:3|7:6|2:1|5] = inst[12|11:10|6:5|4:3|2]
                    let mut offset = ((inst >> 4) & 0x100) // offset[8]
                            | ((inst << 1) & 0xc0) // offset[7:6]
                            | ((inst << 3) & 0x20) // offset[5]
                            | ((inst >> 7) & 0x18) // offset[4:3]
                            | ((inst >> 2) & 0x6); // offset[2:1]
                                                   // Sign-extended.
                    offset = match (offset & 0x100) == 0 {
                        true => offset,
                        false => (0xfe00 | offset) as i16 as i64 as u64,
                    };
                    if cpu.int_regs.read(rs1) == 0 {
                        cpu.pc = cpu.pc.wrapping_add(offset).wrapping_sub(2);
                    }
                }
                0x7 => {
                    // c.bnez
                    // Expands to bne rs1, x0, offset, rs1=rs1'+8.

                    let rs1 = ((inst >> 7) & 0b111) + 8;
                    // offset[8|4:3|7:6|2:1|5] = inst[12|11:10|6:5|4:3|2]
                    let mut offset = ((inst >> 4) & 0x100) // offset[8]
                            | ((inst << 1) & 0xc0) // offset[7:6]
                            | ((inst << 3) & 0x20) // offset[5]
                            | ((inst >> 7) & 0x18) // offset[4:3]
                            | ((inst >> 2) & 0x6); // offset[2:1]
                                                   // Sign-extended.
                    offset = match (offset & 0x100) == 0 {
                        true => offset,
                        false => (0xfe00 | offset) as i16 as i64 as u64,
                    };
                    if cpu.int_regs.read(rs1) != 0 {
                        cpu.pc = cpu.pc.wrapping_add(offset).wrapping_sub(2);
                    }
                }
                _ => {
                    return Err(Exception::IllegalInstruction(inst));
                }
            }
        }
        2 => {
            // Quadrant 2.
            match funct3 {
                0x0 => {
                    // c.slli
                    // Expands to slli rd, rd, shamt.

                    let rd = (inst >> 7) & 0x1f;
                    // shamt[5|4:0] = inst[12|6:2]
                    let shamt = ((inst >> 7) & 0x20) | ((inst >> 2) & 0x1f);
                    if rd != 0 {
                        cpu.int_regs.write(rd, cpu.int_regs.read(rd) << shamt);
                    }
                }
                0x1 => {
                    // c.fldsp
                    // Expands to fld rd, offset(x2).

                    let rd = (inst >> 7) & 0x1f;
                    // offset[5|4:3|8:6] = inst[12|6:5|4:2]
                    let offset = ((inst << 4) & 0x1c0) // offset[8:6]
                            | ((inst >> 7) & 0x20) // offset[5]
                            | ((inst >> 2) & 0x18); // offset[4:3]
                    let val = f64::from_bits(cpu.read(cpu.int_regs.read(2) + offset, DOUBLEWORD)?);
                    cpu.float_regs.write(rd, val);
                }
                0x2 => {
                    // c.lwsp
                    // Expands to lw rd, offset(x2).

                    let rd = (inst >> 7) & 0x1f;
                    // offset[5|4:2|7:6] = inst[12|6:4|3:2]
                    let offset = ((inst << 4) & 0xc0) // offset[7:6]
                            | ((inst >> 7) & 0x20) // offset[5]
                            | ((inst >> 2) & 0x1c); // offset[4:2]
                    let val = cpu.read(cpu.int_regs.read(2).wrapping_add(offset), WORD)?;
                    cpu.int_regs.write(rd, val as i32 as i64 as u64);
                }
                0x3 => {
                    // c.ldsp
                    // Expands to ld rd, offset(x2).

                    let rd = (inst >> 7) & 0x1f;
                    // offset[5|4:3|8:6] = inst[12|6:5|4:2]
                    let offset = ((inst << 4) & 0x1c0) // offset[8:6]
                            | ((inst >> 7) & 0x20) // offset[5]
                            | ((inst >> 2) & 0x18); // offset[4:3]
                    let val = cpu.read(cpu.int_regs.read(2).wrapping_add(offset), DOUBLEWORD)?;
                    cpu.int_regs.write(rd, val);
                }
                0x4 => {
                    match ((inst >> 12) & 0x1, (inst >> 2) & 0x1f) {
                        (0, 0) => {
                            // c.jr
                            // Expands to jalr x0, 0(rs1).

                            let rs1 = (inst >> 7) & 0x1f;
                            if rs1 != 0 {
                                cpu.pc = cpu.int_regs.read(rs1).wrapping_sub(2);
                            }
                        }
                        (0, _) => {
                            // c.mv
                            // Expands to add rd, x0, rs2.

                            let rd = (inst >> 7) & 0x1f;
                            let rs2 = (inst >> 2) & 0x1f;
                            if rs2 != 0 {
                                cpu.int_regs.write(rd, cpu.int_regs.read(rs2));
                            }
                        }
                        (1, 0) => {
                            let rd = (inst >> 7) & 0x1f;
                            if rd == 0 {
                                // c.ebreak
                                // Expands to ebreak.

                                return Err(Exception::Breakpoint);
                            } else {
                                // c.jalr
                                // Expands to jalr x1, 0(rs1).

                                let rs1 = (inst >> 7) & 0x1f;
                                let t = cpu.pc.wrapping_add(2);
                                cpu.pc = cpu.int_regs.read(rs1).wrapping_sub(2);
                                cpu.int_regs.write(1, t);
                            }
                        }
                        (1, _) => {
                            // c.add
                            // Expands to add rd, rd, rs2.

                            let rd = (inst >> 7) & 0x1f;
                            let rs2 = (inst >> 2) & 0x1f;
                            if rs2 != 0 {
                                cpu.int_regs.write(
                                    rd,
                                    cpu.int_regs.read(rd).wrapping_add(cpu.int_regs.read(rs2)),
                                );
                            }
                        }
                        (_, _) => {
                            return Err(Exception::IllegalInstruction(inst));
                        }
                    }
                }
                0x5 => {
                    // c.fsdsp
                    // Expands to fsd rs2, offset(x2).

                    let rs2 = (inst >> 2) & 0x1f;
                    // offset[5:3|8:6] = isnt[12:10|9:7]
                    let offset = ((inst >> 1) & 0x1c0) // offset[8:6]
                            | ((inst >> 7) & 0x38); // offset[5:3]
                    let addr = cpu.int_regs.read(2).wrapping_add(offset);
                    cpu.write(addr, cpu.float_regs.read(rs2).to_bits(), DOUBLEWORD)?;
                }
                0x6 => {
                    // c.swsp
                    // Expands to sw rs2, offset(x2).

                    let rs2 = (inst >> 2) & 0x1f;
                    // offset[5:2|7:6] = inst[12:9|8:7]
                    let offset = ((inst >> 1) & 0xc0) // offset[7:6]
                            | ((inst >> 7) & 0x3c); // offset[5:2]
                    let addr = cpu.int_regs.read(2).wrapping_add(offset);
                    cpu.write(addr, cpu.int_regs.read(rs2), WORD)?;
                }
                0x7 => {
                    // c.sdsp
                    // Expands to sd rs2, offset(x2).

                    let rs2 = (inst >> 2) & 0x1f;
                    // offset[5:3|8:6] = isnt[12:10|9:7]
                    let offset = ((inst >> 1) & 0x1c0) // offset[8:6]
                            | ((inst >> 7) & 0x38); // offset[5:3]
                    let addr = cpu.int_regs.read(2).wrapping_add(offset);
                    cpu.write(addr, cpu.int_regs.read(rs2), DOUBLEWORD)?;
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

use super::{AddrMode, AddrMode2, Encodable, Label, Reg};
use std::collections::HashMap;

pub enum Op {
    Addi(Reg, Reg, u32),
    Adrl(Reg, Label),
    Adri(Reg, i32),
    Ldm(AddrMode, Reg, bool, Vec<Reg>),
    Ldri(AddrMode2, Reg, Reg, i16),
    Ldrl(Reg, Label),
    Movr(Reg, Reg),
    Movw(Reg, u32),
    Subi(Reg, Reg, u32),
    Stm(AddrMode, Reg, bool, Vec<Reg>),
    Svc(u32),
    Placeholder,
}

impl From<Op> for u32 {
    fn from(op: Op) -> u32 {
        match op {
            Op::Addi(rd, rn, imm) => 0xe2800000 | rn << 16 | rd << 12 | imm,
            Op::Adri(rn, imm) => {
                if imm < 0 {
                    Op::Subi(rn, Reg::pc, -imm as u32).into()
                } else {
                    Op::Addi(rn, Reg::pc, imm as u32).into()
                }
            }
            Op::Ldm(mode, rn, wb, regs) => regs.into_iter().fold(
                0xe8100000 | mode << 23 | (wb as u32) << 21 | rn << 16,
                |acc, rn| acc | 1 << rn,
            ),
            Op::Ldri(mode, rt, rn, imm) => {
                let (index, wback) = match mode {
                    AddrMode2::Offset => (1, 0),
                    AddrMode2::PreIndexed => (1, 1),
                    AddrMode2::PostIndexed => (0, 1),
                };

                0xe4100000
                    | index << 24
                    | if imm < 0 { 0 } else { 1 } << 23
                    | wback << 21
                    | rn << 16
                    | rt << 12
                    | imm.abs() as u32
            }
            Op::Movr(rd, rm) => 0xe1a00000 | rd << 12 | rm,
            Op::Movw(rd, imm) => 0xe3000000 | (imm >> 12) << 16 | rd << 12 | ((1 << 12) - 1) & imm,
            Op::Stm(mode, rn, wb, regs) => regs.into_iter().fold(
                0xe8000000 | mode << 23 | (wb as u32) << 21 | rn << 16,
                |acc, rn| acc | 1 << rn,
            ),
            Op::Subi(rd, rn, imm) => 0xe2400000 | rn << 16 | rd << 12 | imm,
            Op::Svc(imm) => 0xef000000 | imm,
            _ => 0,
        }
    }
}

impl Encodable<4> for Op {
    fn enc(self, off: usize, labs: &HashMap<Label, usize>) -> [u8; 4] {
        u32::from(match self {
            Op::Adrl(rn, label) => Op::Adri(rn, Self::res_lab(label, labs, off)),
            Op::Ldrl(rt, label) => Op::Ldri(
                AddrMode2::Offset,
                rt,
                Reg::pc,
                Self::res_lab(label, labs, off) as i16,
            ),
            op => op,
        })
        .to_le_bytes()
    }

    fn calc_offset(op_offset: i32, label_offset: i32) -> i32 {
        label_offset - op_offset - 8
    }
}

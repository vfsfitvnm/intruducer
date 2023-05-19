use std::collections::HashMap;

use crate::{Encodable, Label};

use super::{AddrMode2, Reg, Shift};

pub enum Op {
    Adri(Reg, i32),
    Adrl(Reg, Label),
    Blr(Reg),
    Br(Reg),
    Ldp(AddrMode2, Reg, Reg, Reg, i16),
    Ldri(AddrMode2, Reg, Reg, i32),
    Ldrl(Reg, Label),
    Ldrli(Reg, i32),
    Movi(Reg, i32),
    Orrsr(Reg, Reg, Reg, (Shift, u8)),
    Stp(AddrMode2, Reg, Reg, Reg, i16),
    Stri(AddrMode2, Reg, Reg, i32),
    Svc(u16),
    Placeholder,
}

impl From<Op> for u32 {
    fn from(op: Op) -> u32 {
        match op {
            Op::Adri(xd, imm) => {
                if imm < 0 {
                    Op::Adri(xd, (1 << 21) + imm).into()
                } else {
                    // TODO: this is @$^!@ but works
                    (((((imm as u32) << 1) + 1) % 8) << 28) | ((imm as u32) >> 2) << 5 | xd
                }
            }
            Op::Blr(xn) => 0xd63f0000 | xn << 5,
            Op::Br(xn) => 0xd61f0000 | xn << 5,
            Op::Ldp(mode, xt1, xt2, xn, imm) => {
                if imm < 0 {
                    Op::Ldp(mode, xt1, xt2, xn, 1024 + imm).into()
                } else {
                    let mode = match mode {
                        AddrMode2::Offset => 2,
                        AddrMode2::PreIndexed => 3,
                        AddrMode2::PostIndexed => 1,
                    };
                    0xa8400000 | mode << 23 | (imm as u32 >> 3) << 15 | xt2 << 10 | xn << 5 | xt1
                }
            }
            Op::Ldri(mode, xt, xn, imm) => {
                if imm < 0 {
                    Op::Ldri(mode, xt, xn, 512 + imm).into()
                } else {
                    match mode {
                        AddrMode2::Offset => 0xf9400000 | (imm as u32) << 10 | xn << 5 | xt,
                        AddrMode2::PreIndexed => 0xf8400c00 | (imm as u32) << 12 | xn << 5 | xt,
                        AddrMode2::PostIndexed => 0xf8400400 | (imm as u32) << 12 | xn << 5 | xt,
                    }
                }
            }
            Op::Ldrli(xt, imm) => {
                if imm < 0 {
                    Op::Ldrli(xt, (1 << 21) + imm).into()
                } else {
                    0x58000000 | (imm as u32) << 3 | xt
                }
            }
            Op::Movi(xd, imm) => {
                if imm < 0 {
                    0x40000020 ^ u32::from(Op::Movi(xd, -imm))
                } else {
                    0xd2800000 | (imm as u32) << 5 | xd
                }
            }
            Op::Orrsr(xd, xn, xm, (shift, amount)) => {
                0xaa000000 | shift << 22 | xm << 16 | (amount as u32) << 10 | xn << 5 | xd
            }
            Op::Stp(mode, xt1, xt2, xn, imm) => {
                if imm < 0 {
                    Op::Stp(mode, xt1, xt2, xn, 1024 + imm).into()
                } else {
                    let mode = match mode {
                        AddrMode2::Offset => 2,
                        AddrMode2::PreIndexed => 3,
                        AddrMode2::PostIndexed => 1,
                    };
                    0xa8000000 | mode << 23 | (imm as u32 >> 3) << 15 | xt2 << 10 | xn << 5 | xt1
                }
            }
            Op::Stri(mode, xt, xn, imm) => {
                if imm < 0 {
                    Op::Stri(mode, xt, xn, 512 + imm).into()
                } else {
                    match mode {
                        AddrMode2::Offset => 0xf9000000 | (imm as u32) << 10 | xn << 5 | xt,
                        AddrMode2::PreIndexed => 0xf8000c00 | (imm as u32) << 12 | xn << 5 | xt,
                        AddrMode2::PostIndexed => 0xf8000400 | (imm as u32) << 12 | xn << 5 | xt,
                    }
                }
            }
            Op::Svc(imm) => 0xd4000001 | (imm as u32) << 5,
            _ => 0,
        }
    }
}

impl Encodable<4> for Op {
    fn enc(self, offset: usize, labels: &HashMap<Label, usize>) -> [u8; 4] {
        u32::from(match self {
            Op::Adrl(xd, label) => Op::Adri(xd, Self::res_lab(label, labels, offset)),
            Op::Ldrl(xt, label) => Op::Ldrli(xt, Self::res_lab(label, labels, offset)),
            op => op,
        })
        .to_le_bytes()
    }

    fn calc_offset(instr_offset: i32, label_offset: i32) -> i32 {
        label_offset - instr_offset
    }
}

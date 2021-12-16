use std::{
    collections::HashMap,
    ops::{BitOr, Shl},
};

use super::{Encodable, Label};

#[derive(Clone, Copy, PartialEq)]
pub enum Reg {
    X0 = 0,
    X1 = 1,
    X2 = 2,
    X3 = 3,
    X4 = 4,
    X5 = 5,
    X6 = 6,
    X7 = 7,
    X8 = 8,
    X9 = 9,
    X10 = 10,
    X11 = 11,
    X12 = 12,
    X13 = 13,
    X14 = 14,
    X15 = 15,
    X16 = 16,
    X17 = 17,
    X18 = 18,
    X19 = 19,
    X20 = 20,
    X21 = 21,
    X22 = 22,
    X23 = 23,
    X24 = 24,
    X25 = 25,
    X26 = 26,
    X27 = 27,
    X28 = 28,
    X29 = 29,
    X30 = 30,
    SP,
    XZR,
}

impl Reg {
    const fn val(&self) -> u32 {
        match *self {
            Reg::SP | Reg::XZR => 31,
            reg => reg as u32,
        }
    }
}

pub enum Shift {
    Lsr = 0,
    Asr = 1,
    Lsl = 2,
    Ror = 3,
}

pub enum AddrMode2 {
    Offset,
    PreIndexed,
    PostIndexed,
}

pub(crate) enum Op {
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

/// https://developer.arm.com/documentation/ddi0596/2021-09/Base-Instructions
impl TinyAsm {
    /// Encoding of ADR: `ADR <Xd>, <label>`,
    pub fn adr(mut self, xd: Reg, label: Label) -> Self {
        self.relocs.push((self.buf.len(), Op::Adrl(xd, label)));
        self.op(Op::Placeholder)
    }

    /// Encoding of BLR: `BLR <Xn>`.
    pub fn blr(self, xn: Reg) -> Self {
        self.op(Op::Blr(xn))
    }

    /// Encoding of BR: `BR <Xn>`.
    pub fn br(self, xn: Reg) -> Self {
        self.op(Op::Br(xn))
    }

    /// Encoding of LDP: `LDP <Xt1>, <Xt2>, [<Xn|SP>], #<imm>`, `LDP <Xt1>, <Xt2>, [<Xn|SP>, #<imm>]!`, `LDP <Xt1>, <Xt2>, [<Xn|SP>{, #<imm>}]`.
    pub fn ldp(self, mode: AddrMode2, xt1: Reg, xt2: Reg, xn: Reg, imm: i16) -> Self {
        self.op(Op::Ldp(mode, xt1, xt2, xn, imm))
    }

    /// Encoding of LDR (immediate): `LDR <Xt>, [<Xn|SP>], #<simm>`, `LDR <Xt>, [<Xn|SP>, #<simm>]!`, `LDR <Xt>, [<Xn|SP>{, #<pimm>}]`.
    pub fn ldri(self, mode: AddrMode2, xt: Reg, xn: Reg, imm: i32) -> Self {
        self.op(Op::Ldri(mode, xt, xn, imm))
    }

    /// Encoding of LDR (literal): `LDR <Xt>, <label>`.
    pub fn ldrl(mut self, xt: Reg, label: Label) -> Self {
        self.relocs.push((self.buf.len(), Op::Ldrl(xt, label)));
        self.op(Op::Placeholder)
    }

    /// Encoding of MOV (register): `MOV <Xd>, <Xm>`.
    pub fn movr(self, xd: Reg, xm: Reg) -> Self {
        self.orrsr(xd, Reg::XZR, xm, None)
    }

    /// Encoding of MOV (wide immediate): `MOV Xd, #<imm>`.
    pub fn movi(self, xd: Reg, imm: i32) -> Self {
        self.op(Op::Movi(xd, imm))
    }

    /// Encoding of ORR (Shifted Register): `ORR <Xd>, <Xn>, <Xm>{, <shift> #<amount>}`.
    pub fn orrsr(self, xd: Reg, xn: Reg, xm: Reg, shift: Option<(Shift, u8)>) -> Self {
        self.op(Op::Orrsr(xd, xn, xm, shift.unwrap_or((Shift::Lsl, 0))))
    }

    /// Encoding of STP: `STP <Xt1>, <Xt2>, [<Xn|SP>], #<imm>`, `STP <Xt1>, <Xt2>, [<Xn|SP>, #<imm>]!`, `STP <Xt1>, <Xt2>, [<Xn|SP>, #<imm>]!`.
    pub fn stp(self, mode: AddrMode2, xt1: Reg, xt2: Reg, xn: Reg, imm: i16) -> Self {
        self.op(Op::Stp(mode, xt1, xt2, xn, imm))
    }

    /// Encoding of STR (immediate): `STR <Xt>, [<Xn|SP>], #<simm>`, `STR <Xt>, [<Xn|SP>, #<simm>]!`, `STR <Xt>, [<Xn|SP>{, #<pimm>}]`.
    pub fn stri(self, mode: AddrMode2, xt: Reg, xn: Reg, imm: i32) -> Self {
        self.op(Op::Stri(mode, xt, xn, imm))
    }

    /// Encoding of SVC: `SVC #<imm16>`.
    pub fn svc(self, imm: u16) -> Self {
        self.op(Op::Svc(imm))
    }
}

impl BitOr<Reg> for u32 {
    type Output = u32;

    fn bitor(self, rhs: Reg) -> Self::Output {
        self | rhs.val()
    }
}

impl Shl<Reg> for u32 {
    type Output = u32;

    fn shl(self, rhs: Reg) -> Self::Output {
        self << rhs.val()
    }
}

impl Shl<u32> for Reg {
    type Output = u32;

    fn shl(self, rhs: u32) -> Self::Output {
        self.val() << rhs as u32
    }
}

impl Shl<u32> for Shift {
    type Output = u32;

    fn shl(self, rhs: u32) -> Self::Output {
        (self as u32) << rhs
    }
}

pub(crate) type TinyAsm = super::TinyAsm<Op, 4>;

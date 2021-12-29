use std::{
    collections::HashMap,
    ops::{BitOr, Shl},
};

use super::{Encodable, Label};

#[derive(Clone, Copy, PartialEq)]
pub enum Reg {
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,
    SP,
    LR,
    PC,
}

impl Reg {
    const fn val(self) -> u32 {
        match self {
            Reg::SP => Reg::R13.val(),
            Reg::LR => Reg::R14.val(),
            Reg::PC => Reg::R15.val(),
            reg => reg as u32,
        }
    }
}

pub enum AddrMode {
    DecrAfter = 0,
    IncrAfter = 1,
    DecrBefore = 2,
    IncrBefore = 3,
}

pub enum AddrMode2 {
    Offset,
    PreIndexed,
    PostIndexed,
}

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
                    Op::Subi(rn, Reg::PC, -imm as u32).into()
                } else {
                    Op::Addi(rn, Reg::PC, imm as u32).into()
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
                Reg::PC,
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

/// https://documentation-service.arm.com/static/5f8daeb7f86e16515cdb8c4e
impl TinyAsm {
    /// Encoding of ADD (immediate): `ADD <Rd>, <Rn>, #<uimm12>`.
    pub fn addi(self, rd: Reg, rn: Option<Reg>, imm: u16) -> Self {
        self.op(Op::Addi(rd, rn.unwrap_or(rd), imm as u32))
    }

    /// Encoding of ADR: `ADR <Rd>, <label>`.
    pub fn adrl(mut self, rd: Reg, label: Label) -> Self {
        self.relocs.push((self.buf.len(), Op::Adrl(rd, label)));
        self.op(Op::Placeholder)
    }

    /// Encoding of LDMIA: `LDMIA <Rn>{!}, <registers>`.
    pub fn ldmia<const T: usize>(self, rn: Reg, wb: bool, regs: [Reg; T]) -> Self {
        self.op(Op::Ldm(AddrMode::IncrAfter, rn, wb, regs.to_vec()))
    }

    /// Encoding of MOV (register): `MOV <Rd>, <Rm>`.
    pub fn movr(self, rd: Reg, rm: Reg) -> Self {
        self.op(Op::Movr(rd, rm))
    }

    /// Encoding of MOVW (immediate): `MOVW <Rd>, #<imm16>`.
    pub fn movw(self, rd: Reg, imm: u16) -> Self {
        self.op(Op::Movw(rd, imm as u32))
    }

    /// Encoding of LDR (immediate): `LDR <Rt>, [<Rn>{, #+/-<imm12>}]`, `LDR<Rt>, [<Rn>], #+/-<imm12>`, `LDR <Rt>, [<Rn>, #+/-<imm12>]!`.
    pub fn ldri(self, mode: AddrMode2, rn: Reg, rt: Reg, imm: i16) -> Self {
        self.op(Op::Ldri(mode, rn, rt, imm))
    }

    /// Encoding of LDR (label): `LDR <Rt>, <label>`.
    pub fn ldrl(mut self, rn: Reg, label: Label) -> Self {
        self.relocs.push((self.buf.len(), Op::Ldrl(rn, label)));
        self.op(Op::Placeholder)
    }

    /// Encoding of POP: `POP <registers>`.
    pub fn pop<const T: usize>(self, regs: [Reg; T]) -> Self {
        self.ldmia(Reg::SP, true, regs)
    }

    /// Encoding of PUSH: `PUSH <registers>`.
    pub fn push<const T: usize>(self, regs: [Reg; T]) -> Self {
        self.stmdb(Reg::SP, true, regs)
    }

    /// Encoding of STMDB: `STMDB <Rn>{!}, <registers>`.
    pub fn stmdb<const T: usize>(self, rn: Reg, wb: bool, regs: [Reg; T]) -> Self {
        self.op(Op::Stm(AddrMode::DecrBefore, rn, wb, regs.to_vec()))
    }

    /// Encoding of SUB (immediate): `SUB <Rd>, <Rn>, #<uimm12>`.
    pub fn subi(self, rd: Reg, rn: Option<Reg>, imm: u16) -> Self {
        self.op(Op::Subi(rd, rn.unwrap_or(rd), imm as u32))
    }

    /// Encoding of SVC: `SVC #<imm24>`.
    pub fn svc(self, imm: u32) -> Self {
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
        self.val() << rhs as Self::Output
    }
}

impl Shl<u32> for AddrMode {
    type Output = u32;

    fn shl(self, rhs: u32) -> Self::Output {
        (self as Self::Output) << rhs
    }
}

pub type TinyAsm = super::TinyAsm<Op, 4>;

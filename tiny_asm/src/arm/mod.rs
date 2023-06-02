mod addr_mode;
mod addr_mode_2;
mod op;
mod reg;

pub use addr_mode::AddrMode;
pub use addr_mode_2::AddrMode2;
pub use op::Op;
pub use reg::Reg;

use super::{Encodable, Label};

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
        self.ldmia(Reg::sp, true, regs)
    }

    /// Encoding of PUSH: `PUSH <registers>`.
    pub fn push<const T: usize>(self, regs: [Reg; T]) -> Self {
        self.stmdb(Reg::sp, true, regs)
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

pub type TinyAsm = super::TinyAsm<Op, 4>;

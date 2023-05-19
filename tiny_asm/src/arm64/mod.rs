mod addr_mode_2;
mod op;
mod reg;
mod shift;

pub use addr_mode_2::AddrMode2;
pub use op::Op;
pub use reg::Reg;
pub use shift::Shift;

use super::Label;

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

pub type TinyAsm = super::TinyAsm<Op, 4>;

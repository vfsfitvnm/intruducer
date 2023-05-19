mod op;

pub use op::Op;

use crate::Label;

impl TinyAsm {
    pub fn instr<const T: usize>(mut self, bytes: [u8; T]) -> Self {
        self.buf.extend(bytes);
        self
    }

    pub fn instr_with_ref<const T: usize>(mut self, bytes: [u8; T], label: Label) -> Self {
        self.buf.extend(bytes);
        self.relocs.push((self.buf.len(), Op::Refl(label)));
        self.op(Op::Placeholder)
    }
}

pub type TinyAsm = super::TinyAsm<Op, 4>;

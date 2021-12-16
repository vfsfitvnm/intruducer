use std::collections::HashMap;

use super::{Encodable, Label};

pub(crate) enum Op {
    Ref(Label),
    Placeholder,
}

impl Encodable<4> for Op {
    fn enc(self, instr_offset: usize, labels: &HashMap<Label, usize>) -> [u8; 4] {
        match self {
            Op::Placeholder => 0,
            Op::Ref(label) => Self::res_lab(label, labels, instr_offset),
        }
        .to_le_bytes()
    }

    fn calc_offset(_: i32, label_offset: i32) -> i32 {
        label_offset
    }
}

impl TinyAsm {
    pub(crate) fn instr<const T: usize>(mut self, bytes: [u8; T]) -> Self {
        self.buf.extend(bytes);
        self
    }

    pub(crate) fn instr_with_ref<const T: usize>(mut self, bytes: [u8; T], label: Label) -> Self {
        self.buf.extend(bytes);
        self.relocs.push((self.buf.len(), Op::Ref(label)));
        self.op(Op::Placeholder)
    }
}

pub(crate) type TinyAsm = super::TinyAsm<Op, 4>;

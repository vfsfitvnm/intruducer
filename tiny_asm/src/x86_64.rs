use std::{collections::HashMap, mem::size_of};

use super::{Encodable, Label};

pub enum Op {
    PlaceHolder,
    Refl(Label),
}

impl Encodable<4> for Op {
    fn enc(self, instr_offset: usize, labels: &HashMap<Label, usize>) -> [u8; 4] {
        match self {
            Op::Refl(label) => Self::res_lab(label, labels, instr_offset),
            Op::PlaceHolder => 0,
        }
        .to_le_bytes()
    }

    fn calc_offset(instr_offset: i32, label_offset: i32) -> i32 {
        label_offset - instr_offset - size_of::<i32>() as i32
    }
}

impl TinyAsm {
    pub fn instr<const T: usize>(mut self, bytes: [u8; T]) -> Self {
        self.buf.extend(bytes);
        self
    }

    pub fn instr_with_ref<const T: usize>(mut self, bytes: [u8; T], label: Label) -> Self {
        self.buf.extend(bytes);
        self.relocs.push((self.buf.len(), Op::Refl(label)));
        self.op(Op::PlaceHolder)
    }
}

pub type TinyAsm = super::TinyAsm<Op, 4>;

use std::collections::HashMap;

use crate::{Encodable, Label};

pub enum Op {
    Placeholder,
    Ref(Label),
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

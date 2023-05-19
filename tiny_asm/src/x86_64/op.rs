use std::{collections::HashMap, mem::size_of};

use crate::{Encodable, Label};

pub enum Op {
    Placeholder,
    Refl(Label),
}

impl Encodable<4> for Op {
    fn enc(self, instr_offset: usize, labels: &HashMap<Label, usize>) -> [u8; 4] {
        match self {
            Op::Refl(label) => Self::res_lab(label, labels, instr_offset),
            Op::Placeholder => 0,
        }
        .to_le_bytes()
    }

    fn calc_offset(instr_offset: i32, label_offset: i32) -> i32 {
        label_offset - instr_offset - size_of::<i32>() as i32
    }
}

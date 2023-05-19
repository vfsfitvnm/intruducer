use std::collections::HashMap;

use crate::Label;

/// An instruction which can be encoded to `T` bytes.
/// E.g. every `arm` and `arm64` intruction is encoded to 4 bytes or 32-bit unsigned integer.
/// Unfortunately, `x86` and `x86-64` can't follow this beautiful pattern, so they will receive less love.
pub trait Encodable<const T: usize> {
    /// Common function to grab the given label from the labels hash map, panicking on failure.
    fn res_lab(lab: Label, labs: &HashMap<Label, usize>, instr_offset: usize) -> i32 {
        let offset = labs
            .get(lab)
            .unwrap_or_else(|| panic!("Couldn't find label {}", lab));

        Self::calc_offset(
            instr_offset.try_into().unwrap(),
            (*offset).try_into().unwrap(),
        )
    }

    /// Relocation calculation implementation.
    fn calc_offset(instr_offset: i32, label_offset: i32) -> i32;

    /// Instructions encoding implementation.
    fn enc(self, offset: usize, labels: &HashMap<Label, usize>) -> [u8; T];
}

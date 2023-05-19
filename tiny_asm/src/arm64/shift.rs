use std::ops::Shl;

pub enum Shift {
    Lsr = 0,
    Asr = 1,
    Lsl = 2,
    Ror = 3,
}

impl Shl<u32> for Shift {
    type Output = u32;

    fn shl(self, rhs: u32) -> Self::Output {
        (self as u32) << rhs
    }
}

use std::ops::Shl;

pub enum AddrMode {
    DecrAfter = 0,
    IncrAfter = 1,
    DecrBefore = 2,
    IncrBefore = 3,
}

impl Shl<u32> for AddrMode {
    type Output = u32;

    fn shl(self, rhs: u32) -> Self::Output {
        (self as Self::Output) << rhs
    }
}

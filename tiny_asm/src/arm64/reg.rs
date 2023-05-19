use std::ops::{BitOr, Shl};

#[derive(Clone, Copy, PartialEq)]
pub enum Reg {
    X0 = 0,
    X1 = 1,
    X2 = 2,
    X3 = 3,
    X4 = 4,
    X5 = 5,
    X6 = 6,
    X7 = 7,
    X8 = 8,
    X9 = 9,
    X10 = 10,
    X11 = 11,
    X12 = 12,
    X13 = 13,
    X14 = 14,
    X15 = 15,
    X16 = 16,
    X17 = 17,
    X18 = 18,
    X19 = 19,
    X20 = 20,
    X21 = 21,
    X22 = 22,
    X23 = 23,
    X24 = 24,
    X25 = 25,
    X26 = 26,
    X27 = 27,
    X28 = 28,
    X29 = 29,
    X30 = 30,
    SP,
    XZR,
}

impl Reg {
    const fn val(&self) -> u32 {
        match *self {
            Reg::SP | Reg::XZR => 31,
            reg => reg as u32,
        }
    }
}

impl BitOr<Reg> for u32 {
    type Output = u32;

    fn bitor(self, rhs: Reg) -> Self::Output {
        self | rhs.val()
    }
}

impl Shl<Reg> for u32 {
    type Output = u32;

    fn shl(self, rhs: Reg) -> Self::Output {
        self << rhs.val()
    }
}

impl Shl<u32> for Reg {
    type Output = u32;

    fn shl(self, rhs: u32) -> Self::Output {
        self.val() << rhs as u32
    }
}

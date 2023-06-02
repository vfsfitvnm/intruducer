use std::ops::{BitOr, Shl};

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq)]
pub enum Reg {
    x0 = 0,
    x1 = 1,
    x2 = 2,
    x3 = 3,
    x4 = 4,
    x5 = 5,
    x6 = 6,
    x7 = 7,
    x8 = 8,
    x9 = 9,
    x10 = 10,
    x11 = 11,
    x12 = 12,
    x13 = 13,
    x14 = 14,
    x15 = 15,
    x16 = 16,
    x17 = 17,
    x18 = 18,
    x19 = 19,
    x20 = 20,
    x21 = 21,
    x22 = 22,
    x23 = 23,
    x24 = 24,
    x25 = 25,
    x26 = 26,
    x27 = 27,
    x28 = 28,
    x29 = 29,
    x30 = 30,
    sp,
    xzr,
}

impl Reg {
    const fn val(&self) -> u32 {
        match *self {
            Reg::sp | Reg::xzr => 31,
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

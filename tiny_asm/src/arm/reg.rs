use std::ops::{BitOr, Shl};

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq)]
pub enum Reg {
    r0 = 0,
    r1 = 1,
    r2 = 2,
    r3 = 3,
    r4 = 4,
    r5 = 5,
    r6 = 6,
    r7 = 7,
    r8 = 8,
    r9 = 9,
    r10 = 10,
    r11 = 11,
    r12 = 12,
    r13 = 13,
    r14 = 14,
    r15 = 15,
    sp,
    lr,
    pc,
}

impl Reg {
    const fn val(self) -> u32 {
        match self {
            Reg::sp => Reg::r13.val(),
            Reg::lr => Reg::r14.val(),
            Reg::pc => Reg::r15.val(),
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
        self.val() << rhs as Self::Output
    }
}

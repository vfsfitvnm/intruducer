use std::collections::HashMap;

#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
pub mod arm;
#[cfg(target_arch = "aarch64")]
pub mod arm64;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod x86;
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

pub(crate) type Label = &'static str;

pub(crate) trait Encodable<const T: usize> {
    fn res_lab(lab: Label, labs: &HashMap<Label, usize>, instr_offset: usize) -> i32 {
        let offset = labs
            .get(lab)
            .unwrap_or_else(|| panic!("Couldn't find label {}", lab));

        Self::calc_offset(
            instr_offset.try_into().unwrap(),
            (*offset).try_into().unwrap(),
        )
    }

    fn calc_offset(instr_offset: i32, label_offset: i32) -> i32;

    fn enc(self, offset: usize, labels: &HashMap<Label, usize>) -> [u8; T];
}

pub(crate) struct TinyAsm<T: Encodable<U>, const U: usize> {
    buf: Vec<u8>,
    relocs: Vec<(usize, T)>,
    labels: HashMap<Label, usize>,
}

impl<T: Encodable<U>, const U: usize> TinyAsm<T, U> {
    pub(crate) fn new() -> Self {
        TinyAsm {
            buf: Default::default(),
            relocs: Default::default(),
            labels: Default::default(),
        }
    }

    pub(crate) fn align<const A: usize>(mut self) -> Self {
        while self.buf.len() % A != 0 {
            self.buf.push(0);
        }
        self
    }

    pub(crate) fn ascii(self, str: &str) -> Self {
        self.bytes(str.as_bytes())
    }

    pub(crate) fn asciiz(self, str: &str) -> Self {
        self.ascii(str).bytes(&[0])
    }

    pub(crate) fn bytes(mut self, bytes: &[u8]) -> Self {
        self.buf.extend(bytes);
        self
    }

    pub(crate) fn word(self, word: u16) -> Self {
        self.bytes(&word.to_le_bytes())
    }

    pub(crate) fn dword(self, dword: u32) -> Self {
        self.bytes(&dword.to_le_bytes())
    }

    pub(crate) fn qword(self, qword: u64) -> Self {
        self.bytes(&qword.to_le_bytes())
    }

    pub(crate) fn op(mut self, op: T) -> Self {
        self.buf.extend(op.enc(0, &self.labels));
        self
    }

    pub(crate) fn label(mut self, label: Label) -> Self {
        self.labels.insert(label, self.buf.len());
        self
    }

    pub(crate) fn build(mut self) -> Vec<u8> {
        for (index, op) in self.relocs {
            for (j, byte) in op.enc(index, &self.labels).iter().enumerate() {
                self.buf[index + j] = *byte;
            }
        }

        self.buf
    }
}

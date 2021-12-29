use std::collections::HashMap;

pub mod arm;
pub mod arm64;
pub mod x86;
pub mod x86_64;
pub struct TinyAsm<T: Encodable<U>, const U: usize> {
    buf: Vec<u8>,
    relocs: Vec<(usize, T)>,
    labels: HashMap<Label, usize>,
}

impl<T: Encodable<U>, const U: usize> TinyAsm<T, U> {
    pub fn new() -> Self {
        TinyAsm {
            buf: Default::default(),
            relocs: Default::default(),
            labels: Default::default(),
        }
    }

    pub fn align<const A: usize>(mut self) -> Self {
        while self.buf.len() % A != 0 {
            self.buf.push(0);
        }
        self
    }

    pub fn ascii(self, str: &str) -> Self {
        self.bytes(str.as_bytes())
    }

    pub fn asciiz(self, str: &str) -> Self {
        self.ascii(str).bytes(&[0])
    }

    pub fn bytes(mut self, bytes: &[u8]) -> Self {
        self.buf.extend(bytes);
        self
    }

    pub fn word(self, word: u16) -> Self {
        self.bytes(&word.to_le_bytes())
    }

    pub fn dword(self, dword: u32) -> Self {
        self.bytes(&dword.to_le_bytes())
    }

    pub fn qword(self, qword: u64) -> Self {
        self.bytes(&qword.to_le_bytes())
    }

    pub fn op(mut self, op: T) -> Self {
        self.buf.extend(op.enc(0, &self.labels));
        self
    }

    pub fn label(mut self, label: Label) -> Self {
        self.labels.insert(label, self.buf.len());
        self
    }

    pub fn build(mut self) -> Vec<u8> {
        for (index, op) in self.relocs {
            for (j, byte) in op.enc(index, &self.labels).iter().enumerate() {
                self.buf[index + j] = *byte;
            }
        }

        self.buf
    }
}

pub trait Encodable<const T: usize> {
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

pub type Label = &'static str;
use std::collections::HashMap;

mod encodable;
mod label;

#[cfg(feature = "arm")]
pub mod arm;
#[cfg(feature = "arm64")]
pub mod arm64;
#[cfg(feature = "x86")]
pub mod x86;
#[cfg(feature = "x86_64")]
pub mod x86_64;

pub use encodable::Encodable;
pub use label::Label;

/// Holds the relevant basic stuff to perform intructions encoding and relocations.
/// Every instruction is encoded immediately and pushed into the buffer. If it contains
/// a reference to a label (so the instruction cannot be encoded now), placeholder bytes
/// are pushed instead; also the insert position and the original instruction are pushed
/// into a vector. The final step (the call to [`TinyAsm::build`]) will iterate the
/// relocation vector and replace the placeholder bytes with the re-encoded instruction.
/// Little-endian only.
pub struct TinyAsm<T: Encodable<U>, const U: usize> {
    buf: Vec<u8>,
    relocs: Vec<(usize, T)>,
    labels: HashMap<Label, usize>,
}

impl<T: Encodable<U>, const U: usize> TinyAsm<T, U> {
    /// Creates a new assembler.
    pub fn new() -> Self {
        Self {
            buf: Default::default(),
            relocs: Default::default(),
            labels: Default::default(),
        }
    }

    /// Aligns the buffer to a `A` byte boundary.
    pub fn align<const A: usize>(mut self) -> Self {
        while self.buf.len() % A != 0 {
            self.buf.push(0);
        }
        self
    }

    /// Pushes a string into the buffer.
    pub fn ascii(self, str: &str) -> Self {
        self.bytes(str.as_bytes())
    }

    /// Pushes a string and one NUL char into the buffer.
    pub fn asciiz(self, str: &str) -> Self {
        self.ascii(str).bytes(&[0])
    }

    /// Pushes raw bytes into the buffer.
    pub fn bytes(mut self, bytes: &[u8]) -> Self {
        self.buf.extend(bytes);
        self
    }

    // TODO: this is incorrect, figure out sign
    pub fn word(self, word: u16) -> Self {
        self.bytes(&word.to_le_bytes())
    }

    // TODO: this is incorrect, figure out sign
    pub fn dword(self, dword: u32) -> Self {
        self.bytes(&dword.to_le_bytes())
    }

    // TODO: this is incorrect, figure out sign
    pub fn qword(self, qword: u64) -> Self {
        self.bytes(&qword.to_le_bytes())
    }

    /// Pushes the encoding of a instruction into the buffer.
    pub fn op(mut self, op: T) -> Self {
        self.buf.extend(op.enc(0, &self.labels));
        self
    }

    /// Puts a label at the current position (current buffer length).
    pub fn label(mut self, label: Label) -> Self {
        self.labels.insert(label, self.buf.len());
        self
    }

    /// Performs the relocation of every pending instruction encoding, which
    /// couldn't be encoded early because it contained a reference to a label.
    pub fn build(mut self) -> Vec<u8> {
        for (index, op) in self.relocs {
            for (j, byte) in op.enc(index, &self.labels).iter().enumerate() {
                self.buf[index + j] = *byte;
            }
        }

        self.buf
    }
}

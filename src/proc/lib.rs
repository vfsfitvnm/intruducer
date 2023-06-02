use std::path::PathBuf;

use goblin::elf::Elf;

use crate::{ext::ElfExt, os::VirtAddr};

use super::sym::ProcSym;

/// A struct that represents a loaded shared library.
pub(crate) struct ProcLib {
    /// The base virtual address where the library is located at.
    pub(crate) base_addr: VirtAddr,

    /// The path where the library is located at.
    pub(crate) path: PathBuf,
}

impl ProcLib {
    /// Creates a new [`ProcLib`] that references a shared library loaded at `base_add` and located at `path`.
    pub(crate) fn new(base_addr: u64, path: PathBuf) -> Self {
        ProcLib { base_addr, path }
    }

    /// Finds a symbol with the given `name` exported by the current library.
    ///
    /// Returns [`None`] if no symbol with the given name was found.
    pub(crate) fn find_sym_addr<const N: usize>(&self, names: [&str; N]) -> Option<ProcSym> {
        let buf = std::fs::read(&self.path).ok()?;
        let elf = Elf::parse(&buf).ok()?;

        let sym = names.iter().find_map(|name| elf.find_sym_by_name(name))?;
        Some(ProcSym::new(self.base_addr + sym.st_value))
    }
}

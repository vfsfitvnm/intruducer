use goblin::{
    elf::{Elf, Sym, Symtab},
    strtab::Strtab,
};

/// A extension trait for [`Elf`].
pub(crate) trait ElfExt {
    /// Finds the symbol with the given `name` exported by the current [`Elf`] file.
    ///
    /// Returns [`None`] if no symbols was found.
    fn find_sym_by_name(&self, sym_name: &str) -> Option<Sym>;
}

impl<'a> ElfExt for Elf<'a> {
    fn find_sym_by_name(&self, name: &str) -> Option<Sym> {
        let iter = |syms: &Symtab, strtab: &Strtab| -> Option<Sym> {
            for sym in syms.iter() {
                if let Some(cur_name) = strtab.get_at(sym.st_name) {
                    if cur_name == name {
                        return Some(sym);
                    }
                }
            }

            None
        };

        iter(&self.syms, &self.strtab).or_else(|| iter(&self.dynsyms, &self.dynstrtab))
    }
}

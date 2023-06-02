use crate::os::VirtAddr;

/// A struct that represents a library symbol.
pub(crate) struct ProcSym {
    /// The virtual address where the symbol is located at.
    pub(crate) addr: VirtAddr,
}

impl ProcSym {
    /// Creates a new [`ProcSym`] that references a symbol located at `addr`.
    pub(crate) fn new(addr: VirtAddr) -> Self {
        ProcSym { addr }
    }
}

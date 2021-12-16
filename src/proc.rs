use std::{
    fs::{read_to_string, File, OpenOptions, ReadDir},
    io::Error as IoError,
    os::unix::prelude::MetadataExt,
    path::PathBuf,
};

use goblin::elf::Elf;

use crate::{
    ext::{ElfExt, PathBufExt},
    Gid, ProcId, Uid, VirtAddr,
};

/// A newtype that references the [`/proc/<id>`](https://man7.org/linux/man-pages/man5/proc.5.html) directory.
pub(crate) struct Proc(pub(crate) PathBuf);

impl Proc {
    /// Creates a new [`Proc`] that references the host process.
    pub(crate) fn current() -> Self {
        Proc(PathBuf::root().join("proc").join("self"))
    }

    /// Creates a new [`Proc`] that references the task identified by `id`.
    ///
    /// Returns [`None`] if the path `/proc/<id>` does not exist.
    pub(crate) fn new(id: ProcId) -> Option<Self> {
        let path = PathBuf::root().join("proc").join(id.to_string());
        path.exists().then(|| Proc(path))
    }

    /// Gets the owner of the current [`Proc`].
    pub(crate) fn owner(&self) -> Result<(Uid, Gid), IoError> {
        let metadata = self.0.metadata()?;
        Ok((metadata.uid(), metadata.gid()))
    }

    /// Reads `/proc/<id>/exe` of the current [`Proc`].
    pub(crate) fn exe(&self) -> Result<File, IoError> {
        File::open(self.0.join("exe"))
    }

    /// Reads `/proc/<id>/maps` of the current [`Proc`].
    pub(crate) fn maps(&self) -> Result<File, IoError> {
        File::open(self.0.join("maps"))
    }

    /// Reads `/proc/<id>/mem` of the current [`Proc`].
    pub(crate) fn mem(&self) -> Result<File, IoError> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.0.join("mem"))
    }

    /// Reads `/proc/<id>/syscall` of the current [`Proc`].
    pub(crate) fn syscall(&self) -> Result<File, IoError> {
        File::open(self.0.join("syscall"))
    }

    /// Reads `/proc/<id>/task` of the current [`Proc`].
    pub(crate) fn task(&self) -> Result<ReadDir, IoError> {
        std::fs::read_dir(self.0.join("task"))
    }
}

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
    pub(crate) fn find_sym_addr(&self, name: &str) -> Option<ProcSym> {
        let buf = std::fs::read(&self.path).ok()?;
        let sym = Elf::parse(&buf).ok()?.find_sym_by_name(name)?;
        Some(ProcSym::new(self.base_addr + sym.st_value))
    }
}

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

/// A enum that represents the class of a process (32 bit or 64 bit).
pub(crate) enum ProcClass {
    /// The values used to describe 32 bit processes.
    #[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
    ThirtyTwo,
    /// The values used to describe 64 bit processes.
    #[cfg(target_pointer_width = "64")]
    SixtyFour,
}

/// A enum that represents the content of `/proc/sys/kernel/yama/ptrace_scope`.
///
/// Even if we do not use `ptrace`, `/proc/<pid>/` readability depends on this value.
///
/// Source: https://man7.org/linux/man-pages/man2/ptrace.2.html
pub(crate) enum PtraceScope {
    /// 0.
    All,

    /// 1.
    Restricted,

    /// 2.
    Admin,

    /// 3.
    None,
}

impl PtraceScope {
    /// Gets the content of `/proc/sys/kernel/yama/ptrace_scope`, which determines whether we can read `/proc/<pid>` or not.
    /// If the file doesn't exists, we assume the kernel was not built with the Yama Linux Security Module.
    pub(crate) fn current() -> Self {
        if let Ok(scope) = read_to_string("/proc/sys/kernel/yama/ptrace_scope") {
            match scope.trim() {
                "0" => PtraceScope::All,
                "1" => PtraceScope::Restricted,
                "2" => PtraceScope::Admin,
                "3" => PtraceScope::None,
                _ => unreachable!(),
            }
        } else {
            PtraceScope::All
        }
    }
}

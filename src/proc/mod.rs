use std::{
    fs::{File, OpenOptions, ReadDir},
    io::Error as IoError,
    os::unix::prelude::MetadataExt,
    path::PathBuf,
};

mod class;
mod id;
mod lib;
mod sym;

use crate::{
    ext::PathBufExt,
    os::{Gid, Uid},
};

pub(crate) use class::ProcClass;
pub(crate) use id::ProcId;
pub(crate) use lib::ProcLib;
pub(crate) use sym::ProcSym;

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
        path.exists().then_some(Proc(path))
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

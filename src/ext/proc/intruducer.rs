use crate::{
    constants::DLOPEN_SYM_NAMES,
    os::VirtAddr,
    proc::{Proc, ProcSym},
    Error,
};

use super::ProcExt;

/// A extension trait for [`Proc`] specific to this crate.
pub(crate) trait ProcIntruducerExt {
    /// Looks for `dlopen` symbol into this process.
    ///
    /// Returns [`Error`] if it was not found.
    fn find_dlopen(&self) -> Result<ProcSym, Error>;

    /// Retrieves the instruction pointer of this process, looking into other threads until is found.
    ///
    /// Returns [`Error`] if it was not found.
    fn find_ip(&self) -> Result<VirtAddr, Error>;
}

impl ProcIntruducerExt for Proc {
    fn find_dlopen(&self) -> Result<ProcSym, Error> {
        let dlopen_lib_name = get_dlopen_lib_name();

        let dlopen_lib = self
            .find_lib_by_name(&dlopen_lib_name)
            .ok_or(Error::LibraryNotFound(dlopen_lib_name))?;

        dlopen_lib
            .find_sym_addr(DLOPEN_SYM_NAMES)
            .ok_or_else(|| Error::SymbolNotFound(DLOPEN_SYM_NAMES.to_vec()))
    }

    fn find_ip(&self) -> Result<VirtAddr, Error> {
        self.ip()
            .or_else(|| {
                self.task()
                    .unwrap()
                    .filter_map(|dir| dir.ok())
                    .find_map(|dir| Proc(dir.path()).ip())
            })
            .ok_or(Error::InstructionPointerNotFound)
    }
}

use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
};

#[cfg(target_os = "linux")]
fn get_dlopen_lib_name() -> String {
    BufReader::new(Proc::current().maps().unwrap())
        .lines()
        .filter_map(|line| line.ok())
        .find_map(|line| {
            let path: PathBuf = line.rsplit_once("    ")?.1.into();
            let file_name = path.file_name()?.to_str()?;

            if file_name.starts_with("libc.") || file_name.starts_with("libc-") {
                Some(file_name.to_owned())
            } else {
                None
            }
        })
        .unwrap()
}

#[cfg(target_os = "android")]
fn get_dlopen_lib_name() -> String {
    "libdl.so".to_string()
}

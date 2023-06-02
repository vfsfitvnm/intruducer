use std::{
    io::{BufRead, BufReader, Read},
    path::PathBuf,
};

use goblin::elf::Elf;

#[cfg(any(target_arch = "i386", target_arch = "x86_64"))]
use goblin::elf::header::EM_386;
#[cfg(target_arch = "aarch64")]
use goblin::elf::header::EM_AARCH64;
#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
use goblin::elf::header::EM_ARM;
#[cfg(target_arch = "x86_64")]
use goblin::elf::header::EM_X86_64;

#[cfg(target_os = "android")]
mod android;
mod intruducer;

#[cfg(target_os = "android")]
pub(crate) use android::ProcAndroidExt;
pub(crate) use intruducer::ProcIntruducerExt;

use crate::{
    os::VirtAddr,
    proc::{Proc, ProcClass, ProcLib},
};

/// A extension trait for [`Proc`].
pub(crate) trait ProcExt {
    /// Finds the shared library with the given `name` inside the current process.
    ///
    /// Returns [`None`] if no library with the current name was found.
    fn find_lib_by_name(&self, lib_name: &str) -> Option<ProcLib>;

    /// Determines the class of the current process - if it's running in 32 bit or 64 bit mode.
    ///
    /// Returns [`None`] if the process instruction set is not supported.
    fn class(&self) -> Option<ProcClass>;

    /// Gets the instruction pointer of the current process.
    ///
    /// Returns [`None`] if the process is not blocked.
    fn ip(&self) -> Option<VirtAddr>;

    /// Determines wheter the current process is priviliged, e.g. if its owner is the superuser.
    fn privileged(&self) -> bool;
}

impl ProcExt for Proc {
    fn find_lib_by_name(&self, name: &str) -> Option<ProcLib> {
        BufReader::new(self.maps().ok()?)
            .lines()
            .filter_map(|line| line.ok())
            .find_map(|line| {
                let path: PathBuf = line.rsplit_once("    ")?.1.into();

                if path.file_name()? == name {
                    let base_add = line.split_once('-')?.0;
                    let base_add = VirtAddr::from_str_radix(base_add, 16).ok()?;

                    Some(ProcLib::new(base_add, path))
                } else {
                    None
                }
            })
    }

    // TODO: it should not check if the instruction set is supported.
    fn class(&self) -> Option<ProcClass> {
        let mut header = [0_u8; 0x40];
        self.exe().ok()?.read_exact(&mut header).ok()?;

        let header = Elf::parse_header(&header).ok()?;

        match header.e_machine {
            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            EM_ARM => Some(ProcClass::ThirtyTwo),
            #[cfg(target_arch = "aarch64")]
            EM_AARCH64 => Some(ProcClass::SixtyFour),
            #[cfg(any(target_arch = "i386", target_arch = "x86_64"))]
            EM_386 => Some(ProcClass::ThirtyTwo),
            #[cfg(target_arch = "x86_64")]
            EM_X86_64 => Some(ProcClass::SixtyFour),
            _ => None,
        }
    }

    fn ip(&self) -> Option<VirtAddr> {
        let mut content = String::new();
        self.syscall().ok()?.read_to_string(&mut content).ok()?;

        let ip = content.trim().rsplit_once('x')?.1;

        VirtAddr::from_str_radix(ip, 16).ok()
    }

    fn privileged(&self) -> bool {
        self.owner().unwrap().0 == 0
    }
}

use std::{
    io::{BufRead, BufReader, Read},
    path::PathBuf,
};

#[cfg(any(target_arch = "i386", target_arch = "x86_64"))]
use goblin::elf::header::EM_386;
#[cfg(target_arch = "aarch64")]
use goblin::elf::header::EM_AARCH64;
#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
use goblin::elf::header::EM_ARM;
#[cfg(target_arch = "x86_64")]
use goblin::elf::header::EM_X86_64;

use goblin::{
    elf::{Elf, Sym, Symtab},
    strtab::Strtab,
};

use crate::{Error, VirtAddr, proc::{Proc, ProcClass, ProcLib, ProcSym}};

/// A extension trait for [`PathBuf`].
pub(crate) trait PathBufExt {
    /// Gets the root [`PathBuf`].
    fn root() -> Self;
}

impl PathBufExt for PathBuf {
    fn root() -> Self {
        "/".into()
    }
}

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

/// A extension trait for [`Proc`] specific to the Android operative system.
#[cfg(target_os = "android")]
pub(crate) trait ProcAndroidExt {
    /// Determines the native library directory of the current application, e.g. `/data/app/com.example.application-.../lib/<abi>`.
    ///
    /// Returns [`None`] if the current process is not an Android application.
    fn get_app_lib_dir(&self) -> Option<PathBuf>;

    /// Determines the package name the current Android application, e.g `com.example.application`.
    ///
    /// Returns [`None`] if the current process is not an Android application.

    fn get_app_name(&self) -> Option<String>;
}

#[cfg(target_os = "android")]
impl ProcAndroidExt for Proc {
    fn get_app_lib_dir(&self) -> Option<PathBuf> {
        let package_name = self.get_app_name()?;

        let arch_name = match self.class()? {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            ProcClass::ThirtyTwo => "i386",
            #[cfg(target_arch = "x86_64")]
            ProcClass::SixtyFour => "x86_64",
            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            ProcClass::ThirtyTwo => "arm",
            #[cfg(target_arch = "aarch64")]
            ProcClass::SixtyFour => "arm64",
        };

        std::fs::read_dir("/data/app")
            .ok()?
            .filter_map(|entry| entry.ok())
            .find_map(|entry| {
                let name = entry.file_name().into_string().ok()?;
                if name.starts_with(&package_name) && name.chars().nth(package_name.len())? == '-' {
                    Some(entry.path().join("lib").join(arch_name))
                } else {
                    None
                }
            })
    }

    fn get_app_name(&self) -> Option<String> {
        use crate::Uid;

        let (uid, _) = self.owner().ok()?;

        BufReader::new(std::fs::File::open("/data/system/packages.list").ok()?)
            .lines()
            .filter_map(|line| line.ok())
            .find_map(|line| {
                let (name, line) = line.split_once(' ')?;
                let raw_uid = line.split_once(' ')?.0;

                if raw_uid.parse::<Uid>().unwrap_or(0) == uid {
                    Some(name.to_string())
                } else {
                    None
                }
            })
    }
}

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
            .find_sym_addr(DLOPEN_SYM_NAME)
            .ok_or_else(|| Error::SymbolNotFound(DLOPEN_SYM_NAME.into()))
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


#[cfg(target_os = "linux")]
const DLOPEN_SYM_NAME: &str = "__libc_dlopen_mode";

#[cfg(target_os = "android")]
const DLOPEN_SYM_NAME: &str = "dlopen";

#[cfg(target_os = "linux")]
fn get_dlopen_lib_name() -> String {
    use std::{ffi::CStr, os::raw::c_char};

    #[link(name = "c")]
    extern "C" {
        fn gnu_get_libc_version() -> *const c_char;
    }

    let version = unsafe { CStr::from_ptr(gnu_get_libc_version()).to_str().unwrap() };
    format!("libc-{}.so", version)
}

#[cfg(target_os = "android")]
fn get_dlopen_lib_name() -> String {
    "libdl.so".to_string()
}
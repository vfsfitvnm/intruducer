//! A Rust crate to load a shared library into a target process without using `ptrace`.
//! This is a portable rewrite of [dlinject](https://github.com/DavidBuchanan314/dlinject).

use std::fs::File;
use std::io::{Error as IoError, Write};
use std::ops::Not;
use std::os::unix::prelude::FileExt;
use std::path::PathBuf;

mod ext;
mod proc;
mod utils;

use crate::ext::ProcExt;
use crate::ext::ProcIntruducerExt;
use crate::proc::Proc;
use crate::proc::PtraceScope;
use crate::utils::{first_payload, second_payload};

/// Loads a shared library into the target process.
///
/// `id` is either a process or thread (process task) identifier, e.g any entry of `/proc` is allowed.
///
/// `lib_path` corresponds to the `filename` argument of [`dlopen`](https://man7.org/linux/man-pages/man3/dlopen.3.html).
/// Due to the linker namespaces isolation on Android applications, only pathnames are accepted.
///
/// Returns [`Error`] if the operation fails.
///
/// Examples:
///
/// A library can be provided throught a relative or absolute path.
/// ```no_run
/// use intruducer::intruduce;
///
/// intruduce(1234, "/path/to/lib.so")?;
/// ```
///
/// A system library can be provided throught a name.
/// This will lead to an error when targeting Android applications.
/// ```no_run
/// use intruducer::intruduce;
///
/// intruduce(1234, "libsystem.so")?;
/// ```
pub fn intruduce(id: ProcId, lib_path: PathBuf) -> Result<(), Error> {
    let proc = Proc::new(id).ok_or(Error::ProcessNotRunning)?;

    match PtraceScope::current() {
        // We must be superuser if the target is superuser
        PtraceScope::All => proc.privileged().not() || Proc::current().privileged(),
        // We must be superuser
        PtraceScope::Restricted | PtraceScope::Admin => Proc::current().privileged(),
        // There's nothing we can do about this
        PtraceScope::None => false,
    }
    .then(|| ())
    .ok_or(Error::InsufficientPriviliges)?;

    #[cfg(target_os = "android")]
    // Adjusts the library and second payload file path in case the target process is an Android application.
    {
        use crate::ext::ProcAndroidExt;

        if let Some(lib_dir) = proc.get_app_lib_dir() {
            // We must be superuser if the target process is an Android application
            Proc::current()
                .privileged()
                .then(|| ())
                .ok_or(Error::InsufficientPriviliges)?;

            let lib_path = lib_path
                .canonicalize()
                .ok()
                .ok_or(Error::LibraryPathNeeded)?;

            let lib_name = lib_path.file_name().unwrap();

            let new_lib_path = lib_dir.join(lib_name);

            if !new_lib_path.exists() {
                std::fs::copy(&lib_path, &new_lib_path)?;
            }

            return _intruduce(proc, new_lib_path, lib_dir);
        }
    }

    _intruduce(proc, lib_path, PathBuf::from(TMP_DIR))
}

fn _intruduce(proc: Proc, lib_path: PathBuf, second_payload_path: PathBuf) -> Result<(), Error> {
    let lib_path = lib_path.canonicalize().unwrap_or(lib_path);
    let lib_path = lib_path.to_str().unwrap();
    let second_payload_path = second_payload_path.join("payload.bin");
    let second_payload_path = second_payload_path.to_str().unwrap();

    let dlopen = proc.find_dlopen()?;

    #[cfg(debug_assertions)]
    println!("dlopen address: 0x{:x}", dlopen.addr);

    let class = proc.class().ok_or(Error::UnsupportedArch)?;

    let first_payload = first_payload(&class, second_payload_path);

    let mem = proc.mem()?;

    let mut original_code = vec![0; first_payload.len()];

    let ip = proc.find_ip()?;

    #[cfg(debug_assertions)]
    println!("instruction pointer: 0x{:x}", ip);

    mem.read_exact_at(&mut original_code, ip)?;
    mem.write_all_at(&first_payload, ip)?;

    let second_payload = second_payload(&class, &original_code, ip, lib_path, &dlopen);

    let mut file = File::create(&second_payload_path)?;

    let (uid, gid) = proc.owner()?;

    change_owner(second_payload_path, uid, gid).ok_or(Error::InsufficientPriviliges)?;

    file.write_all(&second_payload)?;

    Ok(())
}

// TODO: wait for https://github.com/rust-lang/rust/issues/88989
fn change_owner(path: &str, uid: Uid, gid: Gid) -> Option<()> {
    use std::{
        ffi::CString,
        os::raw::{c_char, c_int, c_uint},
    };

    #[link(name = "c")]
    extern "C" {
        fn chown(path: *const c_char, uid: c_uint, gid: c_uint) -> c_int;
    }

    let path = CString::new(path).ok()?;

    if unsafe { chown(path.as_ptr(), uid, gid) } == 0 {
        Some(())
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
const TMP_DIR: &str = "/tmp";

#[cfg(target_os = "android")]
const TMP_DIR: &str = "/data/local/tmp";

/// The errors may occurr.
#[derive(Debug)]
pub enum Error {
    /// It occurs when the `dlopen` library (`libc-x.xx.so` on Linux, `libdl.so` on Android)
    /// is not found in the `/proc/<id>/maps` file of the target process. This either means that library has not been loaded - which
    /// is kind of impossible - or `/proc/<id>/maps` was improperly parsed.
    LibraryNotFound(String),
    /// It occurs when `dlopen` symbol name (`__libc_dlopen_mode` on Linux, `dlopen` on Android)
    /// was not found in the expected library.
    SymbolNotFound(String),
    /// It occurs when the instruction pointer of the target process couldn't be retrieved. This either means there's a lack of priviliges,
    /// `/proc/<id>/syscall` is missing or was improperly parsed, or none of the process thread was blocked when the intruduction
    /// was attempted.
    InstructionPointerNotFound,
    /// It occurs when the target process architecture is not supported.
    UnsupportedArch,
    /// It occurs when the target process is not running - e.g. `/proc/<id>` doesn't exist.
    ProcessNotRunning,
    /// It occurs when the intruducer process lacks of sufficient priviliges. This typically depends on `/proc/sys/kernel/yama/ptrace_scope`
    /// value on Linux.
    InsufficientPriviliges,
    #[cfg(target_os = "android")]
    LibraryPathNeeded,
    /// It occurs when a I/O error occurred.
    Io(IoError),
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Error::Io(err)
    }
}

/// A type alias to represent virtual addresses.
pub(crate) type VirtAddr = u64;

/// A type alias to represent a process identifier.
pub(crate) type ProcId = u32;

/// A type alias to represent a user identifier.
pub(crate) type Uid = u32;

/// A type alias to represent a group identifier.
pub(crate) type Gid = u32;

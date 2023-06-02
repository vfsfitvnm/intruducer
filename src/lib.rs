//! A Rust crate to load a shared library into a target process without using `ptrace`.
//! This is a portable rewrite of [dlinject](https://github.com/DavidBuchanan314/dlinject).

use std::fs::File;
use std::io::Write;
use std::ops::Not;
use std::os::unix::prelude::FileExt;
use std::path::PathBuf;

mod constants;
mod error;
mod ext;
mod os;
mod payloads;
mod proc;

use constants::TMP_DIR;
pub use error::Error;
use proc::ProcId;

use ext::ProcExt;
use ext::ProcIntruducerExt;
use os::{chown, PtraceScope};
use proc::Proc;

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
    .then_some(())
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

    let first_payload = payloads::gen_first(&class, second_payload_path);

    let mem = proc.mem()?;

    let mut original_code = vec![0; first_payload.len()];

    let ip = proc.find_ip()?;

    #[cfg(debug_assertions)]
    println!("instruction pointer: 0x{:x}", ip);

    mem.read_exact_at(&mut original_code, ip)?;
    mem.write_all_at(&first_payload, ip)?;

    let second_payload = payloads::gen_second(&class, &original_code, ip, lib_path, &dlopen);

    let mut file = File::create(second_payload_path)?;

    let (uid, gid) = proc.owner()?;

    chown(second_payload_path, uid, gid).ok_or(Error::InsufficientPriviliges)?;

    file.write_all(&second_payload)?;

    Ok(())
}

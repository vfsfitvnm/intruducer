mod gid;
mod ptrace_scope;
mod uid;
mod virt_addr;

pub(crate) use gid::Gid;
pub(crate) use ptrace_scope::PtraceScope;
pub(crate) use uid::Uid;
pub(crate) use virt_addr::VirtAddr;

// TODO: wait for https://github.com/rust-lang/rust/issues/88989
pub(crate) fn chown(path: &str, uid: Uid, gid: Gid) -> Option<()> {
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

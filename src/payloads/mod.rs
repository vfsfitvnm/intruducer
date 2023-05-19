use crate::{
    proc::{ProcClass, ProcSym},
    VirtAddr,
};

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
mod arm;
#[cfg(target_arch = "aarch64")]
mod arm64;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;
#[cfg(target_arch = "x86_64")]
mod x86_64;

pub(crate) fn gen_first(class: &ProcClass, second_payload_path: &str) -> Vec<u8> {
    match class {
        #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
        ProcClass::ThirtyTwo => arm::gen_first(second_payload_path),
        #[cfg(target_arch = "aarch64")]
        ProcClass::SixtyFour => arm64::gen_first(second_payload_path),
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        ProcClass::ThirtyTwo => x86::gen_first(second_payload_path),
        #[cfg(target_arch = "x86_64")]
        ProcClass::SixtyFour => x86_64::gen_first(second_payload_path),
    }
}

pub(crate) fn gen_second(
    class: &ProcClass,
    original_code: &[u8],
    original_ip: VirtAddr,
    lib_path: &str,
    dlopen: &ProcSym,
) -> Vec<u8> {
    match class {
        #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
        ProcClass::ThirtyTwo => arm::gen_second(original_code, original_ip, lib_path, dlopen),
        #[cfg(target_arch = "aarch64")]
        ProcClass::SixtyFour => arm64::gen_second(original_code, original_ip, lib_path, dlopen),
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        ProcClass::ThirtyTwo => x86::gen_second(original_code, original_ip, lib_path, dlopen),
        #[cfg(target_arch = "x86_64")]
        ProcClass::SixtyFour => x86_64::gen_second(original_code, original_ip, lib_path, dlopen),
    }
}

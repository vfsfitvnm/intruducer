#[cfg(target_os = "linux")]
pub(crate) const DLOPEN_SYM_NAMES: [&str; 2] = ["__libc_dlopen_mode", "dlopen"];

#[cfg(target_os = "android")]
pub(crate) const DLOPEN_SYM_NAMES: [&str; 1] = ["dlopen"];

#[cfg(target_os = "linux")]
pub(crate) const TMP_DIR: &str = "/tmp";

#[cfg(target_os = "android")]
pub(crate) const TMP_DIR: &str = "/data/local/tmp";

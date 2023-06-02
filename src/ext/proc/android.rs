/// A extension trait for [`Proc`] specific to the Android operative system.
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

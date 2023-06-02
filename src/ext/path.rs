use std::path::PathBuf;

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

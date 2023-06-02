/// A enum that represents the class of a process (32 bit or 64 bit).
pub(crate) enum ProcClass {
    /// The values used to describe 32 bit processes.
    #[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
    ThirtyTwo,
    /// The values used to describe 64 bit processes.
    #[cfg(target_pointer_width = "64")]
    SixtyFour,
}

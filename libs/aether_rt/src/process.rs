/// This is the startup information passed to each native process on startup,
/// that means the initial data like args, env vars, and stack allocation info.
/// By native process, we mean a process that is **compatible**
/// with the Aether userspace ABI.
#[repr(C)]
#[derive(Debug)]
pub struct StartupInfo {
    /// Magic number to detect memory corruption or ABI mismatch (e.g., "AETH")
    pub magic: u32,
    /// Version of this struct, allows backwards compatibility later
    pub version: u32,

    /// The bottom of the stack (lowest address, where the guard page is)
    pub stack_base: *mut u8,
}

impl StartupInfo {
    pub const MAGIC: u32 = u32::from_le_bytes(*b"AETH");
    pub const VERSION: u32 = 1;

    /// Attempt to create a `StartupInfo` reference from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure the pointer is valid and points to a properly
    /// aligned `StartupInfo` struct.
    pub unsafe fn from_ptr(ptr: *const u8) -> Option<&'static StartupInfo> {
        let magic = unsafe { ptr.cast::<u32>().read() };
        if magic == Self::MAGIC {
            unsafe { ptr.cast::<StartupInfo>().as_ref() }
        } else {
            None
        }
    }
}

use core::{ffi::CStr, ptr::NonNull};

pub trait BootInfo {
    fn get_modules(&self) -> impl Iterator<Item = BootModule<'static>>;

    fn get_framebuffer(&self) -> Option<BootFramebuffer>;
}

pub struct BootModule<'a> {
    pub name: &'a CStr,
    pub data: &'a [u8],
}

pub struct BootFramebuffer {
    pub address: NonNull<u8>,
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub bpp: usize,
}

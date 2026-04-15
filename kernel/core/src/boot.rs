use core::ffi::CStr;

pub trait BootInfo {
    fn get_modules(&self) -> impl Iterator<Item = BootModule<'static>>;
}

pub struct BootModule<'a> {
    pub name: &'a CStr,
    pub data: &'a [u8],
}

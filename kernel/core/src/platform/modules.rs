use core::ffi::CStr;

mod bindings {
    include_bindings!("modules.rs");
}

pub struct Modules;

impl Modules {
    pub unsafe fn find(string: &CStr) -> Option<&'static [u8]> {
        unsafe {
            let file = bindings::module_find(string.as_ptr());
            if file.is_null() {
                return None;
            }
            let addr = (*file).address as *const u8;
            let len = (*file).size as usize;
            Some(core::slice::from_raw_parts(addr, len))
        }
    }
}

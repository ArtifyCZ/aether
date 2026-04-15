use crate::boot::BootModule;
use crate::println;
use alloc::vec::Vec;
use core::ffi::{CStr, c_char};
use core::ptr::{NonNull, null_mut};
use kernel_bindings_gen::{limine_file, limine_module_response};

pub struct Modules;

static mut MODULES: Option<Vec<BootModule>> = None;

impl Modules {
    pub unsafe fn init(modules: impl Iterator<Item = BootModule>) {
        unsafe {
            let modules = modules.collect();
            MODULES = Some(modules);
        }
    }

    #[allow(static_mut_refs)]
    pub unsafe fn find(string: &CStr) -> Option<&'static [u8]> {
        unsafe {
            let file = MODULES
                .as_ref()?
                .iter()
                .find(|module| module.name == string)?;
            Some(file.data)
        }
    }
}

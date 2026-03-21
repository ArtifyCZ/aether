use crate::println;
use core::ffi::{CStr, c_char};
use core::ptr::{NonNull, null_mut};
use kernel_bindings_gen::{limine_file, limine_module_response};

pub struct Modules;

static mut MODULES: Option<NonNull<limine_module_response>> = None;

unsafe fn modules_init(modules: *mut limine_module_response) {
    let modules = NonNull::new(modules).expect("Modules have not been provided!");
    println!("Initializing modules...");
    unsafe {
        let modules = modules.as_ref();
        let count = modules.module_count as usize;
        for i in 0..count {
            let file: *mut limine_file = modules.modules.add(i).read();
            let name = CStr::from_ptr((*file).string);
            let path = CStr::from_ptr((*file).path);
            println!("Module [{:?}]: {:?}", name, path);
        }
    }
    unsafe {
        MODULES = Some(modules);
    }
}

unsafe  fn module_find(string: *const c_char) -> *const limine_file {
    unsafe {
        let modules = MODULES
            .expect("Cannot find module when modules are uninitialized!")
            .as_ref();
        let count = modules.module_count as usize;

        let string = CStr::from_ptr(string);

        for i in 0..count {
            let file: *mut limine_file = modules.modules.add(i).read();
            let name = CStr::from_ptr((*file).string);
            if name == string {
                return file;
            }
        }
    }

    null_mut()
}

impl Modules {
    pub unsafe fn init(modules: *mut limine_module_response) {
        unsafe {
            modules_init(modules);
        }
    }

    pub unsafe fn find(string: &CStr) -> Option<&'static [u8]> {
        unsafe {
            let file = module_find(string.as_ptr());
            if file.is_null() {
                return None;
            }
            let addr = (*file).address as *const u8;
            let len = (*file).size as usize;
            Some(core::slice::from_raw_parts(addr, len))
        }
    }
}

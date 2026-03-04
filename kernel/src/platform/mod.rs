macro_rules! include_bindings {
    ($file:expr) => {
        #[allow(clashing_extern_declarations)]
        #[allow(non_camel_case_types)]
        #[allow(non_snake_case)]
        #[allow(non_upper_case_globals)]
        #[allow(unused)]
        mod inner {
            include!(concat!(env!("OUT_DIR"), "/bindings/", $file));
        }
        pub use inner::*;
    };
}

pub mod early_console;
pub mod elf;
pub mod emergency_console;
pub mod interrupts;
pub mod memory_layout;
pub mod modules;
pub mod physical_memory_manager;
pub mod physical_page_frame;
pub mod platform;
pub mod syscalls;
pub mod tasks;
pub mod terminal;
pub mod timer;
pub mod virtual_address;
pub mod virtual_address_allocator;
pub mod virtual_memory_manager_context;
pub mod virtual_page_address;

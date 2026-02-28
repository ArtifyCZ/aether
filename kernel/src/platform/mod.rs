macro_rules! include_bindings {
    ($module:tt) => {
        include!(concat!(env!("OUT_DIR"), "/bindings/", $module));
    };
}

pub mod drivers;
pub mod elf;
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

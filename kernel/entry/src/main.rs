#![no_std]
#![no_main]

use log::info;

mod early_console;
mod logger;
mod panic;
mod requests;
mod spinlock;
mod start;

unsafe fn main() -> ! {
    let framebuffer = requests::framebuffer().expect("No framebuffer available");
    unsafe {
        early_console::init(framebuffer);
    }
    logger::init();
    atom_core::bsp_main();
}

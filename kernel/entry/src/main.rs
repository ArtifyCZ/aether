#![no_std]
#![no_main]

mod early_console;
mod panic;
mod requests;
mod start;

unsafe fn main() -> ! {
    let framebuffer = requests::framebuffer().expect("No framebuffer available");
    let early_console = unsafe { early_console::init(framebuffer) };
    atom_core::bsp_main(early_console);
}

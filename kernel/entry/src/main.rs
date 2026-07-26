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
        logger::init();
        info!("Hello World using logger lol!");
        let chars = ['a', 'b', 'c', 'd', 'e', 'f'];
        for i in 0..150 {
            let c = chars[i % chars.len()];
            info!("{} - {}", i, c);
            for _ in 0..1000000 {
                core::hint::spin_loop();
            }
        }
    }
    loop {}
}

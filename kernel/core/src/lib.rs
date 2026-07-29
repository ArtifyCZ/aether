#![no_std]

pub mod logger;

use crate::logger::EarlyConsole;
use log::info;

pub fn bsp_main(early_console: &'static mut dyn EarlyConsole) -> ! {
    logger::init(early_console);
    info!("Booting Atom kernel...");
    info!("Bootstrap core starting...");
    let chars = ['a', 'b', 'c', 'd', 'e', 'f'];
    for i in 0..150 {
        let c = chars[i % chars.len()];
        info!("{} - {}", i, c);
        for _ in 0..1000000 {
            core::hint::spin_loop();
        }
    }
    loop {}
}

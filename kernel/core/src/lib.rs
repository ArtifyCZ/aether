#![no_std]

pub mod logger;

use atom_hal::framebuffer_console::Framebuffer;
use atom_hal::serial_console::SerialConsoleConfig;
use log::{info, warn};

pub fn bsp_main(
    framebuffer: Option<Framebuffer>,
    serial_console_config: Option<SerialConsoleConfig>,
) -> ! {
    logger::init(framebuffer, serial_console_config);
    info!("Booting Atom kernel...");
    info!("Bootstrap core starting...");
    warn!("Hello warning world!");
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

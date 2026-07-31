#![no_std]

mod logger;
pub mod memory_regions;

use crate::memory_regions::MemoryRegion;
use atom_hal::framebuffer_console::Framebuffer;
use atom_hal::serial_console::SerialConsoleConfig;
use log::{info, warn};

pub fn bsp_main(
    framebuffer: Option<Framebuffer>,
    serial_console_config: Option<SerialConsoleConfig>,
    memory_regions: impl Iterator<Item = MemoryRegion>,
) -> ! {
    logger::init(framebuffer, serial_console_config);
    info!("Booting Atom kernel...");
    info!("Bootstrap core starting...");
    warn!("Hello warning world!");
    for memory_region in memory_regions {
        info!("Memory region: {:x?}", memory_region);
    }
    warn!("Continuing...");
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

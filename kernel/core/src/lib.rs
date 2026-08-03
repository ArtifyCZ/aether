#![no_std]

mod logger;
pub mod memory_regions;

use crate::memory_regions::{IntoPagedMemoryRegionIterator, MemoryRegion, MemoryRegionKind};
use atom_hal::framebuffer_console::Framebuffer;
use atom_hal::serial_console::SerialConsoleConfig;
use log::{info, warn};

pub fn bsp_main(
    framebuffer: Option<Framebuffer>,
    serial_console_config: Option<SerialConsoleConfig>,
    memory_regions: impl Iterator<Item = MemoryRegion> + Clone,
) -> ! {
    logger::init(framebuffer, serial_console_config);
    info!("Booting Atom kernel...");
    info!("Bootstrap core starting...");
    warn!("Hello warning world!");
    for memory_region in memory_regions.clone() {
        info!("Memory region: {:x?}", memory_region);
    }
    let pages_count = memory_regions
        .filter(|region| region.kind == MemoryRegionKind::Usable)
        .into_paged_iter()
        .count();
    warn!("Pages: {}", pages_count);
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

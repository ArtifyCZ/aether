#![no_std]

mod logger;
pub mod memory_regions;
mod physical_memory_allocator;

use crate::memory_regions::{
    IntoPagedMemoryRegionIterator, MemoryRegion, MemoryRegionKind, MemoryRegionPage,
};
use atom_hal::framebuffer_console::Framebuffer;
use atom_hal::serial_console::SerialConsoleConfig;
use atom_primitives::HhdmOffset;
use log::{info, warn};

pub fn bsp_main(
    framebuffer: Option<Framebuffer>,
    hhdm_offset: HhdmOffset,
    serial_console_config: Option<SerialConsoleConfig>,
    memory_regions: impl Iterator<Item = MemoryRegion>,
    executable_address: (usize, usize),
) -> ! {
    logger::init(framebuffer, serial_console_config);
    info!("Booting Atom kernel...");
    info!("Bootstrap core starting...");
    info!("Initializing physical memory allocator...");
    physical_memory_allocator::init(hhdm_offset);
    info!(
        "Executable address: {:p}; {:p}",
        executable_address.0 as *const (), executable_address.1 as *const (),
    );
    warn!("Hello warning world!");
    let paged_memory_regions = memory_regions
        .filter(|region| region.kind == MemoryRegionKind::Usable)
        .into_paged_iter();
    warn!("Freeing pages...");
    let mut huge_pages: u32 = 0;
    let mut large_pages: u32 = 0;
    let mut normal_pages: u32 = 0;
    for page in paged_memory_regions {
        match page {
            MemoryRegionPage::Huge(page) => {
                physical_memory_allocator::free_huge_page(page);
                huge_pages += 1;
            }
            MemoryRegionPage::Large(page) => {
                physical_memory_allocator::free_large_page(page);
                large_pages += 1;
            }
            MemoryRegionPage::Normal(page) => {
                physical_memory_allocator::free_normal_page(page);
                normal_pages += 1;
            }
        }
    }
    info!(
        "Inserted huge pages (1 GiB) into physical memory allocator: {}",
        huge_pages,
    );
    info!(
        "Inserted large pages (2 MiB) into physical memory allocator: {}",
        large_pages,
    );
    info!(
        "Inserted normal pages (4 kiB) into physical memory allocator: {}",
        normal_pages,
    );
    let page = physical_memory_allocator::alloc_normal_page();
    info!(
        "Allocated normal pages into physical memory allocator: {:?}",
        page,
    );
    unsafe {
        let ptr = ((page.unwrap().base_address + *hhdm_offset) as *mut u64);
        ptr.write(42);
        info!("Written to the allocated page {:#?}", ptr);
        info!("Reading from the allocated page: {}", ptr.read());
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

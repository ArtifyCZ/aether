#![no_std]
#![no_main]

use atom_core::memory_regions::{MemoryRegion, MemoryRegionKind};
use atom_hal::framebuffer_console::Framebuffer;
use atom_hal::serial_console::{SerialConsoleConfig, SerialPortConfig};
use atom_hal::{framebuffer_console, serial_console};
use limine::memmap::{
    MEMMAP_ACPI_NVS, MEMMAP_ACPI_RECLAIMABLE, MEMMAP_BAD_MEMORY, MEMMAP_BOOTLOADER_RECLAIMABLE,
    MEMMAP_EXECUTABLE_AND_MODULES, MEMMAP_FRAMEBUFFER, MEMMAP_MAPPED_RESERVED, MEMMAP_RESERVED,
    MEMMAP_USABLE,
};

mod panic;
mod requests;
mod start;

unsafe fn main() -> ! {
    let framebuffer = requests::framebuffer().map(|framebuffer| Framebuffer {
        address: framebuffer.address(),
        width: framebuffer.width as usize,
        height: framebuffer.height as usize,
        bpp: framebuffer.bpp,
        red_mask_size: framebuffer.red_mask_size,
        red_mask_shift: framebuffer.red_mask_shift,
        green_mask_size: framebuffer.green_mask_size,
        green_mask_shift: framebuffer.green_mask_shift,
        blue_mask_size: framebuffer.blue_mask_size,
        blue_mask_shift: framebuffer.blue_mask_shift,
        unsafe_token: unsafe { framebuffer_console::create_framebuffer_unsafe_token() },
    });
    #[cfg(target_arch = "aarch64")]
    let serial_port_config = SerialPortConfig {
        uart_phys_base: 0x0900_0000,
        hhdm_offset: requests::hhdm_offset(),
    };
    #[cfg(target_arch = "x86_64")]
    let serial_port_config = SerialPortConfig { port: 0x3f8 };
    let serial_console_config = Some(SerialConsoleConfig {
        port_config: serial_port_config,
        unsafe_token: unsafe { serial_console::create_serial_console_unsafe_token() },
    });
    let memory_regions = requests::memory_regions()
        .iter()
        .map(|region| MemoryRegion {
            base_address: region.base as usize,
            length: region.length as usize,
            kind: match region.type_ {
                MEMMAP_USABLE => MemoryRegionKind::Usable,
                MEMMAP_RESERVED => MemoryRegionKind::Reserved,
                MEMMAP_ACPI_RECLAIMABLE => MemoryRegionKind::Reclaimable,
                MEMMAP_ACPI_NVS => MemoryRegionKind::Reserved,
                MEMMAP_BAD_MEMORY => MemoryRegionKind::BadMemory,
                MEMMAP_BOOTLOADER_RECLAIMABLE => MemoryRegionKind::Reclaimable,
                MEMMAP_EXECUTABLE_AND_MODULES => MemoryRegionKind::ExecutableAndModules,
                MEMMAP_FRAMEBUFFER => MemoryRegionKind::Reserved,
                MEMMAP_MAPPED_RESERVED => MemoryRegionKind::Reserved,
                _ => panic!("Unsupported memory region type"),
            },
        });
    atom_core::bsp_main(framebuffer, serial_console_config, memory_regions);
}

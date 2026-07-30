#![no_std]
#![no_main]

use atom_hal::serial_console::SerialPortConfig;

mod early_console;
mod panic;
mod requests;
mod start;

unsafe fn main() -> ! {
    let framebuffer = requests::framebuffer().expect("No framebuffer available");
    let early_console = unsafe {
        early_console::init(
            framebuffer,
            #[cfg(target_arch = "aarch64")]
            SerialPortConfig {
                uart_phys_base: 0x0900_0000,
                hhdm_offset: requests::hhdm_offset(),
            },
            #[cfg(target_arch = "x86_64")]
            SerialPortConfig { port: 0x3f8 },
        )
    };
    atom_core::bsp_main(early_console);
}

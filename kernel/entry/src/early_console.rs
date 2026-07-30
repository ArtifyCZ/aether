use crate::early_console::framebuffer_console::{Framebuffer, FramebufferDisplay};
use atom_core::logger::EarlyConsole;
use atom_hal::serial_console::SerialConsole;
use atom_hal::serial_console::SerialPortConfig;
use atom_hal::{framebuffer_console, serial_console};
use core::cell::UnsafeCell;
use core::fmt::Write;
use core::mem::MaybeUninit;

struct InstanceWrapper(UnsafeCell<MaybeUninit<EarlyConsoleImpl>>);
unsafe impl Sync for InstanceWrapper {}
static INSTANCE: InstanceWrapper = InstanceWrapper(UnsafeCell::new(MaybeUninit::uninit()));

/// Initializes early console
///
/// Returns a mutable reference to the early console instance.
///
/// # Safety
///
/// This function must never be called more than once.
#[expect(clippy::mut_from_ref)]
pub unsafe fn init(
    framebuffer: &'static limine::framebuffer::Framebuffer,
    serial_config: SerialPortConfig,
) -> &'static mut dyn EarlyConsole {
    let framebuffer = unsafe {
        framebuffer_console::init(Framebuffer {
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
            unsafe_token: framebuffer_console::create_framebuffer_unsafe_token(),
        })
    };
    let serial = unsafe { serial_console::init(serial_config) };
    let console = EarlyConsoleImpl {
        framebuffer,
        serial,
    };
    unsafe {
        *INSTANCE.0.get() = MaybeUninit::new(console);
        (*INSTANCE.0.get()).assume_init_mut()
    }
}

struct EarlyConsoleImpl {
    framebuffer: FramebufferDisplay,
    serial: SerialConsole,
}

impl EarlyConsole for EarlyConsoleImpl {
    fn close(&mut self) {
        todo!()
    }
}

impl Write for EarlyConsoleImpl {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.framebuffer.write_str(s)?;
        self.serial.write_str(s)?;
        Ok(())
    }
}

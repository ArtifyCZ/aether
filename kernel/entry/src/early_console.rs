mod framebuffer;

use crate::early_console::framebuffer::FramebufferDisplay;
use atom_core::logger::EarlyConsole;
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
#[allow(clippy::mut_from_ref)]
pub unsafe fn init(
    framebuffer: &'static limine::framebuffer::Framebuffer,
) -> &'static mut dyn EarlyConsole {
    let framebuffer = unsafe { framebuffer::init(framebuffer) };
    let console = EarlyConsoleImpl { framebuffer };
    unsafe {
        *INSTANCE.0.get() = MaybeUninit::new(console);
        (*INSTANCE.0.get()).assume_init_mut()
    }
}

struct EarlyConsoleImpl {
    framebuffer: FramebufferDisplay,
}

impl EarlyConsole for EarlyConsoleImpl {
    fn close(&mut self) {
        todo!()
    }
}

impl Write for EarlyConsoleImpl {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.framebuffer.write_str(s)
    }
}

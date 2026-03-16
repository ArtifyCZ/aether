use crate::interrupt_safe_spin_lock::InterruptSafeSpinLock;
use crate::platform::modules::Modules;
use core::ffi::{CStr, c_char};
use core::ptr::NonNull;
use eclipse_framebuffer::ScrollingTextRenderer;
use kernel_bindings_gen::limine_framebuffer;
use crate::spin_lock::SpinLock;

static TERMINAL: SpinLock<Option<Terminal>> = SpinLock::new(None);

pub struct Terminal {
    renderer: &'static mut ScrollingTextRenderer,
}

#[unsafe(no_mangle)]
unsafe extern "C" fn terminal_print_char(c: c_char) {
    let mut terminal = TERMINAL.lock();
    terminal
        .as_mut()
        .unwrap()
        .renderer
        .write_char((c as u8) as char);
}

#[unsafe(no_mangle)]
unsafe extern "C" fn terminal_print(message: *const c_char) {
    unsafe {
        let message = CStr::from_ptr(message);
        let message = message.to_string_lossy();
        let mut terminal = TERMINAL.lock();
        terminal
            .as_mut()
            .unwrap()
            .renderer
            .write_str(message.as_ref());
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn terminal_println(message: *const c_char) {
    unsafe {
        let message = CStr::from_ptr(message);
        let message = message.to_string_lossy();
        let mut terminal = TERMINAL.lock();
        terminal
            .as_mut()
            .unwrap()
            .renderer
            .write_str(message.as_ref());
        let mut terminal = TERMINAL.lock();
        terminal.as_mut().unwrap().renderer.write_char('\n');
    }
}

// @TODO: remove the dependency on the `eclipse_framebuffer` crate

impl Terminal {
    pub unsafe fn init(framebuffer: NonNull<limine_framebuffer>) {
        let framebuffer = unsafe { framebuffer.as_ref() };
        let font = unsafe { Modules::find(c"kernel-font.psf") }.unwrap();
        ScrollingTextRenderer::init(
            framebuffer.address.cast(),
            framebuffer.width as usize,
            framebuffer.height as usize,
            framebuffer.pitch as usize,
            framebuffer.bpp as usize,
            font,
        );
        let renderer = ScrollingTextRenderer::get();
        renderer.set_colors(0xD4DBDF, 0x04121B);
        renderer.clear();

        unsafe {
            let mut terminal = TERMINAL.lock();
            *terminal = Some(Terminal { renderer });
            crate::logging::enable_terminal();
        }
    }

    pub unsafe fn print_char(c: char) {
        let mut terminal = TERMINAL.lock();
        terminal.as_mut().unwrap().renderer.write_char(c);
    }

    pub unsafe fn print_bytes(bytes: &[u8]) {
        let bytes = str::from_utf8(bytes).unwrap();
        let mut terminal = TERMINAL.lock();
        terminal.as_mut().unwrap().renderer.write_str(bytes);
    }

    pub unsafe fn print(message: &str) {
        let mut terminal = TERMINAL.lock();
        terminal.as_mut().unwrap().renderer.write_str(message);
    }
}

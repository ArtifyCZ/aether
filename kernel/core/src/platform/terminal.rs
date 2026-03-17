use crate::interrupt_safe_spin_lock::InterruptSafeSpinLock;
use crate::platform::modules::Modules;
use crate::println;
use core::ptr::NonNull;
use eclipse_framebuffer::ScrollingTextRenderer;
use kernel_bindings_gen::limine_framebuffer;

static TERMINAL: InterruptSafeSpinLock<Option<Terminal>> = InterruptSafeSpinLock::new(None);

pub struct Terminal {
    renderer: &'static mut ScrollingTextRenderer,
}

// @TODO: remove the dependency on the `eclipse_framebuffer` crate

impl Terminal {
    pub unsafe fn init(framebuffer: NonNull<limine_framebuffer>) {
        println!("Initializing terminal...");
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
        println!("Hoo");

        unsafe {
            let mut terminal = TERMINAL.lock();
            *terminal = Some(Terminal { renderer });
            println!("Hoo2");
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

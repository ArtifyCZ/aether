use atom_core::logger::EarlyConsole;
use core::cell::UnsafeCell;
use core::convert::Infallible;
use core::fmt::Write;
use core::mem::MaybeUninit;
use embedded_graphics::geometry::{OriginDimensions, Size};
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::prelude::{DrawTarget, Point};
use embedded_graphics::text::Text;
use embedded_graphics::{Drawable, Pixel};

struct InstanceWrapper(UnsafeCell<MaybeUninit<EarlyConsoleDisplay>>);
unsafe impl Sync for InstanceWrapper {}
static DISPLAY_INSTANCE: InstanceWrapper = InstanceWrapper(UnsafeCell::new(MaybeUninit::uninit()));

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
    let cursor = Point::new(0, 0);
    let fb_char_width = (framebuffer.width / FONT.character_size.width as u64) as u32;
    let fb_char_height = (framebuffer.height / FONT.character_size.height as u64) as u32;
    let framebuffer_char_size = Size::new(fb_char_width, fb_char_height);
    let mut display = EarlyConsoleDisplay {
        framebuffer,
        cursor,
        framebuffer_char_size,
    };
    writeln!(&mut display).unwrap();
    unsafe {
        *DISPLAY_INSTANCE.0.get() = MaybeUninit::new(display);
        (*DISPLAY_INSTANCE.0.get()).assume_init_mut()
    }
}

const FONT: &MonoFont = &FONT_10X20;

impl EarlyConsole for EarlyConsoleDisplay {
    fn close(&mut self) {
        todo!()
    }
}

impl Write for EarlyConsoleDisplay {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            write_char(self, c);
        }
        Ok(())
    }
}

fn write_char(display: &mut EarlyConsoleDisplay, c: char) {
    let cursor = display.cursor;
    let (new_cursor, should_print) = match c {
        '\n' => (Point::new(0, cursor.y + 1), false),
        _ => {
            if (cursor.x + 1) < display.framebuffer_char_size.width as i32 {
                (Point::new(cursor.x + 1, cursor.y), true)
            } else {
                (Point::new(0, cursor.y + 1), true)
            }
        }
    };
    display.cursor = new_cursor;
    if cursor.y + 1 >= display.framebuffer_char_size.height as i32 && cursor.y < new_cursor.y {
        scroll(display);
    }

    if !should_print {
        return;
    }

    let pixel_coord = Point::new(
        cursor.x * FONT.character_size.width as i32,
        (cursor
            .y
            .min(display.framebuffer_char_size.height as i32 - 1))
            * FONT.character_size.height as i32,
    );

    let mut char_str = [0u8; 4];
    let char_str = c.encode_utf8(&mut char_str);
    let _ = Text::new(
        char_str,
        pixel_coord,
        MonoTextStyle::new(FONT, Rgb888::CYAN),
    )
    .draw(display);
}

fn scroll(display: &mut EarlyConsoleDisplay) {
    let base_ptr = display.framebuffer.address();
    let fb_width = display.framebuffer.width as usize;
    let char_height = FONT.character_size.height as usize;
    let height = display.framebuffer_char_size.height as usize;
    let start_idx = fb_width * char_height;
    let end_idx = char_height * height * fb_width;
    for base_idx in 0..(end_idx - start_idx) {
        let orig_idx = start_idx + base_idx;
        match display.framebuffer.bpp {
            8 => unsafe {
                let base_ptr = base_ptr as *mut u8;
                let color: u8 = base_ptr.add(orig_idx).read();
                base_ptr.add(base_idx).write(color);
            },
            15 | 16 => unsafe {
                let base_ptr = base_ptr as *mut u16;
                let color: u16 = base_ptr.add(orig_idx).read();
                base_ptr.add(base_idx).write(color);
            },
            24 => unsafe {
                let base_ptr = base_ptr as *mut u8;
                let orig_ptr = base_ptr.add(orig_idx * 3);
                let pixel_ptr = base_ptr.add(base_idx * 3);
                pixel_ptr.byte_add(0).write(orig_ptr.byte_add(0).read());
                pixel_ptr.byte_add(1).write(orig_ptr.byte_add(1).read());
                pixel_ptr.byte_add(2).write(orig_ptr.byte_add(2).read());
            },
            32 => unsafe {
                let base_ptr = base_ptr as *mut u32;
                let color: u32 = base_ptr.add(orig_idx).read();
                base_ptr.add(base_idx).write(color);
            },
            _ => todo!(),
        };
    }
}

struct EarlyConsoleDisplay {
    framebuffer: &'static limine::framebuffer::Framebuffer,
    cursor: Point,
    framebuffer_char_size: Size,
}

unsafe impl Send for EarlyConsoleDisplay {}

impl OriginDimensions for EarlyConsoleDisplay {
    fn size(&self) -> Size {
        Size::new(
            self.framebuffer.width as u32,
            self.framebuffer.height as u32,
        )
    }
}

impl DrawTarget for EarlyConsoleDisplay {
    type Color = Rgb888;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let base_ptr = self.framebuffer.address();
        let width = self.framebuffer.width;
        let height = self.framebuffer.height;
        let red_mask_size = self.framebuffer.red_mask_size;
        let red_mask_shift = self.framebuffer.red_mask_shift;
        let green_mask_size = self.framebuffer.green_mask_size;
        let green_mask_shift = self.framebuffer.green_mask_shift;
        let blue_mask_size = self.framebuffer.blue_mask_size;
        let blue_mask_shift = self.framebuffer.blue_mask_shift;
        for Pixel(coord, color) in pixels {
            if coord.x >= 0 && coord.x < width as i32 && coord.y >= 0 && coord.y < height as i32 {
                let idx = coord.x as usize + (coord.y as u64 * width) as usize;
                let direct_color: u32 = 0
                    | ((color.r() as u32 >> (8 - red_mask_size)) << red_mask_shift)
                    | ((color.g() as u32 >> (8 - green_mask_size)) << green_mask_shift)
                    | ((color.b() as u32 >> (8 - blue_mask_size)) << blue_mask_shift);
                match self.framebuffer.bpp {
                    8 => unsafe { (base_ptr as *mut u8).add(idx).write(direct_color as u8) },
                    15 | 16 => unsafe {
                        (base_ptr as *mut u16).add(idx).write(direct_color as u16)
                    },
                    24 => unsafe {
                        let pixel_ptr = (base_ptr as *mut u8).add(idx * 3);
                        pixel_ptr.byte_add(0).write((direct_color >> 16) as u8);
                        pixel_ptr.byte_add(1).write((direct_color >> 8) as u8);
                        pixel_ptr.byte_add(2).write((direct_color >> 0) as u8);
                    },
                    32 => unsafe { (base_ptr as *mut u32).add(idx).write(direct_color) },
                    _ => todo!(),
                };
            }
        }
        Ok(())
    }
}

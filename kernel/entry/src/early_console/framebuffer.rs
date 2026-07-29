use core::convert::Infallible;
use core::fmt::Write;
use embedded_graphics::geometry::{OriginDimensions, Size};
use embedded_graphics::mono_font::iso_8859_16::{FONT_9X18, FONT_9X18_BOLD};
use embedded_graphics::mono_font::{MonoFont, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::prelude::{DrawTarget, Point, Primitive, WebColors};
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::Text;
use embedded_graphics::{Drawable, Pixel};

/// Initializes early console framebuffer backend
///
/// # Safety
///
/// This function must never be called more than once per framebuffer.
pub unsafe fn init(framebuffer: &'static limine::framebuffer::Framebuffer) -> FramebufferDisplay {
    let cursor = Point::new(0, 0);
    let fb_char_width = (framebuffer.width / FONT.character_size.width as u64) as u32;
    let fb_char_height = (framebuffer.height / FONT.character_size.height as u64) as u32;
    let framebuffer_char_size = Size::new(fb_char_width, fb_char_height);
    let mut display = FramebufferDisplay {
        framebuffer,
        cursor,
        framebuffer_char_size,
        foreground_color: DEFAULT_FOREGROUND_COLOR,
        foreground_style: DEFAULT_FOREGROUND_STYLE,
        foreground_high_intensity: DEFAULT_FOREGROUND_HIGH_INTENSITY,
        background_color: DEFAULT_BACKGROUND_COLOR,
        background_high_intensity: DEFAULT_BACKGROUND_HIGH_INTENSITY,
    };
    writeln!(&mut display).unwrap();
    display
}

const FONT: &MonoFont = &FONT_9X18;
const FONT_BOLD: &MonoFont = &FONT_9X18_BOLD;

const DEFAULT_FOREGROUND_COLOR: AsciiColor = AsciiColor::White;
const DEFAULT_FOREGROUND_STYLE: AsciiForegroundStyle = AsciiForegroundStyle::Regular;
const DEFAULT_FOREGROUND_HIGH_INTENSITY: bool = true;
const DEFAULT_BACKGROUND_COLOR: AsciiColor = AsciiColor::Black;
const DEFAULT_BACKGROUND_HIGH_INTENSITY: bool = true;

fn parse_number_from_chars(chars: impl Iterator<Item = char>) -> usize {
    let mut acc = 0;
    for c in chars {
        let c = c as u32;
        if c >= '0' as u32 && c <= '9' as u32 {
            acc *= 10;
            acc += c as usize - '0' as usize;
        }
    }
    acc
}

#[derive(Debug, Copy, Clone)]
enum AsciiEscapeCode {
    Foreground {
        color: AsciiColor,
        style: AsciiForegroundStyle,
        high_intensity: bool,
    },
    Background {
        color: AsciiColor,
        high_intensity: bool,
    },
    Reset,
}

#[derive(Debug, Copy, Clone)]
enum AsciiForegroundStyle {
    Regular,
    Bold,
    Underline,
}

#[derive(Debug, Copy, Clone)]
enum AsciiColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan,
    White,
}

fn color_from_byte(byte: u8) -> Option<AsciiColor> {
    match byte {
        0 => Some(AsciiColor::Black),
        1 => Some(AsciiColor::Red),
        2 => Some(AsciiColor::Green),
        3 => Some(AsciiColor::Yellow),
        4 => Some(AsciiColor::Blue),
        5 => Some(AsciiColor::Purple),
        6 => Some(AsciiColor::Cyan),
        7 => Some(AsciiColor::White),
        _ => None,
    }
}

fn parse_ascii_code(mut code_chars: impl Iterator<Item = char> + Clone) -> Option<AsciiEscapeCode> {
    let n1 = parse_number_from_chars(code_chars.clone().take_while(|c| c.is_ascii_digit()));
    match n1 {
        40..=47 => {
            // regular background: \x1b[40m to \x1b[47m
            Some(AsciiEscapeCode::Background {
                color: color_from_byte(n1 as u8 - 40)?,
                high_intensity: false,
            })
        }
        1 => {
            let separator = (&mut code_chars).skip_while(|c| c.is_ascii_digit()).next();
            if separator != Some(';') {
                return None;
            }
            let n2 = parse_number_from_chars(code_chars.take_while(|c| c.is_ascii_digit()));
            match n2 {
                // bold foreground: \x1b[1;30m to \x1b[1;37m
                30..=37 => Some(AsciiEscapeCode::Foreground {
                    color: color_from_byte(n2 as u8 - 30)?,
                    style: AsciiForegroundStyle::Bold,
                    high_intensity: false,
                }),
                // bold high-intensity foreground: \x1b[1;90m to \x1b[1;97m
                90..=97 => Some(AsciiEscapeCode::Foreground {
                    color: color_from_byte(n2 as u8 - 90)?,
                    style: AsciiForegroundStyle::Bold,
                    high_intensity: true,
                }),
                _ => None,
            }
        }
        0 => {
            let separator = (&mut code_chars).skip_while(|c| c.is_ascii_digit()).next();
            if separator.is_none() {
                // reset: \x1b[0m
                return Some(AsciiEscapeCode::Reset);
            }
            if separator != Some(';') {
                return None;
            }
            let n2 = parse_number_from_chars(code_chars.take_while(|c| c.is_ascii_digit()));
            match n2 {
                // regular foreground: \x1b[0;30m to \x1b[0;37m
                30..=37 => Some(AsciiEscapeCode::Foreground {
                    color: color_from_byte(n2 as u8 - 30)?,
                    style: AsciiForegroundStyle::Regular,
                    high_intensity: false,
                }),
                // regular high-intensity foreground: \x1b[0;90m to \x1b[0;97m
                90..=97 => Some(AsciiEscapeCode::Foreground {
                    color: color_from_byte(n2 as u8 - 90)?,
                    style: AsciiForegroundStyle::Regular,
                    high_intensity: true,
                }),
                // high-intensity background: \x1b[0;100m to \x1b[0;107m
                100..=107 => Some(AsciiEscapeCode::Background {
                    color: color_from_byte(n2 as u8 - 100)?,
                    high_intensity: true,
                }),
                _ => None,
            }
        }
        4 => {
            let separator = (&mut code_chars).skip_while(|c| c.is_ascii_digit()).nth(1);
            if separator != Some(';') {
                return None;
            }
            let n2 = parse_number_from_chars(code_chars.take_while(|c| c.is_ascii_digit()));
            match n2 {
                // underline foreground: \x1b[4;30m to \x1b[4;37m
                30..=37 => Some(AsciiEscapeCode::Foreground {
                    color: color_from_byte(n2 as u8 - 30)?,
                    style: AsciiForegroundStyle::Underline,
                    high_intensity: false,
                }),
                _ => None,
            }
        }
        _ => None,
    }
}

impl Write for FramebufferDisplay {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\x1b' && chars.peek() == Some(&'[') {
                // ASCII color codes
                let code_chars = chars.clone().skip(1).take_while(|c| *c != 'm');
                let code = parse_ascii_code(code_chars);
                if let Some(code) = code {
                    while let Some(c) = chars.next()
                        && c != 'm'
                    {}
                    match code {
                        AsciiEscapeCode::Foreground {
                            color,
                            style,
                            high_intensity,
                        } => {
                            self.foreground_color = color;
                            self.foreground_style = style;
                            self.foreground_high_intensity = high_intensity;
                        }
                        AsciiEscapeCode::Background {
                            color,
                            high_intensity,
                        } => {
                            self.background_color = color;
                            self.background_high_intensity = high_intensity;
                        }
                        AsciiEscapeCode::Reset => {
                            self.foreground_color = DEFAULT_FOREGROUND_COLOR;
                            self.foreground_style = DEFAULT_FOREGROUND_STYLE;
                            self.foreground_high_intensity = DEFAULT_FOREGROUND_HIGH_INTENSITY;
                            self.background_color = DEFAULT_BACKGROUND_COLOR;
                            self.background_high_intensity = DEFAULT_BACKGROUND_HIGH_INTENSITY;
                        }
                    }
                }
            } else {
                write_char(self, c);
            }
        }
        Ok(())
    }
}

fn ascii_color_to_rgb888(color: AsciiColor, high_intensity: bool) -> Rgb888 {
    match high_intensity {
        false => match color {
            AsciiColor::Black => Rgb888::CSS_DARK_GRAY,
            AsciiColor::Red => Rgb888::CSS_DARK_RED,
            AsciiColor::Green => Rgb888::CSS_GREEN,
            AsciiColor::Yellow => Rgb888::CSS_YELLOW,
            AsciiColor::Blue => Rgb888::CSS_BLUE,
            AsciiColor::Purple => Rgb888::CSS_DARK_MAGENTA,
            AsciiColor::Cyan => Rgb888::CSS_CYAN,
            AsciiColor::White => Rgb888::CSS_WHITE,
        },
        true => match color {
            AsciiColor::Black => Rgb888::CSS_BLACK,
            AsciiColor::Red => Rgb888::CSS_RED,
            AsciiColor::Green => Rgb888::CSS_LIGHT_GREEN,
            AsciiColor::Yellow => Rgb888::CSS_LIGHT_YELLOW,
            AsciiColor::Blue => Rgb888::CSS_LIGHT_BLUE,
            AsciiColor::Purple => Rgb888::CSS_MAGENTA,
            AsciiColor::Cyan => Rgb888::CSS_LIGHT_CYAN,
            AsciiColor::White => Rgb888::CSS_WHITE,
        },
    }
}

fn write_char(display: &mut FramebufferDisplay, c: char) {
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
    let _ = Rectangle::new(pixel_coord, FONT.character_size)
        .into_styled(
            PrimitiveStyleBuilder::<Rgb888>::new()
                .fill_color(ascii_color_to_rgb888(
                    display.background_color,
                    display.background_high_intensity,
                ))
                .build(),
        )
        .draw(display);
    let text_style_builder = MonoTextStyleBuilder::new().text_color(ascii_color_to_rgb888(
        display.foreground_color,
        display.foreground_high_intensity,
    ));
    let text_style_builder = match display.foreground_style {
        AsciiForegroundStyle::Regular => text_style_builder.font(FONT),
        AsciiForegroundStyle::Bold => text_style_builder.font(FONT_BOLD),
        AsciiForegroundStyle::Underline => text_style_builder.font(FONT).underline(),
    };
    let _ = Text::new(char_str, pixel_coord, text_style_builder.build()).draw(display);
}

fn scroll(display: &mut FramebufferDisplay) {
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

pub struct FramebufferDisplay {
    framebuffer: &'static limine::framebuffer::Framebuffer,
    cursor: Point,
    framebuffer_char_size: Size,
    foreground_color: AsciiColor,
    foreground_style: AsciiForegroundStyle,
    foreground_high_intensity: bool,
    background_color: AsciiColor,
    background_high_intensity: bool,
}

unsafe impl Send for FramebufferDisplay {}

impl OriginDimensions for FramebufferDisplay {
    fn size(&self) -> Size {
        Size::new(
            self.framebuffer.width as u32,
            self.framebuffer.height as u32,
        )
    }
}

impl DrawTarget for FramebufferDisplay {
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

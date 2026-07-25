#![no_std]
#![no_main]

mod early_console;
mod panic;
mod requests;
mod spinlock;
mod start;

unsafe fn main() -> ! {
    let framebuffer = requests::framebuffer().expect("No framebuffer available");
    unsafe {
        early_console::init(framebuffer);
        early_console::write_str("\n");
        early_console::write_str("Hello World!");
        let chars = ['a', 'b', 'c', 'd', 'e', 'f'];
        for i in 0..150 {
            let c = chars[i % chars.len()];
            let mut char_str = [0u8; 4];
            let char_str = c.encode_utf8(&mut char_str);
            early_console::write_str(char_str);
            early_console::write_str("\n");
            for _ in 0..1000000 {
                core::hint::spin_loop();
            }
        }
    }
    loop {}
}

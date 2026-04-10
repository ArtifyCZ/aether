#![no_std]
#![no_main]

extern crate aether_rt;

use core::arch::asm;
use core::fmt::Write;

fn print(message: &str) {
    let buffer = message.as_bytes();
    unsafe {
        aether_sys::sys_write(1, buffer.as_ptr(), buffer.len()).unwrap();
    }
}

struct Logger;

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        print(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = writeln!($crate::Logger, $($arg)*);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn main(ipc_base: usize) {
    unsafe {
        // @FIXME: For some reason, with the println it crashes on aarch64.
        #[cfg(not(target_arch = "aarch64"))]
        println!("Hello world from hello_world binary!");
        #[cfg(target_arch = "aarch64")]
        print("Hello world from hello_world binary!\n");
        loop {}
    }
}

#![no_std]
#![no_main]

extern crate aether_rt;

use core::arch::asm;
use core::fmt::Write;

unsafe extern "C" fn sys_write(fd: i32, buffer: *const u8, size: usize) {
    unsafe {
        let error_code: u64;
        let result: u64;
        const SYSCALL_NUMBER: u64 = 0x01u64;
        #[cfg(target_arch = "x86_64")]
        asm!(
            "syscall",
            inout("rax") SYSCALL_NUMBER => result,
            in("rdi") fd as u64,
            in("rsi") buffer as u64,
            inout("rdx") size as u64 => error_code,
        );
        #[cfg(target_arch = "aarch64")]
        asm!(
            "svc #0",
            in("x8") SYSCALL_NUMBER,
            inout("x0") fd as u64 => result,
            inout("x1") buffer as u64 => error_code,
            in("x2") size as u64,
        );
        if error_code != 0 {
            loop {}
        }
        let _ = result;
    }
}

fn print(message: &str) {
    let buffer = message.as_bytes();
    unsafe {
        sys_write(1, buffer.as_ptr(), buffer.len());
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

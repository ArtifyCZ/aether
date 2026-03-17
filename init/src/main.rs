#![no_std]
#![no_main]

use core::arch::asm;
use core::fmt::Write;
use init_contract_rust::boot_info;

unsafe extern "C" fn sys_write(fd: i32, buffer: *const u8, size: usize) {
    unsafe {
        let error_code: u64;
        let result: u64;
        asm!(
            "syscall",
            inout("rax") 0x01u64 => result,
            in("rdi") fd as u64,
            in("rsi") buffer as u64,
            inout("rdx") (size as u64) => error_code,
        );
        if error_code != 0 {
            loop {
            }
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
unsafe extern "C" fn _start(boot_info: *mut boot_info) -> ! {
    rmain(boot_info);
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {
    }
}

unsafe extern "C" {
    fn main(boot_info: *mut boot_info);
}

fn rmain(boot_info: *mut boot_info) -> ! {
    println!("Hello Rust init world!");

    unsafe {
        main(boot_info);
    }

    loop {
    }
}

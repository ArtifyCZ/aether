#![no_std]
#![no_main]

extern crate aether_rt;
extern crate alloc;

use alloc::boxed::Box;
use core::arch::asm;
use core::ffi::{c_char, c_void};
use core::fmt::Write;
use core::ptr::null_mut;
use init_contract_rust::boot_info;

use crate::tarball_parsing::parse_tarball_archive;

mod tarball_parsing;

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
#[unsafe(naked)] // CRITICAL: No compiler prologue/epilogue
pub unsafe extern "C" fn _entry(boot_info: *mut boot_info) -> ! {
    #[cfg(target_arch = "x86_64")]
    core::arch::naked_asm!(
        "xor rbp, rbp",      // Clear frame pointer for clean backtrace
        "mov rdi, rdi",      // boot_info is already in rdi, but this is for clarity
        "and rsp, -16",      // Align stack to 16 bytes
        "sub rsp, 8",        // Standard ABI: stack should be (16n + 8) at function entry
                             // (because the 'call' instruction pushes an 8-byte return address)
        "call {rmain}",
        "1: pause",
        "jmp 1b",
        rmain = sym rmain,
    );

    #[cfg(target_arch = "aarch64")]
    core::arch::naked_asm!(
        "mov x29, #0",
        "mov x30, #0",
        "mov x1, sp",
        "and x1, x1, #0xfffffffffffffff0",
        "sub x1, x1, #16",
        "mov sp, x1",
        "bl {rmain}",
        "1: wfe",
        "b 1b",
        rmain = sym rmain,
    );
}

unsafe extern "C" {
    fn main(boot_info: *mut boot_info);
}

#[unsafe(no_mangle)]
unsafe extern "C" fn tar_find_file(
    tar_addr: *mut c_void,
    tar_size: usize,
    filename: *const c_char,
    file_data: *mut *mut c_void,
    file_size: *mut usize,
) {
    let tar_data: &[u8] = core::slice::from_raw_parts(tar_addr.cast(), tar_size);
    let filename = unsafe { core::ffi::CStr::from_ptr(filename) };

    let first_byte = tar_data[0];

    let tarball = parse_tarball_archive(tar_data).expect("Failed to parse a tarball!");
    let file = tarball.iter().find(|h| h.name == filename);

    match file {
        Some(file) => unsafe {
            file_data.write(file.file_data.as_ptr() as *mut u8 as *mut c_void);
            file_size.write(file.size);
        },
        None => {
            file_data.write(null_mut());
            file_size.write(0);
        }
    }
}

fn rmain(boot_info: *mut boot_info) -> ! {
    println!("Hello Rust init world!");
    // @FIXME: support for PIC needed to be able to use formatting
    // println!("Boot info at: {:p}", boot_info);

    unsafe {
        main(boot_info);
    }

    loop {}
}

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

use crate::elf_loading::load_elf_program;
use crate::elf_parsing::parse_elf_file;
use crate::tarball_parsing::parse_tarball_archive;
use aether_sys::sys_write;

mod elf_loading;
mod elf_parsing;
mod tarball_parsing;

fn print(message: &str) {
    let buffer = message.as_bytes();
    unsafe {
        sys_write(1, buffer.as_ptr(), buffer.len()).unwrap();
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
unsafe extern "C" fn elf_load(
    data: *mut c_void,
    data_length: usize,
    out_vaddr_entrypoint: *mut usize,
    out_proc_handle: *mut u64,
) -> i32 {
    let data = unsafe { core::slice::from_raw_parts(data.cast(), data_length) };
    let Ok(elf) = parse_elf_file(data) else {
        return -1;
    };

    let (proc_handle, entrypoint) = load_elf_program(&elf);

    unsafe {
        out_vaddr_entrypoint.write(entrypoint);
        out_proc_handle.write(proc_handle);
    }

    0
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

unsafe extern "C" {
    fn serial_init() -> bool;
}

unsafe extern "C" fn second_thread() -> ! {
    println!("Hello from second thread in Rust!");
    for j in 0u8..10u8 {
        for _ in 0..1000000 {
            // Busy wait
            core::hint::spin_loop();
        }
        let number = [j + b'0', b'\n', 0x00];
        unsafe {
            aether_sys::sys_write(1, number.as_ptr(), number.len());
        }
    }

    unsafe {
        aether_sys::sys_exit();
    }
    panic!("This should never be reached!");
}

fn rmain(boot_info: *mut boot_info) -> ! {
    println!("Hello Rust init world!");
    // @FIXME: support for PIC needed to be able to use formatting
    // println!("Boot info at: {:p}", boot_info);
    if unsafe { serial_init() } {
        panic!("Failed to initialize the serial driver");
    }
    let stack_size = 0x4000;
    let stack_base = unsafe {
        aether_sys::sys_proc_mmap(
            0,
            0x7FFFFFFF8000 as *mut u8,
            stack_size as *mut u8,
            aether_sys::SYS_PROT_READ | aether_sys::SYS_PROT_WRITE,
            0,
        )
    }
    .unwrap();
    if stack_base.addr() < 0x400000 {
        panic!("Mmap failed or returned invalid address!");
    }
    let stack_top = unsafe { stack_base.add(stack_size) };
    unsafe { aether_sys::sys_proc_spawn(0, 0, stack_top, second_thread as *mut u8, 0) }.unwrap();

    unsafe {
        main(boot_info);
    }

    loop {}
}

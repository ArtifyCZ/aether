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

mod legacy_bindings {
    use super::sys_write;

    #[unsafe(no_mangle)]
    unsafe extern "C" fn print(message: *const u8) {
        let mut length = 0;
        while unsafe { *message.add(length) } != 0 {
            length += 1;
        }
        unsafe {
            sys_write(1, message, length).unwrap();
        }
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
    fn serial_init() -> bool;
    fn keyboard_init();
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

    println!("Parent is moving on...");
    unsafe {
        keyboard_init();
    }
    println!("Keyboard initialized!");

    let initrd = unsafe {
        let initrd_start = boot_info.read().initrd_start;
        let initrd_len = boot_info.read().initrd_size;
        core::slice::from_raw_parts(initrd_start as *mut u8, initrd_len as usize)
    };
    let initrd = parse_tarball_archive(initrd).unwrap();
    let hello_world_elf = initrd
        .iter()
        .find(|file| file.name == c"bin/hello_world")
        .unwrap();
    println!("/bin/hello_world found!");
    let hello_world_elf = parse_elf_file(hello_world_elf.file_data).unwrap();
    let (hello_world_handle, hello_world_entrypoint) = load_elf_program(&hello_world_elf);
    let stack_size = 0x4000;
    let stack_base = unsafe {
        aether_sys::sys_proc_mmap(
            hello_world_handle,
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
    println!("Spawning hello_world process...");
    unsafe {
        aether_sys::sys_proc_spawn(
            hello_world_handle,
            0,
            stack_top,
            hello_world_entrypoint as *mut u8,
            0,
        )
    }
    .unwrap();
    println!("hello_world process spawned!");

    loop {}
}

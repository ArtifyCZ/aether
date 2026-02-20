#![no_std]
#![no_main]

extern crate alloc;

mod allocator;
mod entrypoint;
mod interrupt_safe_spin_lock;
mod platform;
mod scheduler;
mod spin_lock;

use crate::platform::drivers::keyboard::KeyboardDriver;
use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::string::ToString;
use alloc::sync::Arc;
use core::ffi::c_void;
use core::ops::DerefMut;
use core::str::FromStr;

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &::core::panic::PanicInfo) -> ! {
    unsafe {
        SerialDriver::write(b"Panic occurred, attempting to print message:\n");
        let message = info.to_string();
        SerialDriver::println(&message);
        hcf()
    }
}

unsafe extern "C" {
    /// Halt and catch fire; implemented in C
    fn hcf() -> !;
}

use crate::platform::drivers::serial::SerialDriver;
use crate::platform::elf::Elf;
use crate::platform::interrupts::Interrupts;
use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::modules::Modules;
use crate::platform::platform::Platform;
use crate::platform::scheduler::Scheduler;
use crate::platform::syscalls::{Syscalls, syscall_args};
use crate::platform::tasks::Task;
use crate::platform::terminal::Terminal;
use crate::platform::ticker::Ticker;
use crate::platform::timer::Timer;
use crate::platform::virtual_memory_manager_context::VirtualMemoryManagerContext;

fn thread_heartbeat() {
    let mut i = 0;
    loop {
        if i == 2000000 {
            unsafe {
                Terminal::print_char('.');
            }
            i = 0;
        }
        i += 1;
    }
}

fn thread_keyboard() {
    loop {
        unsafe {
            if let Some(c) = KeyboardDriver::get_char() {
                Terminal::print_char(c);
            }
        }
    }
}

fn spawn_thread<F>(scheduler: &mut Scheduler, f: F)
where
    F: FnOnce() + 'static,
{
    unsafe extern "C" fn trampoline<F>(args: *mut c_void)
    where
        F: FnOnce() + 'static,
    {
        let f: Box<F> = unsafe { Box::from_raw(args.cast()) };
        f();
        todo!("Invoking syscalls from Rust not implemented yet (should use sys_exit)")
    }

    let arg = Box::into_raw(Box::new(f)).cast();

    let task = Task::new_kernel(trampoline::<F>, arg, 4 * PAGE_FRAME_SIZE);
    scheduler.add(task);
}

fn main(hhdm_offset: u64, rsdp_address: u64) {
    unsafe {
        Interrupts::init();
        Platform::init(rsdp_address);

        SerialDriver::println("Hello from Rust!");

        let scheduler = Scheduler::init();

        Syscalls::init(scheduler);
        Elf::init(hhdm_offset);
        Timer::init(100);

        KeyboardDriver::init();

        Ticker::init(scheduler);

        spawn_thread(scheduler.lock().deref_mut(), thread_heartbeat);
        spawn_thread(scheduler.lock().deref_mut(), thread_keyboard);

        {
            let init_elf_string = CString::from_str("init.elf").expect("Failed to create CString");
            let init_elf = Modules::find(init_elf_string.as_c_str()).unwrap();
            let mut init_ctx = VirtualMemoryManagerContext::create();
            let entrypoint_vaddr = Elf::load(&mut init_ctx, init_elf).unwrap();
            let task = Task::new_user(Arc::new(init_ctx), entrypoint_vaddr);
            scheduler.lock().add(task);
        }

        scheduler.lock().start();

        loop {
            platform::syscalls::sys_exit();
            hcf();
            // Now just wait for the first interrupt
            // @TODO: use yield or exit syscall
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));

            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
        }
    }
}

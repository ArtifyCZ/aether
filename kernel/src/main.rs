#![no_std]
#![no_main]

extern crate alloc;

mod allocator;
mod entrypoint;
mod init_process;
mod interrupt_safe_spin_lock;
mod logging;
mod platform;
mod scheduler;
mod spin_lock;
mod syscall_handler;
mod task_id;
mod task_registry;
mod ticker;

use crate::init_process::spawn_init_process;
use crate::platform::platform::Platform;
use alloc::boxed::Box;
use alloc::string::ToString;
use core::ffi::c_void;

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        logging::switch_to_emergency_console();
    }
    println!();
    println!("====================");
    println!("    KERNEL PANIC    ");
    println!("====================");
    println!("{}", info);
    unsafe { hcf() }
}

unsafe extern "C" {
    /// Halt and catch fire; implemented in C
    fn hcf() -> !;
}

use crate::platform::elf::Elf;
use crate::platform::interrupts::Interrupts;
use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::syscalls::{Syscalls, sys_exit};
use crate::platform::terminal::Terminal;
use crate::platform::timer::Timer;
use crate::syscall_handler::SyscallHandler;
use crate::task_registry::{TaskRegistry, TaskSpec};
use scheduler::Scheduler;
use ticker::Ticker;

fn thread_heartbeat() {
    let mut i = 0;
    loop {
        if i == 20000000 {
            unsafe {
                Terminal::print_char('.');
            }
            i = 0;
        }
        i += 1;
    }
}

fn spawn_thread<F>(scheduler: &Scheduler, f: F)
where
    F: FnOnce() + 'static,
{
    unsafe extern "C" fn trampoline<F>(args: *mut c_void)
    where
        F: FnOnce() + 'static,
    {
        let f: Box<F> = unsafe { Box::from_raw(args.cast()) };
        f();
        unsafe { sys_exit() };
    }

    let arg = Box::into_raw(Box::new(f)).cast();
    scheduler.spawn(TaskSpec::Kernel {
        function: trampoline::<F>,
        arg,
        kernel_stack_size: 4 * PAGE_FRAME_SIZE,
    });
}

fn main(hhdm_offset: u64, rsdp_address: u64) {
    unsafe {
        Interrupts::init();
        Platform::init(rsdp_address);

        println!("Hello from Rust!");
        let registry = TaskRegistry::new();

        let scheduler = Scheduler::init(registry);

        let syscall_handler = SyscallHandler::init(scheduler, registry);
        Syscalls::init(|ctx| syscall_handler.handle(ctx));
        Interrupts::set_irq_handler(|frame, irq| {
            Interrupts::mask_irq(irq);
            scheduler.signal_irq(irq, frame).unwrap_or(frame)
        });
        Elf::init(hhdm_offset);
        Timer::init(100);

        Ticker::init(scheduler);

        spawn_thread(scheduler, thread_heartbeat);
        spawn_init_process(scheduler);

        scheduler.start();

        loop {
            sys_exit();
            hcf()
        }
    }
}

#![no_std]
#![no_main]

extern crate alloc;

mod allocator;
mod elf;
pub mod entrypoint;
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
use core::ffi::c_void;
use core::ptr::NonNull;

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

use crate::elf::Elf;
use crate::platform::early_console::EarlyConsole;
use crate::platform::interrupts::Interrupts;
use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::modules::Modules;
use crate::platform::physical_memory_manager::PhysicalMemoryManager;
use crate::platform::syscalls::{Syscalls, sys_exit};
use crate::platform::terminal::Terminal;
use crate::platform::virtual_address_allocator::VirtualAddressAllocator;
use crate::platform::virtual_memory_manager::VirtualMemoryManager;
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
    unsafe extern "C" fn trampoline<F>(args: *mut c_void) -> !
    where
        F: FnOnce() + 'static,
    {
        let f: Box<F> = unsafe { Box::from_raw(args.cast()) };
        f();
        unsafe { sys_exit() };
        loop {}
    }

    let arg = Box::into_raw(Box::new(f)).cast();
    scheduler.spawn(TaskSpec::Kernel {
        function: trampoline::<F>,
        arg,
        kernel_stack_size: 4 * PAGE_FRAME_SIZE,
    });
}

fn main(
    hhdm_offset: u64,
    memmap: *mut kernel_bindings_gen::limine_memmap_response,
    framebuffer: *mut kernel_bindings_gen::limine_framebuffer,
    modules: *mut kernel_bindings_gen::limine_module_response,
    rsdp_address: u64,
) -> ! {
    unsafe {
        VirtualAddressAllocator::init();
        PhysicalMemoryManager::init(memmap);
        VirtualMemoryManager::init(hhdm_offset);
        #[cfg(target_arch = "x86_64")]
        const SERIAL_BASE: usize = 0x3f8;
        #[cfg(target_arch = "aarch64")]
        const SERIAL_BASE: usize = 0x9000000;
        EarlyConsole::init(SERIAL_BASE);
        println!("\nEarly console initialized!\n");
        Modules::init(modules);
        Platform::init(rsdp_address);
        Terminal::init(NonNull::new(framebuffer).unwrap());
        println!("Terminal initialized!");
        println!("Booting...");
        Interrupts::init();

        println!("Hello from Rust!");
        let registry = TaskRegistry::new();
        println!("Foo1");

        let scheduler = Scheduler::init(registry);
        println!("Bar1");

        let syscall_handler = SyscallHandler::init(scheduler, registry);
        Syscalls::init(|ctx| syscall_handler.handle(ctx));
        Interrupts::set_irq_handler(|frame, irq| {
            Interrupts::mask_irq(irq);
            scheduler.signal_irq(irq, frame)
        });
        println!("Baz1");
        Ticker::init(100, scheduler);
        Interrupts::enable();
        println!("Qaz1");

        spawn_thread(scheduler, thread_heartbeat);
        println!("Gez1");
        let elf = Elf::init(hhdm_offset);
        spawn_init_process(&elf, scheduler);

        println!("Disabling early console...");
        logging::disable_early_console();

        scheduler.start();

        loop {
            sys_exit();
            hcf()
        }
    }
}

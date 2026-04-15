#![no_std]
#![no_main]

extern crate alloc;

pub mod allocator;
mod args;
pub mod boot;
mod elf_loading;
mod elf_parsing;
mod init_process;
mod interrupt_safe_spin_lock;
mod logging;
mod platform;
mod scheduler;
mod spin_lock;
mod syscall_handler;
mod tarball_parsing;
mod task_id;
mod task_registry;
mod ticker;

use crate::boot::BootInfo;
use crate::init_process::spawn_init_process;
use crate::platform::platform::Platform;
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

use crate::args::KernelArgs;
use crate::platform::early_console::EarlyConsole;
use crate::platform::interrupts::Interrupts;
use crate::platform::modules::Modules;
use crate::platform::physical_memory_manager::PhysicalMemoryManager;
use crate::platform::syscalls::Syscalls;
use crate::platform::terminal::Terminal;
use crate::platform::virtual_address_allocator::VirtualAddressAllocator;
use crate::platform::virtual_memory_manager::VirtualMemoryManager;
use crate::syscall_handler::SyscallHandler;
use crate::task_registry::TaskRegistry;
use scheduler::Scheduler;
use ticker::Ticker;

static PAGED_ALLOCATOR: allocator::Allocator = unsafe { allocator::Allocator::init() };

pub fn main(
    cmdline: &str,
    hhdm_offset: u64,
    memmap: *mut kernel_bindings_gen::limine_memmap_response,
    framebuffer: *mut kernel_bindings_gen::limine_framebuffer,
    rsdp_address: u64,
    switch_to_paged_allocator: impl FnOnce(*const allocator::Allocator),
    boot_info: impl BootInfo,
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
        Modules::init(boot_info.get_modules());
        Platform::init(rsdp_address);
        Terminal::init(NonNull::new(framebuffer).unwrap());
        println!("Terminal initialized!");
        let args = KernelArgs::parse(cmdline);
        println!("Cmdline: {:?}", args);
        println!("Booting...");
        Interrupts::init();
        println!("Switching to paged allocator...");
        switch_to_paged_allocator(&raw const PAGED_ALLOCATOR);
        println!("Switched to paged allocator!");

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

        println!("Gez1");
        spawn_init_process(args.init_program, scheduler, hhdm_offset as usize);

        println!("Disabling early console...");
        logging::disable_early_console();

        kernel_hal::tasks::switch_to(scheduler.start())
    }
}

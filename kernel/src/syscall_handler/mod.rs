mod sys_exit;
mod sys_write;
mod sys_clone;

use crate::platform::drivers::serial::SerialDriver;
use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::physical_memory_manager::PhysicalMemoryManager;
use crate::platform::syscalls::{SyscallContext, SyscallIntent, syscall_num};
use crate::platform::terminal::Terminal;
use crate::platform::virtual_memory_manager_context::VirtualMemoryMappingFlags;
use crate::platform::virtual_page_address::VirtualPageAddress;
use crate::scheduler::Scheduler;
use crate::syscall_handler::sys_exit::SysExitCommand;
use crate::task_registry::TaskSpec;
use alloc::boxed::Box;
use alloc::format;
use core::ptr::slice_from_raw_parts;
use crate::syscall_handler::sys_clone::SysCloneCommand;
use crate::syscall_handler::sys_write::SysWriteCommand;

pub struct SyscallHandler {
    scheduler: &'static Scheduler,
}

pub trait SyscallCommand: Sized {
    fn parse<'a>(ctx: &SyscallContext<'a>) -> Option<Self>
    where
        Self: 'a;
}

pub trait SyscallCommandHandler<TSyscallCommand> {
    fn handle_command(&self, command: TSyscallCommand) -> SyscallIntent;
}

impl SyscallHandler {
    pub fn init(scheduler: &'static Scheduler) -> &'static Self {
        let syscall_handler: &'static Self = Box::leak(Box::new(SyscallHandler { scheduler }));
        syscall_handler
    }

    pub fn handle(&self, ctx: &SyscallContext<'_>) -> SyscallIntent {
        match ctx.num {
            syscall_num::SYS_EXIT => self.handle_command(SysExitCommand::parse(ctx).unwrap()),
            syscall_num::SYS_WRITE => self.handle_command(SysWriteCommand::parse(ctx).unwrap()),
            syscall_num::SYS_CLONE => self.handle_command(SysCloneCommand::parse(ctx).unwrap()),
            syscall_num::SYS_MMAP => self.sys_mmap(ctx),
            _ => panic!("Non-existent syscall triggered!"), // @TODO: add better handling
        }
    }

    fn sys_mmap(&self, ctx: &SyscallContext<'_>) -> SyscallIntent {
        let addr = ctx.args[0] as usize;
        let length = ctx.args[1] as usize;
        let _prot = ctx.args[2] as u32;
        let _flags = ctx.args[3] as u32;

        if addr >= 0x800000000000 || (addr + length) >= 0x800000000000 {
            unsafe {
                SerialDriver::println("mmap: EFAULT: Bad Address");
                SerialDriver::println(&format!("addr: {}; len: {}", addr, length));
            }
            return SyscallIntent::Return(0);
        }

        const PAGE_MASK: usize = !(PAGE_FRAME_SIZE - 1);
        let addr = addr & PAGE_MASK;
        let pages_count = (length + PAGE_FRAME_SIZE - 1) / PAGE_FRAME_SIZE;
        // @TODO: implement protection flags
        // @TODO: implement flags

        for page_idx in 0..pages_count {
            let page_vaddr = VirtualPageAddress::new(addr + page_idx * PAGE_FRAME_SIZE).unwrap();
            let phys = unsafe { PhysicalMemoryManager::alloc_frame() }.unwrap();
            unsafe {
                self.scheduler.access_current_task_context(|task| {
                    task.get_virtual_memory_manager().map_page(
                        page_vaddr,
                        phys,
                        VirtualMemoryMappingFlags::PRESENT
                            | VirtualMemoryMappingFlags::USER
                            | VirtualMemoryMappingFlags::WRITE,
                    )
                });
            }
        }

        SyscallIntent::Return(addr as u64)
    }
}

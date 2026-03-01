use alloc::format;
use crate::platform::drivers::serial::SerialDriver;
use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::physical_memory_manager::PhysicalMemoryManager;
use crate::platform::syscalls::{SyscallContext, SyscallIntent};
use crate::platform::virtual_memory_manager_context::VirtualMemoryMappingFlags;
use crate::platform::virtual_page_address::VirtualPageAddress;
use crate::syscall_handler::{SyscallCommand, SyscallCommandHandler, SyscallHandler};

pub struct SysMmapCommand {
    addr: usize,
    length: usize,
    // @TODO: implement protection flags
    #[allow(unused)]
    prot: u32,
    // @TODO: implement flags
    #[allow(unused)]
    flags: u32,
}

impl SyscallCommand for SysMmapCommand {
    fn parse<'a>(ctx: &SyscallContext<'a>) -> Option<Self>
    where
        Self: 'a
    {
        Some(Self {
            addr: ctx.args[0] as usize,
            length: ctx.args[1] as usize,
            prot: ctx.args[2] as u32,
            flags: ctx.args[3] as u32,
        })
    }
}

impl SyscallCommandHandler<SysMmapCommand> for SyscallHandler {
    fn handle_command(&self, command: SysMmapCommand) -> SyscallIntent {

        if command.addr >= 0x800000000000 || (command.addr + command.length) >= 0x800000000000 {
            unsafe {
                SerialDriver::println("mmap: EFAULT: Bad Address");
                SerialDriver::println(&format!("addr: {}; len: {}", command.addr, command.length));
            }
            return SyscallIntent::Return(0);
        }

        const PAGE_MASK: usize = !(PAGE_FRAME_SIZE - 1);
        let addr = command.addr & PAGE_MASK;
        let pages_count = (command.length + PAGE_FRAME_SIZE - 1) / PAGE_FRAME_SIZE;

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

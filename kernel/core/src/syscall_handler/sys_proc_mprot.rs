use kernel_hal::mmu::VirtualMemoryMappingFlags;
use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::syscalls::{SyscallContext, SyscallError, SyscallIntent};
use crate::platform::virtual_page_address::VirtualPageAddress;
use crate::syscall_handler::user_ptr::UserPtr;
use crate::syscall_handler::user_slice::UserSlice;
use crate::syscall_handler::{SyscallCommand, SyscallCommandHandler, SyscallHandler};
use crate::task_id::TaskId;

pub struct SysProcMprotCommand {
    proc_handle: u64,
    chunk: UserSlice<*const [u8]>,
    prot: u32,
}

impl SyscallCommand for SysProcMprotCommand {
    type Error = SyscallError;

    fn parse<'a>(ctx: &SyscallContext<'a>) -> Result<Self, Self::Error>
    where
        Self: 'a,
    {
        let proc_handle = ctx.args[0] as u64;
        let addr = ctx.args[1] as usize;
        let length = ctx.args[2] as usize;
        let prot = ctx.args[3] as u32;

        let chunk_start = UserPtr::try_from(addr)?;
        let chunk = UserSlice::try_from((chunk_start, length))?;

        Ok(Self {
            proc_handle,
            chunk,
            prot,
        })
    }
}

impl SyscallCommandHandler<SysProcMprotCommand> for SyscallHandler {
    type Ok = ();
    type Err = SyscallError;

    fn handle_command(
        &self,
        command: SysProcMprotCommand,
    ) -> Result<SyscallIntent<Self::Ok>, Self::Err> {
        let task_id = TaskId::get_current().expect("Scheduler is not started yet!");
        let task = self
            .task_registry
            .get(task_id)
            .expect("Current task should exist!");
        let target_vmm = if command.proc_handle == 0 {
            task.get_virtual_memory_manager()
        } else {
            task.get_proc_handle(command.proc_handle)
                .expect("Not assigned proc handle not handled")
        };
        const PAGE_MASK: usize = !(PAGE_FRAME_SIZE - 1);
        let addr: UserPtr<usize> = UserPtr::try_from(command.chunk.addr() & PAGE_MASK)?;
        let pages_count = (command.chunk.len() + PAGE_FRAME_SIZE - 1) / PAGE_FRAME_SIZE;

        for page_idx in 0..pages_count {
            let page_vaddr = UserPtr::try_from(*addr + page_idx * PAGE_FRAME_SIZE)?;
            let page_vaddr = VirtualPageAddress::new(*page_vaddr).unwrap();
            let phys = unsafe { target_vmm.translate(page_vaddr) }
                .expect("Failed to translate!")
                .expect("Address not mapped!");
            let mut flags = VirtualMemoryMappingFlags::PRESENT | VirtualMemoryMappingFlags::USER;
            if command.prot & syscalls_rust::SYS_PROT_WRITE != 0 {
                flags.insert(VirtualMemoryMappingFlags::WRITE);
            } else if command.prot & syscalls_rust::SYS_PROT_EXEC != 0 {
                flags.insert(VirtualMemoryMappingFlags::EXEC);
            }
            let flags = flags;
            unsafe {
                // @TODO: actually update the flags instead of unmapping and mapping back
                target_vmm
                    .unmap_page(page_vaddr)
                    .expect("Unmapping failed!");
                target_vmm
                    .map_page(page_vaddr, phys, flags)
                    .expect("Mapping failed!");
            }
        }

        Ok(SyscallIntent::Return(()))
    }
}

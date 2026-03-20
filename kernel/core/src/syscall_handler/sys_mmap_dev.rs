use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::physical_page_frame::PhysicalPageFrame;
use crate::platform::syscalls::{SyscallContext, SyscallError, SyscallIntent};
use crate::platform::virtual_page_address::VirtualPageAddress;
use crate::syscall_handler::user_ptr::UserPtr;
use crate::syscall_handler::user_slice::UserSlice;
use crate::syscall_handler::{SyscallCommand, SyscallCommandHandler, SyscallHandler};
use crate::task_id::TaskId;
use alloc::boxed::Box;
use kernel_hal::mmu::VirtualMemoryMappingFlags;
use kernel_hal::tasks::TaskFrame;

pub struct SysMmapDevCommand {
    task_frame: Box<TaskFrame>,
    chunk: UserSlice<*const [u8]>,
    phys: usize,
    // @TODO: implement protection flags
    #[allow(unused)]
    prot: u32,
    // @TODO: implement flags
    #[allow(unused)]
    flags: u32,
}

impl SyscallCommand for SysMmapDevCommand {
    type Error = SyscallError;

    fn parse(ctx: SyscallContext) -> Result<Self, (Box<TaskFrame>, Self::Error)> {
        let task_frame = ctx.task_frame;
        let addr = ctx.args[0] as usize;
        let length = ctx.args[1] as usize;
        let phys = ctx.args[2] as usize;
        let prot = ctx.args[3] as u32;
        let flags = ctx.args[4] as u32;

        let chunk_start = match UserPtr::try_from(addr) {
            Ok(chunk_start) => chunk_start,
            Err(err) => return Err((task_frame, err)),
        };
        let chunk = match UserSlice::try_from((chunk_start, length)) {
            Ok(chunk) => chunk,
            Err(err) => return Err((task_frame, err)),
        };

        Ok(Self {
            task_frame,
            chunk,
            phys,
            prot,
            flags,
        })
    }
}

impl SyscallCommandHandler<SysMmapDevCommand> for SyscallHandler {
    type Ok = UserPtr<usize>;
    type Err = SyscallError;

    fn handle_command(
        &self,
        command: SysMmapDevCommand,
    ) -> Result<SyscallIntent<Self::Ok>, (Box<TaskFrame>, Self::Err)> {
        // @TODO: add checks for the physical address
        // @TODO: add authorization checks (is the task authorized to map this device?)
        let task_id = TaskId::get_current().expect("Scheduler is not started yet!");
        let task = self
            .task_registry
            .get(task_id)
            .expect("Current task should exist!");
        let vmm = task.get_virtual_memory_manager();
        const PAGE_MASK: usize = !(PAGE_FRAME_SIZE - 1);
        let addr = match UserPtr::try_from(command.chunk.addr() & PAGE_MASK) {
            Ok(addr) => addr,
            Err(err) => return Err((command.task_frame, err)),
        };
        let phys = command.phys & PAGE_MASK;
        let pages_count = (command.chunk.len() + PAGE_FRAME_SIZE - 1) / PAGE_FRAME_SIZE;

        for page_idx in 0..pages_count {
            let page_vaddr = match UserPtr::try_from(*addr + page_idx * PAGE_FRAME_SIZE) {
                Ok(page_vaddr) => page_vaddr,
                Err(err) => return Err((command.task_frame, err)),
            };
            let page_vaddr = VirtualPageAddress::new(*page_vaddr).unwrap();
            let phys = PhysicalPageFrame::new(phys + page_idx * PAGE_FRAME_SIZE).unwrap();
            unsafe {
                vmm.map_page(
                    page_vaddr,
                    phys,
                    VirtualMemoryMappingFlags::PRESENT
                        | VirtualMemoryMappingFlags::USER
                        | VirtualMemoryMappingFlags::WRITE
                        | VirtualMemoryMappingFlags::DEVICE,
                )
                .expect("Mapping failed!");
            }
        }

        Ok(SyscallIntent::Return(command.task_frame, addr))
    }
}

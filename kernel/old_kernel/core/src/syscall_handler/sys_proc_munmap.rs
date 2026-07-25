use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::syscalls::{SyscallContext, SyscallError, SyscallIntent};
use crate::platform::virtual_page_address::VirtualPageAddress;
use crate::syscall_handler::user_ptr::UserPtr;
use crate::syscall_handler::user_slice::UserSlice;
use crate::syscall_handler::{SyscallCommand, SyscallCommandHandler, SyscallHandler};
use crate::task_id::TaskId;
use alloc::boxed::Box;
use kernel_hal::tasks::TaskFrame;

pub struct SysProcMunmapCommand {
    task_frame: Box<TaskFrame>,
    proc_handle: u64,
    chunk: UserSlice<*const [u8]>,
}

impl SyscallCommand for SysProcMunmapCommand {
    type Error = SyscallError;

    fn parse(ctx: SyscallContext) -> Result<Self, (Box<TaskFrame>, Self::Error)> {
        let task_frame = ctx.task_frame;
        let proc_handle = ctx.args[0] as u64;
        let addr = ctx.args[1] as usize;
        let length = ctx.args[2] as usize;

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
            proc_handle,
            chunk,
        })
    }
}

impl SyscallCommandHandler<SysProcMunmapCommand> for SyscallHandler {
    type Ok = ();
    type Err = SyscallError;

    fn handle_command(
        &self,
        command: SysProcMunmapCommand,
    ) -> Result<SyscallIntent<Self::Ok>, (Box<TaskFrame>, Self::Err)> {
        let task_id = TaskId::get_current().expect("Scheduler is not started yet!");
        let task = self
            .task_registry
            .get(task_id)
            .expect("Current task should exist!");
        let target_vmm = if command.proc_handle == 0 {
            task.get_proc_handle(command.proc_handle)
                .expect("Not assigned proc handle not handled")
        } else {
            task.get_virtual_memory_manager()
        };
        const PAGE_MASK: usize = !(PAGE_FRAME_SIZE - 1);
        let addr: UserPtr<usize> = match UserPtr::try_from(command.chunk.addr() & PAGE_MASK) {
            Ok(addr) => addr,
            Err(err) => return Err((command.task_frame, err)),
        };
        let pages_count = (command.chunk.len() + PAGE_FRAME_SIZE - 1) / PAGE_FRAME_SIZE;

        for page_idx in 0..pages_count {
            let page_vaddr = match UserPtr::try_from(*addr + page_idx * PAGE_FRAME_SIZE) {
                Ok(page_vaddr) => page_vaddr,
                Err(err) => return Err((command.task_frame, err)),
            };
            let page_vaddr = VirtualPageAddress::new(*page_vaddr).unwrap();
            unsafe {
                target_vmm
                    .unmap_page(page_vaddr)
                    .expect("Unmapping failed!");
            }
            // @TODO: free the physical page frame if not used by any other process
        }

        Ok(SyscallIntent::Return(command.task_frame, ()))
    }
}

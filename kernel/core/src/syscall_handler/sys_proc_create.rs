use crate::platform::syscalls::{SyscallContext, SyscallError, SyscallIntent};
use crate::platform::virtual_memory_manager_context::VirtualMemoryManagerContext;
use crate::syscall_handler::{SyscallCommand, SyscallCommandHandler, SyscallHandler};
use crate::task_id::TaskId;
use alloc::boxed::Box;
use alloc::sync::Arc;
use kernel_hal::tasks::TaskFrame;

pub struct SysProcCreateCommand {
    task_frame: Box<TaskFrame>,
    #[allow(unused)]
    flags: u64,
}

impl SyscallCommand for SysProcCreateCommand {
    type Error = SyscallError;

    fn parse(ctx: SyscallContext) -> Result<Self, (Box<TaskFrame>, Self::Error)> {
        let task_frame = ctx.task_frame;
        let flags = ctx.args[0];

        Ok(Self { task_frame, flags })
    }
}

impl SyscallCommandHandler<SysProcCreateCommand> for SyscallHandler {
    type Ok = u64;
    type Err = SyscallError;

    fn handle_command(
        &self,
        command: SysProcCreateCommand,
    ) -> Result<SyscallIntent<Self::Ok>, (Box<TaskFrame>, Self::Err)> {
        let task_id = TaskId::get_current().expect("Scheduler is not started yet!");
        let mut task = self
            .task_registry
            .get(task_id)
            .expect("Current task should exist!");
        let vmm = Arc::new(unsafe { VirtualMemoryManagerContext::create() });
        let handle = task.add_proc_handle(vmm);
        Ok(SyscallIntent::Return(command.task_frame, handle))
    }
}

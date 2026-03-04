use crate::platform::syscalls::{SyscallContext, SyscallError, SyscallIntent};
use crate::syscall_handler::user_ptr::UserPtr;
use crate::syscall_handler::{SyscallCommand, SyscallCommandHandler, SyscallHandler};
use crate::task_id::TaskId;
use crate::task_registry::TaskSpec;

pub struct SysCloneCommand {
    // @TODO: implement flags
    #[allow(unused)]
    flags: u64,
    stack_pointer: UserPtr<usize>,
    entrypoint: UserPtr<usize>,
}

impl SyscallCommand for SysCloneCommand {
    type Error = SyscallError;

    fn parse<'a>(ctx: &SyscallContext<'a>) -> Result<Self, Self::Error>
    where
        Self: 'a,
    {
        let flags = ctx.args[0];
        let stack_pointer = UserPtr::try_from(ctx.args[1])?;
        let entrypoint = UserPtr::try_from(ctx.args[2])?;

        Ok(Self {
            flags,
            stack_pointer,
            entrypoint,
        })
    }
}

impl SyscallCommandHandler<SysCloneCommand> for SyscallHandler {
    type Ok = TaskId;
    type Err = SyscallError;

    fn handle_command(
        &self,
        command: SysCloneCommand,
    ) -> Result<SyscallIntent<Self::Ok>, Self::Err> {
        let current_task_id = TaskId::get_current().expect("Scheduler is not started yet!");
        let vmm = self
            .task_registry
            .get(current_task_id)
            .map(|task| task.get_virtual_memory_manager().clone())
            .expect("Current task should exist!");

        let pid = self.scheduler.spawn(TaskSpec::User {
            virtual_memory_manager_context: vmm,
            user_stack_vaddr: *command.stack_pointer,
            entrypoint_vaddr: *command.entrypoint,
        });
        Ok(SyscallIntent::Return(pid))
    }
}

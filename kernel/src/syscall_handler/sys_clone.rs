use crate::platform::syscalls::{SyscallContext, SyscallIntent};
use crate::syscall_handler::{SyscallCommand, SyscallCommandHandler, SyscallHandler};
use crate::task_registry::TaskSpec;

pub struct SysCloneCommand {
    // @TODO: implement flags
    #[allow(unused)]
    flags: u64,
    stack_pointer: usize,
    entrypoint: usize,
}

impl SyscallCommand for SysCloneCommand {
    fn parse<'a>(ctx: &SyscallContext<'a>) -> Option<Self>
    where
        Self: 'a
    {
        Some(Self {
            flags: ctx.args[0],
            stack_pointer: ctx.args[1] as usize,
            entrypoint: ctx.args[2] as usize,
        })
    }
}

impl SyscallCommandHandler<SysCloneCommand> for SyscallHandler {
    fn handle_command(&self, command: SysCloneCommand) -> SyscallIntent {
        let vmm = self
            .scheduler
            .access_current_task_context(|task| task.get_virtual_memory_manager().clone())
            .expect("Scheduler is not started yet!");

        if command.stack_pointer >= 0x800000000000 || command.entrypoint >= 0x800000000000 {
            return SyscallIntent::Return(0);
        }
        let pid = self.scheduler.spawn(TaskSpec::User {
            virtual_memory_manager_context: vmm,
            user_stack_vaddr: command.stack_pointer,
            entrypoint_vaddr: command.entrypoint,
        });
        SyscallIntent::Return(pid.get())
    }
}

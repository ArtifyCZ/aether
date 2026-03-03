use crate::platform::syscalls::{SyscallContext, SyscallError, SyscallIntent};
use crate::platform::tasks::TaskFrame;
use crate::println;
use crate::syscall_handler::{SyscallCommand, SyscallCommandHandler, SyscallHandler};

pub struct SysExitCommand {
    task_frame: TaskFrame,
}

impl SyscallCommand for SysExitCommand {
    type Error = SyscallError;

    fn parse<'a>(ctx: &SyscallContext<'a>) -> Result<Self, Self::Error>
    where
        Self: 'a,
    {
        Ok(Self {
            task_frame: *ctx.task_frame,
        })
    }
}

impl SyscallCommandHandler<SysExitCommand> for SyscallHandler {
    type Ok = ();
    type Err = SyscallError;

    fn handle_command(&self, command: SysExitCommand) -> Result<SyscallIntent<Self::Ok>, Self::Err> {
        println!("=== EXIT SYSCALL ===");
        let next_task_state = self
            .scheduler
            .exit_current_task(command.task_frame)
            .unwrap();

        Ok(SyscallIntent::SwitchTo(next_task_state))
    }
}

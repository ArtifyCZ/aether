use alloc::boxed::Box;
use kernel_hal::tasks::TaskFrame;
use crate::platform::syscalls::{SyscallContext, SyscallError, SyscallIntent};
use crate::println;
use crate::syscall_handler::{SyscallCommand, SyscallCommandHandler, SyscallHandler};

pub struct SysExitCommand {
    task_frame: Box<TaskFrame>,
}

impl SyscallCommand for SysExitCommand {
    type Error = SyscallError;

    fn parse(ctx: SyscallContext) -> Result<Self, (Box<TaskFrame>, Self::Error)>
    {
        Ok(Self {
            task_frame: ctx.task_frame,
        })
    }
}

impl SyscallCommandHandler<SysExitCommand> for SyscallHandler {
    type Ok = ();
    type Err = SyscallError;

    fn handle_command(&self, command: SysExitCommand) -> Result<SyscallIntent<Self::Ok>, (Box<TaskFrame>, Self::Err)> {
        println!("=== EXIT SYSCALL ===");
        let next_task_state = self
            .scheduler
            .exit_current_task(command.task_frame);

        Ok(SyscallIntent::SwitchTo(next_task_state))
    }
}

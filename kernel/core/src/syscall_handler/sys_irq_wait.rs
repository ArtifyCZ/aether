use alloc::boxed::Box;
use kernel_hal::tasks::TaskFrame;
use crate::platform::syscalls::{SyscallContext, SyscallError, SyscallIntent};
use crate::syscall_handler::{SyscallCommand, SyscallCommandHandler, SyscallHandler};
use crate::task_id::TaskId;

pub struct SysIrqWaitCommand {
    frame: Box<TaskFrame>,
    irq: u8,
}

impl SyscallCommand for SysIrqWaitCommand {
    type Error = SyscallError;

    fn parse(ctx: SyscallContext) -> Result<Self, (Box<TaskFrame>, Self::Error)>
    {
        let irq = ctx.args[0] as u8;
        let frame = ctx.task_frame;

        Ok(Self { irq, frame })
    }
}

impl SyscallCommandHandler<SysIrqWaitCommand> for SyscallHandler {
    type Ok = ();
    type Err = SyscallError;

    fn handle_command(
        &self,
        command: SysIrqWaitCommand,
    ) -> Result<SyscallIntent<Self::Ok>, (Box<TaskFrame>, Self::Err)> {
        {
            let mut task = self
                .task_registry
                .get(TaskId::get_current().unwrap())
                .unwrap();
            task.return_syscall_value(Ok(()));
        }

        let next_or_prev_frame = self.scheduler.wait_for_irq(command.irq, command.frame);

        Ok(SyscallIntent::SwitchTo(next_or_prev_frame))
    }
}

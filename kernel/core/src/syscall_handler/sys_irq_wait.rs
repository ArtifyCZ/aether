use crate::platform::syscalls::{SyscallContext, SyscallError, SyscallIntent};
use crate::platform::tasks::TaskFrame;
use crate::syscall_handler::{SyscallCommand, SyscallCommandHandler, SyscallHandler};
use crate::task_id::TaskId;

pub struct SysIrqWaitCommand {
    irq: u8,
    frame: TaskFrame,
}

impl SyscallCommand for SysIrqWaitCommand {
    type Error = SyscallError;

    fn parse<'a>(ctx: &SyscallContext<'a>) -> Result<Self, Self::Error>
    where
        Self: 'a,
    {
        let irq = ctx.args[0] as u8;
        let frame = ctx.task_frame.clone();

        Ok(Self { irq, frame })
    }
}

impl SyscallCommandHandler<SysIrqWaitCommand> for SyscallHandler {
    type Ok = ();
    type Err = SyscallError;

    fn handle_command(
        &self,
        command: SysIrqWaitCommand,
    ) -> Result<SyscallIntent<Self::Ok>, Self::Err> {
        if let Some(next_frame) = self.scheduler.wait_for_irq(command.irq, command.frame) {
            if next_frame == command.frame {
                Ok(SyscallIntent::Return(()))
            } else {
                let mut task = self
                    .task_registry
                    .get(TaskId::get_current().unwrap())
                    .unwrap();
                task.return_syscall_value(Ok(()));
                Ok(SyscallIntent::SwitchTo(next_frame))
            }
        } else {
            Ok(SyscallIntent::SwitchTo(command.frame))
        }
    }
}

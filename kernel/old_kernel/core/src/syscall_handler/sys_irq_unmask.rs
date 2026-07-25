use crate::platform::interrupts::Interrupts;
use crate::platform::syscalls::{SyscallContext, SyscallError, SyscallIntent};
use crate::syscall_handler::{SyscallCommand, SyscallCommandHandler, SyscallHandler};
use alloc::boxed::Box;
use kernel_hal::tasks::TaskFrame;

pub struct SysIrqUnmaskCommand {
    task_frame: Box<TaskFrame>,
    irq: u8,
}

impl SyscallCommand for SysIrqUnmaskCommand {
    type Error = SyscallError;

    fn parse(ctx: SyscallContext) -> Result<Self, (Box<TaskFrame>, Self::Error)> {
        let task_frame = ctx.task_frame;
        let irq = ctx.args[0] as u8;
        Ok(Self { task_frame, irq })
    }
}

impl SyscallCommandHandler<SysIrqUnmaskCommand> for SyscallHandler {
    type Ok = ();
    type Err = SyscallError;

    fn handle_command(
        &self,
        command: SysIrqUnmaskCommand,
    ) -> Result<SyscallIntent<Self::Ok>, (Box<TaskFrame>, Self::Err)> {
        unsafe {
            Interrupts::unmask_irq(command.irq);
        }

        Ok(SyscallIntent::Return(command.task_frame, ()))
    }
}

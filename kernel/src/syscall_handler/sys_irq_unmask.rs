use crate::platform::interrupts::Interrupts;
use crate::platform::syscalls::{SyscallContext, SyscallError, SyscallIntent};
use crate::syscall_handler::{SyscallCommand, SyscallCommandHandler, SyscallHandler};

pub struct SysIrqUnmaskCommand {
    irq: u8,
}

impl SyscallCommand for SysIrqUnmaskCommand {
    type Error = SyscallError;

    fn parse<'a>(ctx: &SyscallContext<'a>) -> Result<Self, Self::Error>
    where
        Self: 'a,
    {
        let irq = ctx.args[0] as u8;
        Ok(Self { irq })
    }
}

impl SyscallCommandHandler<SysIrqUnmaskCommand> for SyscallHandler {
    type Ok = ();
    type Err = SyscallError;

    fn handle_command(&self, command: SysIrqUnmaskCommand) -> Result<SyscallIntent<Self::Ok>, Self::Err> {
        unsafe {
            Interrupts::unmask_irq(command.irq);
        }

        Ok(SyscallIntent::Return(()))
    }
}

use crate::platform::syscalls::{SyscallContext, SyscallError, SyscallIntent};
use crate::platform::terminal::Terminal;
use crate::syscall_handler::user_ptr::UserPtr;
use crate::syscall_handler::user_slice::UserSlice;
use crate::syscall_handler::{SyscallCommand, SyscallCommandHandler, SyscallHandler};

pub struct SysWriteCommand {
    fd: i32,
    buf: UserSlice<*const [u8]>,
}

impl SyscallCommand for SysWriteCommand {
    type Error = SyscallError;

    fn parse<'a>(ctx: &SyscallContext<'a>) -> Result<Self, Self::Error>
    where
        Self: 'a,
    {
        let fd = ctx.args[0] as i32;
        let buf = UserPtr::try_from(ctx.args[1])?;
        let count = ctx.args[2] as usize;
        let buf = UserSlice::try_from((buf, count))?;
        Ok(Self { fd, buf })
    }
}

impl SyscallCommandHandler<SysWriteCommand> for SyscallHandler {
    type Ok = ();
    type Err = SyscallError;

    fn handle_command(
        &self,
        command: SysWriteCommand,
    ) -> Result<SyscallIntent<Self::Ok>, Self::Err> {
        // @TODO: add defensive checks (e.g. is the buffer in its full size mapped to the address space?
        // @TODO: use new-type and other patterns
        // stdout or stderr
        if command.fd != 1 && command.fd != 2 {
            // EBADF: Bad File Descriptor
            return Err(SyscallError::SYS_EBADF);
        }

        unsafe {
            Terminal::print_bytes(command.buf.as_slice());
        }

        Ok(SyscallIntent::Return(()))
    }
}

use crate::platform::syscalls::{SyscallContext, SyscallError, SyscallIntent};
use crate::platform::terminal::Terminal;
use crate::syscall_handler::user_ptr::UserPtr;
use crate::syscall_handler::user_slice::UserSlice;
use crate::syscall_handler::{SyscallCommand, SyscallCommandHandler, SyscallHandler};
use alloc::boxed::Box;
use kernel_hal::tasks::TaskFrame;

pub struct SysWriteCommand {
    task_frame: Box<TaskFrame>,
    fd: i32,
    buf: UserSlice<*const [u8]>,
}

impl SyscallCommand for SysWriteCommand {
    type Error = SyscallError;

    fn parse(ctx: SyscallContext) -> Result<Self, (Box<TaskFrame>, Self::Error)> {
        let task_frame = ctx.task_frame;
        let fd = ctx.args[0] as i32;
        let buf = match UserPtr::try_from(ctx.args[1]) {
            Ok(buf) => buf,
            Err(err) => return Err((task_frame, err)),
        };
        let count = ctx.args[2] as usize;
        let buf = match UserSlice::try_from((buf, count)) {
            Ok(buf) => buf,
            Err(err) => return Err((task_frame, err)),
        };
        Ok(Self {
            task_frame,
            fd,
            buf,
        })
    }
}

impl SyscallCommandHandler<SysWriteCommand> for SyscallHandler {
    type Ok = ();
    type Err = SyscallError;

    fn handle_command(
        &self,
        command: SysWriteCommand,
    ) -> Result<SyscallIntent<Self::Ok>, (Box<TaskFrame>, Self::Err)> {
        // @TODO: add defensive checks (e.g. is the buffer in its full size mapped to the address space?
        // @TODO: use new-type and other patterns
        // stdout or stderr
        if command.fd != 1 && command.fd != 2 {
            // EBADF: Bad File Descriptor
            return Err((command.task_frame, SyscallError::SYS_EBADF));
        }

        unsafe {
            Terminal::print_bytes(command.buf.as_slice());
        }

        Ok(SyscallIntent::Return(command.task_frame, ()))
    }
}

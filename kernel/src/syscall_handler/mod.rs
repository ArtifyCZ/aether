mod sys_clone;
mod sys_exit;
mod sys_mmap;
mod sys_write;
mod user_ptr;
mod user_slice;
mod sys_irq_wait;
mod sys_irq_unmask;
mod sys_mmap_dev;

use crate::platform::syscalls::{
    SyscallContext, SyscallError, SyscallIntent, SyscallReturnValue, SyscallReturnable, syscall_num,
};
use crate::scheduler::Scheduler;
use crate::syscall_handler::sys_clone::SysCloneCommand;
use crate::syscall_handler::sys_exit::SysExitCommand;
use crate::syscall_handler::sys_mmap::SysMmapCommand;
use crate::syscall_handler::sys_write::SysWriteCommand;
use crate::task_registry::TaskRegistry;
use alloc::boxed::Box;
use crate::syscall_handler::sys_irq_unmask::SysIrqUnmaskCommand;
use crate::syscall_handler::sys_irq_wait::SysIrqWaitCommand;
use crate::syscall_handler::sys_mmap_dev::SysMmapDevCommand;

macro_rules! define_syscall_request {
    ($name:ident, { $(
            $syscall_num:expr => $syscall_name:ident : $syscall_command:ty,
        )* } $(,)?
    ) => {
        #[repr(u64)]
        enum $name {
            $(
                $syscall_name ($syscall_command) = $syscall_num,
            )*
        }

        impl SyscallCommand for $name {
            type Error = SyscallError;

            fn parse<'a>(ctx: &SyscallContext<'a>) -> Result<Self, Self::Error> where Self: 'a {
                match ctx.num {
                    $(
                        num if num == $syscall_num => {
                            let command: $syscall_command = SyscallCommand::parse(ctx)?;
                            Ok($name::$syscall_name(command))
                        },
                    )*
                    _ => Err(SyscallError::SYS_ENOSYS),
                }
            }
        }

        impl SyscallHandler {
            fn handle_command(&self, command: $name) -> Result<SyscallIntent<SyscallReturnValue>, SyscallError> {
                let result = match command {
                    $(
                        $name::$syscall_name(command) => SyscallCommandHandler::< $syscall_command >::handle_command(self, command)?.into(),
                    )*
                };
                Ok(result)
            }
        }
    };
}

define_syscall_request!(
    SyscallRequest,
    {
        syscall_num::SYS_EXIT => Exit: SysExitCommand,
        syscall_num::SYS_WRITE => Write: SysWriteCommand,
        syscall_num::SYS_CLONE => Clone: SysCloneCommand,
        syscall_num::SYS_MMAP => Mmap: SysMmapCommand,
        syscall_num::SYS_IRQ_WAIT => IrqWait: SysIrqWaitCommand,
        syscall_num::SYS_IRQ_UNMASK => IrqUnmask: SysIrqUnmaskCommand,
        syscall_num::SYS_MMAP_DEV => MmapDev: SysMmapDevCommand,
    },
);

pub trait SyscallCommand: Sized {
    type Error: Into<SyscallError>;

    fn parse<'a>(ctx: &SyscallContext<'a>) -> Result<Self, Self::Error>
    where
        Self: 'a;
}

pub trait SyscallCommandHandler<TSyscallCommand> {
    type Ok: SyscallReturnable;
    type Err: Into<SyscallError>;

    fn handle_command(
        &self,
        command: TSyscallCommand,
    ) -> Result<SyscallIntent<Self::Ok>, Self::Err>;
}

pub struct SyscallHandler {
    scheduler: &'static Scheduler,
    task_registry: &'static TaskRegistry,
}

impl SyscallHandler {
    pub fn init(
        scheduler: &'static Scheduler,
        task_registry: &'static TaskRegistry,
    ) -> &'static Self {
        let syscall_handler: &'static Self = Box::leak(Box::new(SyscallHandler {
            scheduler,
            task_registry,
        }));
        syscall_handler
    }

    pub fn handle(
        &self,
        ctx: &SyscallContext<'_>,
    ) -> Result<SyscallIntent<SyscallReturnValue>, SyscallError> {
        let request = SyscallRequest::parse(ctx)?;
        self.handle_command(request)
    }
}

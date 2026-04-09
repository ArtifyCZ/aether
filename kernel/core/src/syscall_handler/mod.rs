mod sys_exit;
mod sys_irq_unmask;
mod sys_irq_wait;
mod sys_mmap_dev;
mod sys_proc_create;
mod sys_proc_mmap;
mod sys_proc_mprot;
mod sys_proc_munmap;
mod sys_proc_spawn;
mod sys_write;
mod user_ptr;
mod user_slice;

use crate::platform::syscalls::{
    SyscallContext, SyscallError, SyscallIntent, SyscallReturnValue, SyscallReturnable,
};
use crate::scheduler::Scheduler;
use crate::syscall_handler::sys_exit::SysExitCommand;
use crate::syscall_handler::sys_irq_unmask::SysIrqUnmaskCommand;
use crate::syscall_handler::sys_irq_wait::SysIrqWaitCommand;
use crate::syscall_handler::sys_mmap_dev::SysMmapDevCommand;
use crate::syscall_handler::sys_proc_create::SysProcCreateCommand;
use crate::syscall_handler::sys_proc_mmap::SysProcMmapCommand;
use crate::syscall_handler::sys_proc_mprot::SysProcMprotCommand;
use crate::syscall_handler::sys_proc_munmap::SysProcMunmapCommand;
use crate::syscall_handler::sys_proc_spawn::SysProcSpawnCommand;
use crate::syscall_handler::sys_write::SysWriteCommand;
use crate::task_registry::TaskRegistry;
use alloc::boxed::Box;
use kernel_hal::tasks::TaskFrame;
use syscalls_rust::SyscallNumber;

macro_rules! define_syscall_request {
    ($name:ident, { $(
            $syscall_num:expr => $syscall_name:ident : $syscall_command:ty,
        )* } $(,)?
    ) => {
        #[repr(u64)]
        enum $name {
            $(
                $syscall_name ($syscall_command) = $syscall_num as u64,
            )*
        }

        impl SyscallCommand for $name {
            type Error = SyscallError;

            fn parse(ctx: SyscallContext) -> Result<Self, (Box<TaskFrame>, Self::Error)> {
                match ctx.num {
                    $(
                        num if num == ($syscall_num as u64) => {
                            let command: $syscall_command = SyscallCommand::parse(ctx)?;
                            Ok($name::$syscall_name(command))
                        },
                    )*
                    _ => Err((ctx.task_frame, SyscallError::Enosys)),
                }
            }
        }

        impl SyscallHandler {
            fn handle_command(&self, command: $name) -> Result<SyscallIntent<SyscallReturnValue>, (Box<TaskFrame>, SyscallError)> {
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
        SyscallNumber::Exit => Exit: SysExitCommand,
        SyscallNumber::Write => Write: SysWriteCommand,
        SyscallNumber::IrqWait => IrqWait: SysIrqWaitCommand,
        SyscallNumber::IrqUnmask => IrqUnmask: SysIrqUnmaskCommand,
        SyscallNumber::MmapDev => MmapDev: SysMmapDevCommand,
        SyscallNumber::ProcCreate => ProcCreate: SysProcCreateCommand,
        SyscallNumber::ProcMmap => ProcMmap: SysProcMmapCommand,
        SyscallNumber::ProcMprot => ProcMprot: SysProcMprotCommand,
        SyscallNumber::ProcMunmap => ProcMunmap: SysProcMunmapCommand,
        SyscallNumber::ProcSpawn => ProcSpawn: SysProcSpawnCommand,
    },
);

pub trait SyscallCommand: Sized {
    type Error: Into<SyscallError>;

    fn parse(ctx: SyscallContext) -> Result<Self, (Box<TaskFrame>, Self::Error)>;
}

pub trait SyscallCommandHandler<TSyscallCommand> {
    type Ok: SyscallReturnable;
    type Err: Into<SyscallError>;

    fn handle_command(
        &self,
        command: TSyscallCommand,
    ) -> Result<SyscallIntent<Self::Ok>, (Box<TaskFrame>, Self::Err)>;
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
        ctx: SyscallContext,
    ) -> Result<SyscallIntent<SyscallReturnValue>, (Box<TaskFrame>, SyscallError)> {
        let request = SyscallRequest::parse(ctx)?;
        self.handle_command(request)
    }
}

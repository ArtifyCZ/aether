mod sys_exit;
mod sys_write;
mod sys_clone;
mod sys_mmap;

use crate::platform::syscalls::{syscall_num, SyscallContext, SyscallIntent};
use crate::scheduler::Scheduler;
use crate::syscall_handler::sys_clone::SysCloneCommand;
use crate::syscall_handler::sys_exit::SysExitCommand;
use crate::syscall_handler::sys_mmap::SysMmapCommand;
use crate::syscall_handler::sys_write::SysWriteCommand;
use alloc::boxed::Box;

pub struct SyscallHandler {
    scheduler: &'static Scheduler,
}

pub trait SyscallCommand: Sized {
    fn parse<'a>(ctx: &SyscallContext<'a>) -> Option<Self>
    where
        Self: 'a;
}

pub trait SyscallCommandHandler<TSyscallCommand> {
    fn handle_command(&self, command: TSyscallCommand) -> SyscallIntent;
}

impl SyscallHandler {
    pub fn init(scheduler: &'static Scheduler) -> &'static Self {
        let syscall_handler: &'static Self = Box::leak(Box::new(SyscallHandler { scheduler }));
        syscall_handler
    }

    pub fn handle(&self, ctx: &SyscallContext<'_>) -> SyscallIntent {
        match ctx.num {
            syscall_num::SYS_EXIT => self.handle_command(SysExitCommand::parse(ctx).unwrap()),
            syscall_num::SYS_WRITE => self.handle_command(SysWriteCommand::parse(ctx).unwrap()),
            syscall_num::SYS_CLONE => self.handle_command(SysCloneCommand::parse(ctx).unwrap()),
            syscall_num::SYS_MMAP => self.handle_command(SysMmapCommand::parse(ctx).unwrap()),
            _ => panic!("Non-existent syscall triggered!"), // @TODO: add better handling
        }
    }
}

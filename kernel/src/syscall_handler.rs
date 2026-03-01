use crate::platform::drivers::serial::SerialDriver;
use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::physical_memory_manager::PhysicalMemoryManager;
use crate::platform::syscalls::SyscallContext;
use crate::platform::terminal::Terminal;
use crate::platform::virtual_memory_manager_context::VirtualMemoryMappingFlags;
use crate::platform::virtual_page_address::VirtualPageAddress;
use crate::scheduler::Scheduler;
use crate::task_registry::TaskSpec;
use alloc::boxed::Box;
use alloc::format;
use core::ptr::slice_from_raw_parts;

pub struct SyscallHandler {
    scheduler: &'static Scheduler,
}

impl SyscallHandler {
    pub fn init(scheduler: &'static Scheduler) -> &'static Self {
        let syscall_handler: &'static Self = Box::leak(Box::new(SyscallHandler { scheduler }));
        syscall_handler
    }

    pub fn handle(&self, ctx: &mut SyscallContext<'_>) {
        match ctx.num {
            0x00 => self.sys_exit(ctx),
            0x01 => self.sys_write(ctx),
            0x02 => self.sys_clone(ctx),
            0x03 => self.sys_mmap(ctx),
            _ => panic!("Non-existent syscall triggered!"), // @TODO: add better handling
        }
    }

    fn sys_exit(&self, ctx: &mut SyscallContext<'_>) {
        unsafe {
            SerialDriver::println("=== EXIT SYSCALL ===");
        }
        let prev_task_state = ctx.task_frame.clone();
        let next_task_state = self.scheduler.exit_current_task(prev_task_state).unwrap();

        *ctx.task_frame = next_task_state;
    }

    fn sys_write(&self, ctx: &mut SyscallContext<'_>) {
        let fd = ctx.args[0];
        let user_buf = ctx.args[1];
        let count = ctx.args[2];

        // stdout or stderr
        if fd != 1 && fd != 2 {
            unsafe {
                // EBADF: Bad File Descriptor
                self.scheduler
                    .update_current_task_context(|task| task.set_syscall_return_value(1));
            }
            return;
        }

        // Basic Range Check: Is the buffer in User Space?
        // On x86_64, user addresses are usually < 0x00007FFFFFFFFFFF
        if user_buf >= 0x800000000000 || (user_buf + count) >= 0x800000000000 {
            unsafe {
                // EFAULT: Bad Address
                self.scheduler
                    .update_current_task_context(|task| task.set_syscall_return_value(1));
            }
            return;
        }

        let user_buf = user_buf as *const u8;
        unsafe {
            let user_buf = slice_from_raw_parts(user_buf, count as usize)
                .as_ref()
                .unwrap();
            SerialDriver::write(user_buf);
            Terminal::print_bytes(user_buf);
        }

        unsafe {
            self.scheduler
                .update_current_task_context(|task| task.set_syscall_return_value(0));
        }
    }

    fn sys_clone(&self, ctx: &mut SyscallContext<'_>) {
        let vmm = self
            .scheduler
            .access_current_task_context(|task| task.get_virtual_memory_manager().clone())
            .expect("Scheduler is not started yet!");
        // @TODO: implement flags
        let _flags = ctx.args[0];
        let stack_pointer = ctx.args[1] as usize;
        let entrypoint = ctx.args[2] as usize;
        if stack_pointer >= 0x800000000000 || entrypoint >= 0x800000000000 {
            unsafe {
                self.scheduler
                    .update_current_task_context(|task| task.set_syscall_return_value(0));
            }
            return;
        }
        let pid = self.scheduler.spawn(TaskSpec::User {
            virtual_memory_manager_context: vmm,
            user_stack_vaddr: stack_pointer,
            entrypoint_vaddr: entrypoint,
        });
        unsafe {
            self.scheduler
                .update_current_task_context(|task| task.set_syscall_return_value(pid.get()));
        }
    }

    fn sys_mmap(&self, ctx: &mut SyscallContext<'_>) {
        let addr = ctx.args[0] as usize;
        let length = ctx.args[1] as usize;
        let _prot = ctx.args[2] as u32;
        let _flags = ctx.args[3] as u32;

        if addr >= 0x800000000000 || (addr + length) >= 0x800000000000 {
            unsafe {
                SerialDriver::println("mmap: EFAULT: Bad Address");
                SerialDriver::println(&format!("addr: {}; len: {}", addr, length));
                self.scheduler
                    .update_current_task_context(|task| task.set_syscall_return_value(0));
            }
            return;
        }

        const PAGE_MASK: usize = !(PAGE_FRAME_SIZE - 1);
        let addr = addr & PAGE_MASK;
        let pages_count = (length + PAGE_FRAME_SIZE - 1) / PAGE_FRAME_SIZE;
        // @TODO: implement protection flags
        // @TODO: implement flags

        for page_idx in 0..pages_count {
            let page_vaddr = VirtualPageAddress::new(addr + page_idx * PAGE_FRAME_SIZE).unwrap();
            let phys = unsafe { PhysicalMemoryManager::alloc_frame() }.unwrap();
            unsafe {
                self.scheduler.access_current_task_context(|task| {
                    task.get_virtual_memory_manager().map_page(
                        page_vaddr,
                        phys,
                        VirtualMemoryMappingFlags::PRESENT
                            | VirtualMemoryMappingFlags::USER
                            | VirtualMemoryMappingFlags::WRITE,
                    )
                });
            }
        }

        unsafe {
            self.scheduler
                .update_current_task_context(|task| task.set_syscall_return_value(addr as u64));
        }
    }
}

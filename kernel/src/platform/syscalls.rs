use crate::interrupt_safe_spin_lock::InterruptSafeSpinLock;
use crate::platform::drivers::serial::SerialDriver;
use crate::platform::syscalls::bindings::syscall_frame;
use crate::platform::tasks::{TaskContext, TaskFrame};
use crate::platform::terminal::Terminal;
use crate::scheduler::Scheduler;
use alloc::format;
use core::ffi::c_void;
use core::ptr::slice_from_raw_parts;

mod bindings {
    include_bindings!("syscalls.rs");
}

use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::physical_memory_manager::PhysicalMemoryManager;
use crate::platform::virtual_memory_manager_context::VirtualMemoryMappingFlags;
use crate::platform::virtual_page_address::VirtualPageAddress;
pub use bindings::syscall_args;

pub struct Syscalls;

macro_rules! zeroed_array {
    ($size:expr) => {
        [0; $size]
    };
    (@accum $array:ident, 0, $item:expr) => {
        {
            $array[0] = $item;
        }
    };
    (@accum $array:ident, $size:expr, $idx:expr) => {
        {
        }
    };
    (@accum $array:ident, $size:expr, $idx:expr, $cur_item:expr $(, $item:expr)*) => {
        {
            $array[$idx] = $cur_item;
            zeroed_array!(@accum $array, $size, ($idx + 1) $(, $item)*);
        }
    };
    ($size:expr $(, $item:expr)*) => {
        {
            let mut array = zeroed_array!($size);
            zeroed_array!(@accum array, $size, 0 $(, $item)*);
            array
        }
    }
}

macro_rules! wrap_syscall {
    ($name:ident, $num:expr $(, $param_name:ident: $param_type:ty)* $(,)?) => {
        pub unsafe fn $name($($param_name: $param_type,)*) -> u64 {
            let args = syscall_args {
                num: $num,
                a: zeroed_array!(5 $(, ($param_name as u64))*),
            };
            unsafe { Syscalls::invoke(args) }
        }
    };
}

wrap_syscall!(sys_exit, 0x00,);
wrap_syscall!(sys_write, 0x01, fd: i32, user_buf: u64, count: usize);
wrap_syscall!(sys_clone, 0x02, flags: u64, stack_pointer: usize, entrypoint: usize);
wrap_syscall!(sys_mmap, 0x03, addr: usize, length: usize, prot: u32, flags: u32);

impl Syscalls {
    pub unsafe fn init(scheduler: &'static Scheduler) {
        unsafe {
            SerialDriver::println("Initializing syscalls...");
            bindings::syscalls_init(
                Some(Self::syscalls_dispatch),
                scheduler as *const _ as *mut _,
            );
            SerialDriver::println("Syscalls initialized!");
        }
    }

    pub unsafe fn invoke(args: syscall_args) -> u64 {
        unsafe { bindings::syscalls_raw(args) }
    }

    fn sys_exit(frame: &mut syscall_frame, scheduler: &Scheduler) {
        unsafe {
            SerialDriver::println("=== EXIT SYSCALL ===");
            let prev_task_interrupt_frame = (*frame.interrupt_frame).cast();

            let prev_task_state = TaskFrame(prev_task_interrupt_frame);
            let next_task_state = scheduler
                .exit_current_task(
                    |prev_task| {
                        prev_task.set_state(prev_task_state);
                    },
                    |next_task| {
                        next_task.prepare_switch();
                        next_task.get_state()
                    },
                )
                .unwrap();

            *frame.interrupt_frame = next_task_state.0.cast();
        }
    }

    fn sys_write(frame: &mut bindings::syscall_frame, scheduler: &Scheduler) {
        let fd = frame.a[0];
        let user_buf = frame.a[1];
        let count = frame.a[2];

        // stdout or stderr
        if fd != 1 && fd != 2 {
            unsafe {
                // EBADF: Bad File Descriptor
                scheduler.update_current_task_context(|task| task.set_syscall_return_value(1));
            }
            return;
        }

        // Basic Range Check: Is the buffer in User Space?
        // On x86_64, user addresses are usually < 0x00007FFFFFFFFFFF
        if user_buf >= 0x800000000000 || (user_buf + count) >= 0x800000000000 {
            unsafe {
                // EFAULT: Bad Address
                scheduler.update_current_task_context(|task| task.set_syscall_return_value(1));
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
            scheduler.update_current_task_context(|task| task.set_syscall_return_value(0));
        }
    }

    fn sys_clone(frame: &mut bindings::syscall_frame, scheduler: &Scheduler) {
        let vmm = scheduler
            .access_current_task_context(|task| task.get_virtual_memory_manager().clone())
            .expect("Scheduler is not started yet!");
        // @TODO: implement flags
        let _flags = frame.a[0];
        let stack_pointer = frame.a[1] as usize;
        let entrypoint = frame.a[2] as usize;
        if stack_pointer >= 0x800000000000 || entrypoint >= 0x800000000000 {
            unsafe {
                scheduler.update_current_task_context(|task| task.set_syscall_return_value(1));
            }
            return;
        }
        let new_task = TaskContext::new_user(vmm, stack_pointer, entrypoint);
        scheduler.add(new_task);
        unsafe {
            scheduler.update_current_task_context(|task| task.set_syscall_return_value(0));
        }
    }

    fn sys_mmap(frame: &mut syscall_frame, scheduler: &Scheduler) {
        let addr = frame.a[0] as usize;
        let length = frame.a[1] as usize;
        let _prot = frame.a[2] as u32;
        let _flags = frame.a[3] as u32;

        if addr >= 0x800000000000 || (addr + length) >= 0x800000000000 {
            unsafe {
                SerialDriver::println("mmap: EFAULT: Bad Address");
                SerialDriver::println(&format!("addr: {}; len: {}", addr, length));
                scheduler.update_current_task_context(|task| task.set_syscall_return_value(0));
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
                scheduler.access_current_task_context(|task| {
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
            scheduler.update_current_task_context(|task| task.set_syscall_return_value(addr as u64));
        }
    }

    unsafe extern "C" fn syscalls_dispatch(
        frame: *mut bindings::syscall_frame,
        scheduler: *mut c_void,
    ) {
        let frame = unsafe { frame.as_mut() }.unwrap();
        let scheduler: &'static Scheduler = unsafe { &*scheduler.cast() };
        match frame.num {
            0x00 => Self::sys_exit(frame, scheduler),
            0x01 => Self::sys_write(frame, scheduler),
            0x02 => Self::sys_clone(frame, scheduler),
            0x03 => Self::sys_mmap(frame, scheduler),
            _ => panic!("Non-existent syscall triggered!"), // @TODO: add better handling
        };
    }
}

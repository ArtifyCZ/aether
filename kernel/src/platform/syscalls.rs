use crate::platform::drivers::serial::SerialDriver;
use crate::platform::scheduler::Scheduler;
use crate::platform::syscalls::bindings::syscall_frame;
use crate::platform::terminal::Terminal;
use core::ffi::c_void;
use core::ptr::{null_mut, slice_from_raw_parts};
use crate::interrupt_safe_spin_lock::InterruptSafeSpinLock;
use crate::platform::tasks::TaskState;

mod bindings {
    include_bindings!("syscalls.rs");
}

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

impl Syscalls {
    pub unsafe fn init(scheduler: &'static InterruptSafeSpinLock<Scheduler>) {
        unsafe {
            SerialDriver::println("Initializing syscalls...");
            bindings::syscalls_init(Some(Self::syscalls_dispatch), scheduler as *const _ as *mut _);
            SerialDriver::println("Syscalls initialized!");
        }
    }

    pub unsafe fn invoke(args: syscall_args) -> u64 {
        unsafe {
            bindings::syscalls_raw(args)
        }
    }

    fn sys_exit(frame: &mut syscall_frame, scheduler: &InterruptSafeSpinLock<Scheduler>) -> u64 {
        unsafe {
            SerialDriver::println("=== EXIT SYSCALL ===");
            let prev_task_interrupt_frame = (*frame.interrupt_frame).cast();
            let mut scheduler = scheduler.lock();

            let prev_task_state = TaskState(prev_task_interrupt_frame);
            if let Some(prev_task) = scheduler.get_current_task_mut() {
                prev_task.set_state(prev_task_state);
            }
            scheduler.exit_current_task();

            if let Some(next_task) = scheduler.pick_next() {
                next_task.prepare_switch();
                let next_task_interrupt_frame = next_task.get_state().0;
                *frame.interrupt_frame = next_task_interrupt_frame.cast();
            }

            0
        }
    }

    fn sys_write(frame: &mut bindings::syscall_frame) -> u64 {
        let fd = frame.a[0];
        let user_buf = frame.a[1];
        let count = frame.a[2];

        // stdout or stderr
        if fd != 1 && fd != 2 {
            return 1; // EBADF: Bad File Descriptor
        }

        // Basic Range Check: Is the buffer in User Space?
        // On x86_64, user addresses are usually < 0x00007FFFFFFFFFFF
        if user_buf >= 0x800000000000 || (user_buf + count) >= 0x800000000000 {
            return 1; // EFAULT: Bad Address
        }

        let user_buf = user_buf as *const u8;
        unsafe {
            let user_buf = slice_from_raw_parts(user_buf, count as usize)
                .as_ref()
                .unwrap();
            SerialDriver::write(user_buf);
            Terminal::print_bytes(user_buf);
        }
        0
    }

    unsafe extern "C" fn syscalls_dispatch(
        frame: *mut bindings::syscall_frame,
        scheduler: *mut c_void,
    ) -> u64 {
        let frame = unsafe { frame.as_mut() }.unwrap();
        let scheduler: &'static InterruptSafeSpinLock<Scheduler> = unsafe { &*scheduler.cast() };
        match frame.num {
            0x00 => Self::sys_exit(frame, scheduler),
            0x01 => Self::sys_write(frame),
            _ => panic!("Non-existent syscall triggered!"), // @TODO: add better handling
        }
    }
}

use crate::platform::drivers::serial::SerialDriver;
use crate::platform::syscalls::bindings::syscall_frame;
use crate::platform::tasks::TaskFrame;
use alloc::boxed::Box;
use core::ffi::c_void;

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
wrap_syscall!(sys_clone, 0x02, flags: u64, stack_pointer: usize, entrypoint: usize);
wrap_syscall!(sys_mmap, 0x03, addr: usize, length: usize, prot: u32, flags: u32);

pub struct SyscallContext<'a> {
    pub task_frame: &'a mut TaskFrame,
    pub args: [u64; 5],
    pub num: u64,
}

impl Syscalls {
    pub unsafe fn init<F>(f: F)
    where
        F: (FnMut(&mut SyscallContext<'_>)) + 'static,
    {
        unsafe extern "C" fn trampoline<F>(syscall_frame: *mut syscall_frame, arg: *mut c_void)
        where
            F: (FnMut(&mut SyscallContext<'_>)) + 'static,
        {
            let f: *mut F = arg.cast();
            let f: &mut F = unsafe { &mut *f };
            let syscall_frame: &mut syscall_frame = unsafe { &mut *syscall_frame };
            let mut task_frame = TaskFrame(unsafe { syscall_frame.interrupt_frame.read() }.cast());
            let mut context = SyscallContext {
                task_frame: &mut task_frame,
                args: syscall_frame.a,
                num: syscall_frame.num,
            };
            f(&mut context);
            unsafe {
                syscall_frame.interrupt_frame.write(task_frame.0.cast());
            }
        }

        unsafe {
            SerialDriver::println("Initializing syscalls...");
            bindings::syscalls_init(
                Some(trampoline::<F>),
                Box::into_raw(Box::new(f)) as *mut c_void,
            );
            SerialDriver::println("Syscalls initialized!");
        }
    }

    unsafe fn invoke(args: syscall_args) -> u64 {
        unsafe { bindings::syscalls_raw(args) }
    }
}

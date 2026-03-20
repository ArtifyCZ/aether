use crate::println;
use alloc::boxed::Box;
pub use kernel_bindings_gen::syscall_args;
use kernel_bindings_gen::syscalls_raw;
use kernel_hal::syscalls;
use kernel_hal::tasks::TaskFrame;
pub use syscalls_rust::syscall_err as SyscallError;
pub use syscalls_rust::syscall_num;

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

wrap_syscall!(sys_exit, syscall_num::SYS_EXIT);
wrap_syscall!(sys_write, syscall_num::SYS_WRITE, fd: i32, user_buf: u64, count: usize);

pub struct SyscallContext {
    pub task_frame: Box<TaskFrame>,
    pub args: [u64; 5],
    pub num: u64,
}

pub enum SyscallIntent<TOut> {
    /// Returns to the caller
    Return(Box<TaskFrame>, TOut),
    /// Switches to the specified task
    SwitchTo(Box<TaskFrame>),
}

impl<T> From<SyscallIntent<T>> for SyscallIntent<SyscallReturnValue>
where
    T: SyscallReturnable,
{
    fn from(value: SyscallIntent<T>) -> SyscallIntent<SyscallReturnValue> {
        match value {
            SyscallIntent::Return(task_frame, value) => {
                SyscallIntent::Return(task_frame, value.into_return_value())
            }
            SyscallIntent::SwitchTo(frame) => SyscallIntent::SwitchTo(frame),
        }
    }
}

#[derive(Debug)]
pub struct SyscallReturnValue(pub u64);

pub trait SyscallReturnable {
    fn into_return_value(self) -> SyscallReturnValue;
}

impl SyscallReturnable for () {
    fn into_return_value(self) -> SyscallReturnValue {
        SyscallReturnValue(0)
    }
}

/// @TODO: Make u64 not returnable (should use the newtype pattern instead)
impl SyscallReturnable for u64 {
    fn into_return_value(self) -> SyscallReturnValue {
        SyscallReturnValue(self)
    }
}

impl Syscalls {
    pub unsafe fn init<F>(mut f: F)
    where
        F: (FnMut(SyscallContext) -> Result<SyscallIntent<SyscallReturnValue>, (Box<TaskFrame>, SyscallError)>)
            + 'static,
    {
        println!("Initializing syscalls...");
        unsafe {
            syscalls::init(move |syscall_frame| {
                let num = syscall_frame.number();
                let args = syscall_frame.args();
                let task_frame = syscall_frame.into_task_frame();
                let context = SyscallContext {
                    task_frame,
                    args,
                    num,
                };
                let intent = match f(context) {
                    Ok(intent) => match intent {
                        SyscallIntent::Return(task_frame, value) => SyscallIntent::Return(task_frame, Ok(value)),
                        SyscallIntent::SwitchTo(frame) => SyscallIntent::SwitchTo(frame),
                    },
                    Err((task_frame, error)) => SyscallIntent::Return(task_frame, Err(error)),
                };
                match intent {
                    SyscallIntent::Return(mut task_frame, value) => unsafe {
                        task_frame.set_syscall_return_value(value.map(|value| value.0).map_err(|err_code| err_code as u64));
                        task_frame
                    },
                    SyscallIntent::SwitchTo(task_frame) => task_frame,
                }
            });
        }
        println!("Syscalls initialized!");
    }

    unsafe fn invoke(args: syscall_args) -> u64 {
        unsafe { syscalls_raw(args) }
    }
}

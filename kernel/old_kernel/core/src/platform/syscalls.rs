use crate::println;
use alloc::boxed::Box;
use kernel_hal::syscalls;
use kernel_hal::tasks::TaskFrame;
pub use syscalls_rust::SyscallError;
pub use syscalls_rust::SyscallNumber;

pub struct Syscalls;

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
        F: (FnMut(
                SyscallContext,
            )
                -> Result<SyscallIntent<SyscallReturnValue>, (Box<TaskFrame>, SyscallError)>)
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
                        SyscallIntent::Return(task_frame, value) => {
                            SyscallIntent::Return(task_frame, Ok(value))
                        }
                        SyscallIntent::SwitchTo(frame) => SyscallIntent::SwitchTo(frame),
                    },
                    Err((task_frame, error)) => SyscallIntent::Return(task_frame, Err(error)),
                };
                match intent {
                    SyscallIntent::Return(mut task_frame, value) => unsafe {
                        task_frame.set_syscall_return_value(
                            value
                                .map(|value| value.0)
                                .map_err(|err_code| err_code as u64),
                        );
                        task_frame
                    },
                    SyscallIntent::SwitchTo(task_frame) => task_frame,
                }
            });
        }
        println!("Syscalls initialized!");
    }
}

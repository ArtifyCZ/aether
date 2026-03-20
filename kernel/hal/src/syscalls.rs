use crate::arch::syscalls;
use crate::tasks::TaskFrame;
use alloc::boxed::Box;

#[repr(C, align(16))]
pub struct SyscallFrame(pub(crate) TaskFrame);

impl SyscallFrame {
    pub fn into_task_frame(self: Box<Self>) -> Box<TaskFrame> {
        unsafe {
            let ptr = Box::into_raw(self) as *mut TaskFrame;
            Box::from_raw(ptr)
        }
    }
}

pub unsafe fn init<F>(f: F)
where
    F: FnMut(Box<SyscallFrame>) -> Box<TaskFrame> + 'static,
{
    unsafe {
        syscalls::init(f);
    }
}

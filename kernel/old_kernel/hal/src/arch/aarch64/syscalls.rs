use crate::arch::aarch64::interrupts::InterruptFrame;
use crate::syscalls::SyscallFrame;
use crate::tasks::TaskFrame;
use alloc::boxed::Box;
use core::ffi::c_void;
use core::ptr::null_mut;

static mut HANDLER: Option<Box<dyn FnMut(Box<SyscallFrame>) -> Box<TaskFrame>>> = None;

pub unsafe fn init<F>(handler: F)
where
    F: FnMut(Box<SyscallFrame>) -> Box<TaskFrame> + 'static,
{
    unsafe {
        HANDLER = Some(Box::new(handler));
    }
}

pub unsafe fn interrupt_handler(interrupt_frame: *mut InterruptFrame) -> *mut InterruptFrame {
    let syscall_frame = Box::new(SyscallFrame(TaskFrame {
        hw_frame: interrupt_frame,
    }));
    let return_frame = unsafe {
        #[allow(static_mut_refs)]
        if let Some(handler) = HANDLER.as_mut() {
            handler(syscall_frame)
        } else {
            syscall_frame.into_task_frame()
        }
    };

    return_frame.hw_frame
}

impl SyscallFrame {
    pub fn number(&self) -> u64 {
        unsafe {
            let frame = self.0.hw_frame.as_ref().unwrap();
            frame.x[8]
        }
    }

    pub fn args(&self) -> [u64; 5] {
        unsafe {
            let frame = self.0.hw_frame.as_ref().unwrap();
            [frame.x[0], frame.x[1], frame.x[2], frame.x[3], frame.x[4]]
        }
    }
}

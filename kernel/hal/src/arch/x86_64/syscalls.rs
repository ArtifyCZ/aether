use crate::arch::x86_64::interrupts::InterruptFrame;
use crate::syscalls::SyscallFrame;
use crate::tasks::TaskFrame;
use alloc::boxed::Box;

static mut HANDLER: Option<Box<dyn FnMut(Box<SyscallFrame>) -> Box<TaskFrame>>> = None;

pub unsafe fn init<F>(handler: F)
where
    F: FnMut(Box<SyscallFrame>) -> Box<TaskFrame> + 'static,
{
    unsafe {
        HANDLER = Some(Box::new(handler));
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn syscalls_inner_handler(interrupt_frame: *mut InterruptFrame) -> usize {
    assert!(interrupt_frame.is_aligned(), "Interrupt frame {:p} not aligned!", interrupt_frame);
    let syscall_frame = Box::new(SyscallFrame(TaskFrame {
        hw_frame: interrupt_frame,
    }));
    assert!((&raw const *syscall_frame).is_aligned(), "Syscall frame {:p} is not aligned!", syscall_frame);

    let return_frame = unsafe {
        #[allow(static_mut_refs)]
        if let Some(handler) = HANDLER.as_mut() {
            handler(syscall_frame)
        } else {
            syscall_frame.into_task_frame()
        }
    };

    return_frame.hw_frame as usize
}

impl SyscallFrame {
    pub fn number(&self) -> u64 {
        unsafe {
            assert!(self.0.hw_frame.is_aligned(), "Hw frame {:p} is not aligned!", self.0.hw_frame);
            let frame = self.0.hw_frame.as_ref().unwrap();
            frame.rax
        }
    }

    pub fn args(&self) -> [u64; 5] {
        unsafe {
            let frame = self.0.hw_frame.as_ref().unwrap();
            [frame.rdi, frame.rsi, frame.rdx, frame.r10, frame.r8]
        }
    }
}

use crate::arch::interrupts::InterruptFrame;
use alloc::boxed::Box;

#[derive(Debug)]
#[repr(C)]
pub struct TaskFrame {
    pub(crate) hw_frame: *mut InterruptFrame, // Registers, PC, PSTATE
}

impl TaskFrame {
    pub unsafe fn from_ptr_legacy(
        legacy_ptr: *mut kernel_bindings_gen::interrupt_frame,
    ) -> Box<Self> {
        Box::new(Self {
            hw_frame: legacy_ptr.cast(),
        })
    }

    pub unsafe fn to_legacy_ptr(self: Box<Self>) -> *mut kernel_bindings_gen::interrupt_frame {
        self.hw_frame.cast()
    }

    pub unsafe fn set_syscall_return_value(&mut self, value: Result<u64, u64>) {
        unsafe {
            self.hw_frame
                .as_mut()
                .unwrap()
                .set_syscall_return_value(value);
        }
    }
}

pub unsafe fn prepare_switch(kernel_stack_top: usize, task_id: u64) {
    crate::arch::tasks::prepare_switch(kernel_stack_top, task_id);
}

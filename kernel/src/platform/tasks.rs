use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::syscalls::{SyscallError, SyscallReturnValue, SyscallReturnable};
use crate::platform::tasks::bindings::vmm_context;
use crate::platform::virtual_memory_manager_context::VirtualMemoryManagerContext;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::ffi::c_void;
use core::pin::Pin;
use crate::task_id::TaskId;

mod bindings {
    include_bindings!("tasks.rs");
}

pub const TASK_KERNEL_STACK_SIZE: usize = 4 * PAGE_FRAME_SIZE;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(transparent)]
#[must_use]
pub struct TaskFrame(pub(super) *mut bindings::interrupt_frame);

unsafe impl Send for TaskFrame {}

#[derive(Debug)]
pub struct TaskContext {
    task_id: TaskId,
    #[allow(unused)]
    user_ctx: Option<Arc<VirtualMemoryManagerContext>>,
    kernel_stack: Pin<Box<[u8]>>,
    state: TaskFrame,
    pending_syscall_return_value: Option<Result<SyscallReturnValue, SyscallError>>,
}

impl TaskContext {
    pub fn new_user(
        task_id: TaskId,
        user_ctx: Arc<VirtualMemoryManagerContext>,
        user_stack_vaddr: usize,
        entrypoint_vaddr: usize,
    ) -> Self {
        let kernel_stack = unsafe {
            Pin::new_unchecked(Box::<[u8]>::new_zeroed_slice(TASK_KERNEL_STACK_SIZE).assume_init())
        };

        let state = unsafe {
            let user_ctx_ptr: *const vmm_context = core::mem::transmute(user_ctx.inner());
            let kernel_stack_top = kernel_stack.as_ptr_range().end as usize;
            TaskFrame(bindings::task_setup_user(
                user_ctx_ptr,
                entrypoint_vaddr,
                user_stack_vaddr,
                kernel_stack_top,
            ))
        };

        Self {
            task_id,
            user_ctx: Some(user_ctx),
            kernel_stack,
            state,
            pending_syscall_return_value: None,
        }
    }

    pub fn new_kernel(
        task_id: TaskId,
        function: unsafe extern "C" fn(arg: *mut c_void),
        arg: *mut c_void,
        kernel_stack_size: usize,
    ) -> Self {
        let kernel_stack = unsafe {
            Pin::new_unchecked(Box::<[u8]>::new_zeroed_slice(kernel_stack_size).assume_init())
        };

        let state = unsafe {
            let kernel_stack_top = kernel_stack.as_ptr_range().end as usize;
            TaskFrame(bindings::task_setup_kernel(
                kernel_stack_top,
                Some(function),
                arg,
            ))
        };

        Self {
            task_id,
            user_ctx: None,
            kernel_stack,
            state,
            pending_syscall_return_value: None,
        }
    }

    pub fn get_virtual_memory_manager(&self) -> &Arc<VirtualMemoryManagerContext> {
        self.user_ctx.as_ref().unwrap()
    }

    pub fn set_frame(&mut self, state: TaskFrame) {
        self.state = state;
    }

    pub fn return_syscall_value(&mut self, value: Result<impl SyscallReturnable, SyscallError>) {
        self.pending_syscall_return_value = Some(value.map(|value| value.into_return_value()));
    }

    pub fn activate(&mut self) -> TaskFrame {
        let kernel_stack_top = self.kernel_stack.as_ptr_range().end as usize;
        unsafe {
            bindings::task_prepare_switch(kernel_stack_top, self.task_id.get());
        }
        let mut frame = self.state;
        if let Some(value) = self.pending_syscall_return_value.take() {
            unsafe {
                frame.set_syscall_return_value(value);
            }
        }
        frame
    }
}

impl TaskId {
    /// Returns the current task id of the current CPU (the CPU core this function is invoked on).
    pub fn get_current() -> Option<Self> {
        let task_id = unsafe {
            bindings::task_get_current_id()
        };
        if task_id == 0 {
            None
        } else {
            Some(unsafe { TaskId::from_u64(task_id) })
        }
    }
}

impl TaskFrame {
    pub(super) unsafe fn set_syscall_return_value(&mut self, value: Result<SyscallReturnValue, SyscallError>) {
        unsafe {
            let (error_code, value) = match value {
                Ok(value) => (SyscallError::SYS_SUCCESS, value),
                Err(error_code) => (error_code, ().into_return_value()),
            };
            bindings::task_set_syscall_return_value(self.0, error_code as u64, value.0);
        }
    }
}

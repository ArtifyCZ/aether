use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::physical_memory_manager::PhysicalMemoryManager;
use crate::platform::tasks::bindings::vmm_context;
use crate::platform::virtual_memory_manager_context::{
    VirtualMemoryManagerContext, VirtualMemoryMappingFlags,
};
use crate::platform::virtual_page_address::VirtualPageAddress;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::ffi::c_void;
use core::pin::Pin;

pub(super) mod bindings {
    include_bindings!("tasks.rs");
}

pub const TASK_KERNEL_STACK_SIZE: usize = 4 * PAGE_FRAME_SIZE;

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct TaskFrame(pub(super) *mut bindings::interrupt_frame);

unsafe impl Send for TaskFrame {}

#[derive(Debug)]
pub struct TaskContext {
    #[allow(unused)]
    user_ctx: Option<Arc<VirtualMemoryManagerContext>>,
    kernel_stack: Pin<Box<[u8]>>,
    state: TaskFrame,
}

impl TaskContext {
    pub fn new_user(user_ctx: Arc<VirtualMemoryManagerContext>, user_stack_vaddr: usize, entrypoint_vaddr: usize) -> Self {
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
            user_ctx: Some(user_ctx),
            kernel_stack,
            state,
        }
    }

    pub fn new_kernel(
        function: unsafe extern "C" fn(arg: *mut c_void),
        arg: *mut c_void,
        kernel_stack_size: usize,
    ) -> Self {
        let kernel_stack = unsafe {
            Pin::new_unchecked(Box::<[u8]>::new_zeroed_slice(kernel_stack_size).assume_init())
        };

        let state = unsafe {
            let kernel_stack_top = kernel_stack.as_ptr_range().end as usize;
            TaskFrame(bindings::task_setup_kernel(kernel_stack_top, Some(function), arg))
        };

        Self {
            user_ctx: None,
            kernel_stack,
            state,
        }
    }

    pub fn get_virtual_memory_manager(&self) -> &Arc<VirtualMemoryManagerContext> {
        self.user_ctx.as_ref().unwrap()
    }

    pub fn get_state(&self) -> TaskFrame {
        self.state
    }

    pub fn set_state(&mut self, state: TaskFrame) {
        self.state = state;
    }

    pub(super) unsafe fn set_syscall_return_value(&mut self, value: u64) {
        unsafe {
            bindings::task_set_syscall_return_value(self.state.0, value);
        }
    }

    pub fn prepare_switch(&self) {
        unsafe {
            let kernel_stack_top = self.kernel_stack.as_ptr_range().end as usize;
            bindings::task_prepare_switch(kernel_stack_top);
        }
    }
}

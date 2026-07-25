use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::syscalls::{SyscallError, SyscallReturnValue, SyscallReturnable};
use crate::platform::virtual_memory_manager_context::VirtualMemoryManagerContext;
use crate::task_id::TaskId;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::ffi::c_void;
use core::pin::Pin;
use kernel_hal::tasks;
use kernel_hal::tasks::TaskFrame;

pub const TASK_KERNEL_STACK_SIZE: usize = 8 * PAGE_FRAME_SIZE;

#[derive(Debug)]
pub struct TaskContext {
    task_id: TaskId,
    #[allow(unused)]
    user_ctx: Option<Arc<VirtualMemoryManagerContext>>,
    kernel_stack: Pin<Box<[u8]>>,
    state: Option<Box<TaskFrame>>,
    proc_handles: BTreeMap<u64, Arc<VirtualMemoryManagerContext>>,
    next_handle: u64,
    pending_syscall_return_value: Option<Result<SyscallReturnValue, SyscallError>>,
}

impl TaskContext {
    pub fn new_user(
        task_id: TaskId,
        user_ctx: Arc<VirtualMemoryManagerContext>,
        user_stack_vaddr: usize,
        entrypoint_vaddr: usize,
        arg: u64,
    ) -> Self {
        let kernel_stack = unsafe {
            Pin::new_unchecked(Box::<[u8]>::new_zeroed_slice(TASK_KERNEL_STACK_SIZE).assume_init())
        };

        let state = unsafe {
            let kernel_stack_top = kernel_stack.as_ptr_range().end as usize;
            tasks::setup_user(
                user_ctx.inner(),
                entrypoint_vaddr,
                user_stack_vaddr,
                kernel_stack_top,
                arg,
            )
        };

        Self {
            task_id,
            user_ctx: Some(user_ctx),
            kernel_stack,
            state: Some(state),
            proc_handles: BTreeMap::new(),
            next_handle: 1,
            pending_syscall_return_value: None,
        }
    }

    pub fn new_kernel(
        task_id: TaskId,
        function: unsafe extern "C" fn(arg: *mut c_void) -> !,
        arg: *mut c_void,
        kernel_stack_size: usize,
    ) -> Self {
        let kernel_stack = unsafe {
            Pin::new_unchecked(Box::<[u8]>::new_zeroed_slice(kernel_stack_size).assume_init())
        };

        let state = unsafe {
            let kernel_stack_top = kernel_stack.as_ptr_range().end as usize;
            tasks::setup_kernel(kernel_stack_top, function, arg)
        };

        Self {
            task_id,
            user_ctx: None,
            kernel_stack,
            state: Some(state),
            proc_handles: BTreeMap::new(),
            next_handle: 1,
            pending_syscall_return_value: None,
        }
    }

    pub fn get_virtual_memory_manager(&self) -> &Arc<VirtualMemoryManagerContext> {
        self.user_ctx.as_ref().unwrap()
    }

    pub fn set_frame(&mut self, state: Box<TaskFrame>) {
        self.state = Some(state);
    }

    pub fn return_syscall_value(&mut self, value: Result<impl SyscallReturnable, SyscallError>) {
        self.pending_syscall_return_value = Some(value.map(|value| value.into_return_value()));
    }

    pub fn activate(&mut self) -> Box<TaskFrame> {
        let mut frame = self.state.take().unwrap();
        let kernel_stack_top = self.kernel_stack.as_ptr_range().end as usize;
        unsafe {
            tasks::prepare_switch(kernel_stack_top, self.task_id.get());
        }
        if let Some(value) = self.pending_syscall_return_value.take() {
            unsafe {
                frame
                    .set_syscall_return_value(value.map(|value| value.0).map_err(|err| err as u64));
            }
        }
        frame
    }

    pub fn add_proc_handle(&mut self, vmm: Arc<VirtualMemoryManagerContext>) -> u64 {
        // @TODO: use newtype pattern for handles
        // @TODO: probably extract the address space (VMM) and proc handles (capabilities) to ProcessControlBlock or a similar struct
        let handle = self.next_handle;
        self.next_handle += 1;
        self.proc_handles.insert(handle, vmm);
        handle
    }

    pub fn get_proc_handle(&self, handle: u64) -> Option<&Arc<VirtualMemoryManagerContext>> {
        self.proc_handles.get(&handle)
    }
}

impl TaskId {
    /// Returns the current task id of the current CPU (the CPU core this function is invoked on).
    pub fn get_current() -> Option<Self> {
        let task_id = unsafe { tasks::get_current_id() };
        if task_id == 0 {
            None
        } else {
            Some(unsafe { TaskId::from_u64(task_id) })
        }
    }
}

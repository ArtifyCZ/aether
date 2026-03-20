use crate::interrupt_safe_spin_lock::{InterruptSafeSpinLock, InterruptSafeSpinLockGuard};
use crate::platform::tasks::TaskContext;
use crate::platform::virtual_memory_manager_context::VirtualMemoryManagerContext;
use crate::task_id::TaskId;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::ops::{Deref, DerefMut};

#[derive(Default, Debug)]
pub struct TaskRegistry(InterruptSafeSpinLock<TaskRegistryInner>);

#[derive(Default, Debug)]
struct TaskRegistryInner {
    tasks: BTreeMap<TaskId, TaskContext>,
}

#[derive(Debug)]
pub enum TaskSpec {
    User {
        virtual_memory_manager_context: Arc<VirtualMemoryManagerContext>,
        user_stack_vaddr: usize,
        entrypoint_vaddr: usize,
        arg: u64,
    },
    Kernel {
        function: unsafe extern "C" fn(arg: *mut core::ffi::c_void) -> !,
        arg: *mut core::ffi::c_void,
        kernel_stack_size: usize,
    },
}

impl TaskRegistry {
    pub fn new() -> &'static Self {
        Box::leak(Box::new(Default::default()))
    }

    pub fn get(&self, id: TaskId) -> Option<TaskGuard<'_>> {
        let inner = self.0.lock();
        if !inner.tasks.contains_key(&id) {
            return None;
        }
        Some(TaskGuard { id, inner })
    }

    pub fn create_task(&self, spec: TaskSpec) -> TaskId {
        let id = TaskId::new();
        self.insert(
            id,
            match spec {
                TaskSpec::User {
                    virtual_memory_manager_context,
                    user_stack_vaddr,
                    entrypoint_vaddr,
                    arg,
                } => TaskContext::new_user(
                    id,
                    virtual_memory_manager_context,
                    user_stack_vaddr,
                    entrypoint_vaddr,
                    arg,
                ),
                TaskSpec::Kernel {
                    function,
                    arg,
                    kernel_stack_size,
                } => TaskContext::new_kernel(id, function, arg, kernel_stack_size),
            },
        );
        id
    }

    fn insert(&self, id: TaskId, task: TaskContext) {
        let mut inner = self.0.lock();
        if inner.tasks.contains_key(&id) {
            panic!("Trying to insert a task with an existing id: {:?}!", id);
        }
        inner.tasks.insert(id, task);
    }
}

pub struct TaskGuard<'a> {
    id: TaskId,
    inner: InterruptSafeSpinLockGuard<'a, TaskRegistryInner>,
}

impl Deref for TaskGuard<'_> {
    type Target = TaskContext;

    fn deref(&self) -> &Self::Target {
        self.inner
            .tasks
            .get(&self.id)
            .expect("Non-existence should be checked before creating TaskGuard!")
    }
}

impl DerefMut for TaskGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
            .tasks
            .get_mut(&self.id)
            .expect("Non-existence should be checked before creating TaskGuard!")
    }
}

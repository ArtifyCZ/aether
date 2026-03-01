use crate::interrupt_safe_spin_lock::InterruptSafeSpinLock;
use crate::platform::drivers::serial::SerialDriver;
use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::tasks::{TaskContext, TaskFrame};
use crate::task_id::TaskId;
use crate::task_registry::{TaskGuard, TaskRegistry, TaskSpec};
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::ffi::c_void;
use core::ops::Deref;
use core::ptr::null_mut;

#[derive(Debug)]
pub struct Scheduler(InterruptSafeSpinLock<SchedulerInner>);

#[derive(Debug)]
#[repr(C)]
struct SchedulerInner {
    null_task: TaskId,
    started: bool,
    tasks: &'static TaskRegistry,
    ready_tasks: VecDeque<TaskId>,
}

impl SchedulerInner {
    fn pick_next_task(&mut self) -> Option<TaskGuard<'_>> {
        if !self.started {
            return None;
        }

        while let Some(task_id) = self.ready_tasks.pop_front() {
            if let Some(task) = self.tasks.get(task_id) {
                return Some(task);
            }
        }

        None
    }
}

unsafe extern "C" fn null_thread(_arg: *mut c_void) {
    loop {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));

            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
        }
    }
}

impl Scheduler {
    pub fn init(task_registry: &'static TaskRegistry) -> &'static Self {
        unsafe {
            SerialDriver::println("Initializing scheduler...");
            let null_task_id = task_registry.create_task(TaskSpec::Kernel {
                function: null_thread,
                arg: null_mut(),
                kernel_stack_size: PAGE_FRAME_SIZE,
            });
            let mut ready_tasks = VecDeque::new();
            ready_tasks.push_back(null_task_id);
            let scheduler: &'static Self = Box::leak(Box::new(Scheduler(
                InterruptSafeSpinLock::new(SchedulerInner {
                    started: false,
                    null_task: null_task_id,
                    tasks: task_registry,
                    ready_tasks,
                }),
            )));
            SerialDriver::println("Scheduler initialized!");
            scheduler
        }
    }

    pub fn start(&self) {
        let mut inner = self.0.lock();
        inner.started = true;
    }

    pub fn spawn(&self, task: TaskSpec) -> TaskId {
        let mut inner = self.0.lock();
        let id = inner.tasks.create_task(task);
        inner.ready_tasks.push_back(id);
        id
    }

    pub fn access_current_task_context<TOut>(
        &self,
        f: impl FnOnce(&TaskContext) -> TOut,
    ) -> Option<TOut> {
        let inner = self.0.lock();
        if !inner.started {
            return None;
        }
        let task_id = TaskId::get_current()?;
        let task = inner.tasks.get(task_id).unwrap();
        Some(f(task.deref()))
    }

    pub fn heartbeat(&self, prev_frame: TaskFrame) -> Option<TaskFrame> {
        let mut inner = self.0.lock();
        if !inner.started {
            return None;
        }

        if let Some(prev_task_id) = TaskId::get_current() {
            let mut prev_task = inner.tasks.get(prev_task_id).unwrap();
            prev_task.set_frame(prev_frame);
            inner.ready_tasks.push_back(prev_task_id);
        }

        let next_task = inner.pick_next_task()?;
        Some(next_task.activate())
    }

    pub fn exit_current_task(&self, prev_frame: TaskFrame) -> Option<TaskFrame> {
        let mut inner = self.0.lock();
        if !inner.started {
            return None;
        }

        if let Some(prev_id) = TaskId::get_current() {
            let mut prev_task = inner.tasks.get(prev_id).unwrap();
            prev_task.set_frame(prev_frame);
            if let Some((idx, _)) = inner
                .ready_tasks
                .iter()
                .enumerate()
                .find(|(_, task_id)| **task_id == prev_id)
            {
                inner.ready_tasks.remove(idx);
            }
        }

        let next_task = inner.pick_next_task()?;
        Some(next_task.activate())
    }
}

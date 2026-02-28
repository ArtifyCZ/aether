use crate::interrupt_safe_spin_lock::InterruptSafeSpinLock;
use crate::platform::drivers::serial::SerialDriver;
use crate::platform::tasks::TaskContext;
use crate::task_id::TaskId;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, VecDeque};

#[derive(Default)]
pub struct Scheduler(InterruptSafeSpinLock<SchedulerInner>);

#[repr(C)]
struct SchedulerInner {
    current_task: Option<TaskId>,
    null_task: TaskId,
    started: bool,
    tasks: BTreeMap<TaskId, TaskContext>,
    ready_tasks: VecDeque<TaskId>,
}

impl Default for SchedulerInner {
    fn default() -> Self {
        let null_task_id = TaskId::new();
        let null_task = TaskContext::new_kernel_null();
        let mut tasks = BTreeMap::new();
        tasks.insert(null_task_id, null_task);
        let mut ready_tasks = VecDeque::new();
        ready_tasks.push_back(null_task_id);
        SchedulerInner {
            started: false,
            current_task: None,
            null_task: null_task_id,
            tasks,
            ready_tasks,
        }
    }
}

impl SchedulerInner {
    fn pick_next_task(&mut self) -> Option<&mut TaskContext> {
        if !self.started {
            return None;
        }

        while let Some(task_id) = self.ready_tasks.pop_front() {
            if self.tasks.contains_key(&task_id) {
                self.current_task = Some(task_id);
                return Some(self.tasks.get_mut(&task_id).unwrap());
            }
        }

        None
    }
}

impl Scheduler {
    pub fn init() -> &'static Self {
        unsafe {
            SerialDriver::println("Initializing scheduler...");
            let scheduler: &'static Self = Box::leak(Box::new(Default::default()));
            SerialDriver::println("Scheduler initialized!");
            scheduler
        }
    }

    pub fn start(&self) {
        let mut inner = self.0.lock();
        inner.started = true;
    }

    pub fn add(&self, task: TaskContext) {
        let mut inner = self.0.lock();
        let task_id = TaskId::new();
        inner.tasks.insert(task_id, task);
        inner.ready_tasks.push_back(task_id);
    }

    pub fn update_current_task_context(&self, f: impl FnOnce(&mut TaskContext)) {
        let mut inner = self.0.lock();
        if !inner.started {
            return;
        }
        let task_id = match inner.current_task {
            Some(task_id) => task_id,
            None => return,
        };
        let task = inner.tasks.get_mut(&task_id).unwrap();
        f(task);
    }

    pub fn access_current_task_context<TOut>(
        &self,
        f: impl FnOnce(&TaskContext) -> TOut,
    ) -> Option<TOut> {
        let inner = self.0.lock();
        if !inner.started {
            return None;
        }
        let task_id = inner.current_task?;
        Some(f(&inner.tasks[&task_id]))
    }

    pub fn heartbeat<FPrev, FNext, TOut>(&self, f_prev: FPrev, f_next: FNext) -> Option<TOut>
    where
        FPrev: FnOnce(&mut TaskContext),
        FNext: FnOnce(&mut TaskContext) -> TOut,
    {
        let mut inner = self.0.lock();
        if !inner.started {
            return None;
        }

        if let Some(prev_idx) = inner.current_task {
            let prev_task = inner.tasks.get_mut(&prev_idx).unwrap();
            f_prev(prev_task);
            inner.ready_tasks.push_back(prev_idx);
        }

        let next_task = inner.pick_next_task()?;
        let result = f_next(next_task);

        Some(result)
    }

    pub fn exit_current_task<FPrev, FNext, TOut>(
        &self,
        f_prev: FPrev,
        f_next: FNext,
    ) -> Option<TOut>
    where
        FPrev: FnOnce(&mut TaskContext),
        FNext: FnOnce(&mut TaskContext) -> TOut,
    {
        let mut inner = self.0.lock();
        if !inner.started {
            return None;
        }

        if let Some(prev_id) = inner.current_task {
            let prev_task = inner.tasks.get_mut(&prev_id).unwrap();
            f_prev(prev_task);
            inner.tasks.remove(&prev_id);
        }

        let next_task = inner.pick_next_task()?;
        let result = f_next(next_task);

        Some(result)
    }
}

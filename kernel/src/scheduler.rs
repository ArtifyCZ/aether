use crate::interrupt_safe_spin_lock::{InterruptSafeSpinLock, InterruptSafeSpinLockGuard};
use crate::platform::drivers::serial::SerialDriver;
use crate::platform::tasks::{TaskContext, TaskFrame};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::ptr::null_mut;
use crate::platform::memory_layout::PAGE_FRAME_SIZE;

#[derive(Default)]
pub struct Scheduler(InterruptSafeSpinLock<SchedulerInner>);

#[repr(C)]
struct SchedulerInner {
    current_task: i32,
    started: bool,
    tasks: Vec<TaskContext>,
}

impl Default for SchedulerInner {
    fn default() -> Self {
        SchedulerInner {
            current_task: -1,
            started: false,
            tasks: Vec::with_capacity(16),
        }
    }
}

unsafe extern "C" fn scheduler_null_thread(_arg: *mut c_void) {
    loop {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));

            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
        }
    }
}

impl SchedulerInner {
    fn find_next_runnable_task(&mut self) -> Option<(usize, &mut TaskContext)> {
        for offset in 1..=self.tasks.len() {
            let idx = if self.current_task < 0 {
                offset - 1
            } else {
                ((self.current_task as usize) + offset) % self.tasks.len()
            };
            return Some((idx, &mut self.tasks[idx]));
        }

        None
    }
}

impl Scheduler {
    pub fn init() -> &'static Self {
        unsafe {
            SerialDriver::println("Initializing scheduler...");
            let scheduler: &'static Self = Box::leak(Box::new(Default::default()));
            scheduler.add(TaskContext::new_kernel(scheduler_null_thread, null_mut(), PAGE_FRAME_SIZE));
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
        inner.tasks.push(task);
    }

    pub fn update_current_task_context(&self, f: impl FnOnce(&mut TaskContext)) {
        let mut inner = self.0.lock();
        if inner.current_task < 0 {
            return;
        }
        let current_task = inner.current_task as usize;
        f(&mut inner.tasks[current_task]);
    }

    pub fn access_current_task_context<TOut>(&self, f: impl FnOnce(&TaskContext) -> TOut) -> Option<TOut> {
        let inner = self.0.lock();
        if !inner.started {
            return None;
        }
        let current_task = inner.current_task as usize;
        Some(f(&inner.tasks[current_task]))
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

        if inner.current_task >= 0 {
            let prev_idx = inner.current_task as usize;
            let prev_task = &mut inner.tasks[prev_idx];
            f_prev(prev_task);
        }

        let (next_idx, next_task) = inner.find_next_runnable_task().unwrap();
        let result = f_next(next_task);
        inner.current_task = next_idx as i32;

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

        if inner.current_task >= 0 {
            let prev_idx = inner.current_task as usize;
            let prev_task = &mut inner.tasks[prev_idx];
            f_prev(prev_task);
            inner.tasks.remove(prev_idx);

            if prev_idx == 0 {
                inner.current_task = inner.tasks.len() as i32 - 1;
            } else {
                inner.current_task -= 1;
            }
        }

        let (next_idx, next_task) = inner.find_next_runnable_task().unwrap();
        let result = f_next(next_task);
        inner.current_task = next_idx as i32;

        Some(result)
    }
}

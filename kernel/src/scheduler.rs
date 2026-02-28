use crate::interrupt_safe_spin_lock::{InterruptSafeSpinLock, InterruptSafeSpinLockGuard};
use crate::platform::drivers::serial::SerialDriver;
use crate::platform::tasks::{TaskContext, TaskFrame};
use alloc::boxed::Box;
use alloc::vec::Vec;

#[repr(C)]
pub struct Scheduler {
    current_task: i32,
    started: bool,
    tasks: Vec<TaskContext>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Scheduler {
            current_task: -1,
            started: false,
            tasks: Vec::with_capacity(16),
        }
    }
}

impl Scheduler {
    pub fn init() -> &'static InterruptSafeSpinLock<Self> {
        unsafe {
            SerialDriver::println("Initializing scheduler...");
            let scheduler = Box::leak(Box::new(Default::default()));
            SerialDriver::println("Scheduler initialized!");
            scheduler
        }
    }

    pub fn start(&mut self) {
        self.started = true;
    }

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

    pub fn add(&mut self, task: TaskContext) {
        self.tasks.push(task);
    }

    pub fn exit_current_task(&mut self) {
        if self.current_task < 0 {
            return;
        }

        self.tasks.remove(self.current_task as usize);

        if self.current_task == 0 {
            self.current_task = self.tasks.len() as i32 - 1;
        } else {
            self.current_task -= 1;
        }
    }

    pub fn get_current_task_mut(&mut self) -> Option<&mut TaskContext> {
        if self.current_task < 0 {
            None
        } else {
            self.tasks.get_mut(self.current_task as usize)
        }
    }

    pub fn pick_next(&mut self) -> Option<&mut TaskContext> {
        if !self.started {
            return None;
        }

        let (next_idx, _next_task) = match self.find_next_runnable_task() {
            None => {
                return None;
            }
            Some((next_idx, next_task)) => (next_idx as i32, next_task),
        };

        self.current_task = next_idx;
        let next_task = &mut self.tasks[next_idx as usize];

        Some(next_task)
    }
}

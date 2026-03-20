use crate::interrupt_safe_spin_lock::InterruptSafeSpinLock;
use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::println;
use crate::task_id::TaskId;
use crate::task_registry::{TaskGuard, TaskRegistry, TaskSpec};
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, VecDeque};
use core::ffi::c_void;
use core::ptr::null_mut;
use kernel_hal::tasks::TaskFrame;

#[derive(Debug)]
pub struct Scheduler(InterruptSafeSpinLock<SchedulerInner>);

#[derive(Debug)]
#[repr(C)]
struct SchedulerInner {
    null_task: TaskId,
    started: bool,
    tasks: &'static TaskRegistry,
    ready_tasks: VecDeque<TaskId>,
    waiting_for_irq_tasks: BTreeMap<u8, TaskId>,
    pending_irq_interrupts: u64,
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

        Some(self.tasks.get(self.null_task)?)
    }
}

unsafe extern "C" fn null_thread(_arg: *mut c_void) -> ! {
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
        println!("Initializing scheduler...");
        let null_task_id = task_registry.create_task(TaskSpec::Kernel {
            function: null_thread,
            arg: null_mut(),
            kernel_stack_size: PAGE_FRAME_SIZE,
        });
        let mut ready_tasks = VecDeque::new();
        ready_tasks.push_back(null_task_id);
        let scheduler: &'static Self = Box::leak(Box::new(Scheduler(InterruptSafeSpinLock::new(
            SchedulerInner {
                started: false,
                null_task: null_task_id,
                tasks: task_registry,
                ready_tasks,
                waiting_for_irq_tasks: BTreeMap::new(),
                pending_irq_interrupts: 0,
            },
        ))));
        println!("Scheduler initialized!");
        scheduler
    }

    pub fn start(&self) -> Box<TaskFrame> {
        let mut inner = self.0.lock();
        inner.started = true;
        let mut first_task = inner.pick_next_task().unwrap();
        first_task.activate()
    }

    pub fn spawn(&self, task: TaskSpec) -> TaskId {
        let mut inner = self.0.lock();
        let id = inner.tasks.create_task(task);
        inner.ready_tasks.push_back(id);
        id
    }

    pub fn heartbeat(&self, prev_frame: Box<TaskFrame>) -> Box<TaskFrame> {
        let mut inner = self.0.lock();
        if !inner.started {
            return prev_frame;
        }

        if let Some(prev_task_id) = TaskId::get_current() {
            let mut prev_task = inner.tasks.get(prev_task_id).unwrap();
            prev_task.set_frame(prev_frame);
            inner.ready_tasks.push_back(prev_task_id);
        }

        let mut next_task = inner.pick_next_task().unwrap();
        next_task.activate()
    }

    pub fn wait_for_irq(&self, irq: u8, prev_frame: Box<TaskFrame>) -> Box<TaskFrame> {
        let mut inner = self.0.lock();
        if !inner.started {
            return prev_frame;
        }
        let pending_irq_bitmask = 1u64 << irq;
        if inner.pending_irq_interrupts & pending_irq_bitmask != 0 {
            inner.pending_irq_interrupts &= !pending_irq_bitmask;
            return prev_frame;
        }
        {
            let prev_task_id = TaskId::get_current().unwrap();
            let mut prev_task = inner.tasks.get(prev_task_id).unwrap();
            prev_task.set_frame(prev_frame);
            inner.waiting_for_irq_tasks.insert(irq, prev_task_id);
        }

        let mut next_task = inner.pick_next_task().unwrap();
        next_task.activate()
    }

    pub fn signal_irq(&self, irq: u8, prev_frame: Box<TaskFrame>) -> Box<TaskFrame> {
        let mut inner = self.0.lock();
        if !inner.started {
            return prev_frame;
        }

        let next_task_id = match inner.waiting_for_irq_tasks.remove(&irq) {
            Some(waiting_task_id) => waiting_task_id,
            None => {
                inner.pending_irq_interrupts |= 1u64 << irq;
                return prev_frame;
            }
        };

        if let Some(prev_task_id) = TaskId::get_current() {
            let mut prev_task = inner.tasks.get(prev_task_id).unwrap();
            prev_task.set_frame(prev_frame);
            inner.ready_tasks.push_back(prev_task_id);
        }

        let mut next_task = inner.tasks.get(next_task_id).unwrap();
        next_task.activate()
    }

    pub fn exit_current_task(&self, prev_frame: Box<TaskFrame>) -> Box<TaskFrame> {
        let mut inner = self.0.lock();
        if !inner.started {
            return prev_frame;
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

        let mut next_task = inner.pick_next_task().unwrap();
        next_task.activate()
    }
}

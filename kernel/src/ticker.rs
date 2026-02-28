use crate::platform::drivers::serial::SerialDriver;
use crate::platform::tasks::TaskFrame;
use crate::platform::timer::Timer;
use crate::scheduler::Scheduler;
use alloc::boxed::Box;
use alloc::format;

pub struct Ticker {
    scheduler: &'static Scheduler,
}

impl Ticker {
    pub fn init(scheduler: &'static Scheduler) -> &'static Ticker {
        let ticker: &'static Ticker = Box::leak(Box::new(Self { scheduler }));

        Timer::set_tick_handler(|frame| ticker.tick_handler(frame));

        ticker
    }

    fn tick_handler(&self, prev_frame: TaskFrame) -> TaskFrame {
        let next_frame: TaskFrame = self
            .scheduler
            .heartbeat(
                |prev_task| {
                    prev_task.set_state(prev_frame);
                },
                |next_task| {
                    next_task.prepare_switch();
                    next_task.get_state()
                },
            )
            .unwrap_or(prev_frame);
        let ticks = Timer::get_ticks();
        if ticks % 100 == 0 {
            unsafe {
                SerialDriver::println(&format!("Timer ticks: {}", ticks));
            }
        }

        next_frame
    }
}

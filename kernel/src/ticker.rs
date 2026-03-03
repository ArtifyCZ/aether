use crate::platform::tasks::TaskFrame;
use crate::platform::timer::Timer;
use crate::println;
use crate::scheduler::Scheduler;
use alloc::boxed::Box;

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
            .heartbeat(prev_frame)
            .unwrap_or(prev_frame);
        let ticks = Timer::get_ticks();
        if ticks % 100 == 0 {
            println!("Timer ticks: {:08X}", ticks);
        }

        next_frame
    }
}

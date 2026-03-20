use crate::println;
use crate::scheduler::Scheduler;
use alloc::boxed::Box;
use kernel_hal::tasks::TaskFrame;
use kernel_hal::timer;

pub struct Ticker {
    scheduler: &'static Scheduler,
}

impl Ticker {
    pub fn init(freq_hz: u32, scheduler: &'static Scheduler) -> &'static Ticker {
        let ticker: &'static Ticker = Box::leak(Box::new(Self { scheduler }));

        unsafe {
            timer::init(freq_hz, |frame| ticker.tick_handler(frame));
        }

        ticker
    }

    fn tick_handler(&self, prev_frame: Box<TaskFrame>) -> Box<TaskFrame> {
        let next_frame: Box<TaskFrame> = self.scheduler.heartbeat(prev_frame);
        let ticks = timer::get_ticks();
        if ticks % 100 == 0 {
            println!("Timer ticks: {:08X}", ticks);
        }

        next_frame
    }
}

use crate::platform::drivers::serial::SerialDriver;
use crate::platform::tasks::TaskFrame;
use crate::platform::timer::Timer;
use crate::scheduler::Scheduler;
use alloc::format;
use core::ffi::c_void;

pub struct Ticker;

impl Ticker {
    pub unsafe fn init(scheduler: &'static Scheduler) {
        unsafe {
            Timer::set_tick_handler(Some(Self::tick_handler), scheduler as *const _ as *mut _);
        }
    }

    unsafe extern "C" fn tick_handler(
        frame: *mut *mut super::timer::bindings::interrupt_frame,
        scheduler: *mut c_void,
    ) -> bool {
        unsafe {
            let scheduler: &'static Scheduler = &*scheduler.cast();

            let prev_frame: *mut super::timer::bindings::interrupt_frame = frame.read();
            let prev_state = TaskFrame(prev_frame.cast());
            scheduler.update_current_task_context(|task| task.set_state(prev_state));
            let next_frame: TaskFrame = scheduler.heartbeat(|prev_task| {
                prev_task.set_state(prev_state);
            }, |next_task| {
                next_task.prepare_switch();
                next_task.get_state()
            }).unwrap_or(prev_state);
            let next_frame: *mut super::timer::bindings::interrupt_frame = next_frame.0.cast();
            frame.write(next_frame);

            let ticks = Timer::get_ticks();
            if ticks % 100 == 0 {
                SerialDriver::println(&format!("Timer ticks: {}", ticks));
            }

            true
        }
    }
}

use crate::arch::timer;
use crate::tasks::TaskFrame;
use alloc::boxed::Box;

pub unsafe fn init<F>(freq_hz: u32, handler: F)
where
    F: FnMut(Box<TaskFrame>) -> Box<TaskFrame> + 'static,
{
    unsafe {
        timer::init(freq_hz, handler);
    }
}

pub fn get_ticks() -> u64 {
    unsafe { timer::get_ticks() }
}

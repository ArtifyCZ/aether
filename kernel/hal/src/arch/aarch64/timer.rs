use crate::arch::aarch64::gic;
use crate::arch::aarch64::interrupts;
use crate::arch::aarch64::interrupts::InterruptFrame;
use crate::early_console;
use crate::tasks::TaskFrame;
use alloc::boxed::Box;
use core::arch::asm;
use core::ffi::c_void;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicU64, Ordering};

static TICKS: AtomicU64 = AtomicU64::new(0);

static mut FREQ_HZ: u32 = 0;
static mut HANDLER: Option<Box<dyn FnMut(Box<TaskFrame>) -> Box<TaskFrame>>> = None;

pub unsafe fn init<F>(freq_hz: u32, handler: F)
where
    F: FnMut(Box<TaskFrame>) -> Box<TaskFrame> + 'static,
{
    unsafe {
        let freq_hz = if freq_hz == 0 { 100 } else { freq_hz };

        FREQ_HZ = freq_hz;
        HANDLER = Some(Box::new(handler));
        TICKS.store(0, Ordering::Release);

        // Get the system counter's frequency (usually 62.5MHz on QEMU)
        let freq: u64;
        asm!("mrs {}, cntfrq_el0", out(reg) freq);

        let ticks_per_int = freq / freq_hz as u64;

        // Set the timer value (countdown)
        asm!("msr cntv_tval_el0, {}", in(reg) ticks_per_int);

        // Enable the timer and unmask the interrupt
        // Control register: bit 0 = enable, bit 1 = imask (0 to unmask)
        asm!("msr cntv_ctl_el0, {}", in(reg) 1u64);

        // Register the handler in your common system
        gic::configure_interrupt(interrupts::INTID_TIMER, 0x80);
        gic::unmask_vector(interrupts::INTID_TIMER);

        early_console::print("Timer initialized!");
    }
}

pub unsafe fn interrupt_handler(frame: *mut InterruptFrame) -> *mut InterruptFrame {
    unsafe {
        // Reset the timer for the next tick! (Crucial: ARM timers aren't periodic by default)
        let freq: u64;
        asm!("mrs {}, cntfrq_el0", out(reg) freq);
        let ticks_per_int: u64 = freq / FREQ_HZ as u64;
        asm!("msr cntv_tval_el0, {}", in(reg) ticks_per_int);

        TICKS.fetch_add(1, Ordering::SeqCst);

        let task_frame = Box::new(TaskFrame { hw_frame: frame });

        #[allow(static_mut_refs)]
        let return_frame = if let Some(handler) = HANDLER.as_mut() {
            handler(task_frame)
        } else {
            task_frame
        };

        return_frame.hw_frame
    }
}

pub unsafe fn get_ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

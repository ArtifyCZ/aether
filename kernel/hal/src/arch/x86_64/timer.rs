use crate::arch::x86_64::interrupts::InterruptFrame;
use crate::arch::x86_64::io_wrapper::{inb, outb};
use crate::arch::x86_64::{interrupts, lapic};
use crate::early_console;
use crate::tasks::TaskFrame;
use alloc::boxed::Box;
use core::ffi::c_void;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicU64, Ordering};

const TARGET_MS: u32 = 10;

// Value 0x3 corresponds to a divisor of 16
const TIMER_DIVISOR: u32 = 0x3;

const PIT_FREQ: u32 = 1193182;
const PIT_CMD: u16 = 0x43;
const PIT_CH0_DATA: u16 = 0x40;

static mut TICKS_PER_MS: u32 = 0;

static TICKS: AtomicU64 = AtomicU64::new(0);

static mut HANDLER: Option<Box<dyn FnMut(Box<TaskFrame>) -> Box<TaskFrame>>> = None;

unsafe fn calibrate_timer() {
    // Software-enable LAPIC via Spurious Vector Register
    let svr = lapic::read(lapic::REG_SVR);
    lapic::write(lapic::REG_SVR, svr | 0x100);

    // Set divider
    lapic::write(lapic::REG_TDCR, TIMER_DIVISOR);

    // Prepare PIT for 10 ms delay (Mode 0)
    let pit_reload_value: u16 = (PIT_FREQ / 100) as u16;
    outb(PIT_CMD, 0x30);
    outb(PIT_CH0_DATA, (pit_reload_value & 0xFF) as u8);
    outb(PIT_CH0_DATA, ((pit_reload_value >> 8) & 0xFF) as u8);

    // Start LAPIC timer countdown from max
    lapic::write(lapic::REG_TICRET, 0xFFFFFFFF);

    // Poll PIT until it hits zero
    loop {
        outb(PIT_CMD, 0x00); // Latch
        let low = inb(PIT_CH0_DATA) as u16;
        let high = inb(PIT_CH0_DATA) as u16;
        if (low | (high << 8)) == 0 {
            break;
        }
    }

    // Calculate how many ticks passed in 10 ms
    let current_count = lapic::read(lapic::REG_TCCR);
    let ticks_in_10ms = 0xFFFFFFFF - current_count;

    // Stop the timer
    lapic::write(lapic::REG_TICRET, 0);

    TICKS_PER_MS = ticks_in_10ms / TARGET_MS;
}

pub unsafe fn init<F>(freq_hz: u32, handler: F)
where
    F: FnMut(Box<TaskFrame>) -> Box<TaskFrame> + 'static,
{
    unsafe {
        let freq_hz = if freq_hz == 0 { 100 } else { freq_hz };

        // Initialize LAPIC abstraction (Maps MMIO if not already done)
        lapic::init(0xFEE00000);

        // Mask legacy PIC
        outb(0x21, 0xFF);
        outb(0xA1, 0xFF);

        calibrate_timer();

        // Calculate reload value based on calibrated frequency
        let initial_count = (TICKS_PER_MS * 1000) / freq_hz;

        lapic::write(
            lapic::REG_LVT_TMR,
            (interrupts::LAPIC_TIMER_VECTOR as u32) | (1 << 17),
        );
        lapic::write(lapic::REG_TDCR, TIMER_DIVISOR);
        lapic::write(lapic::REG_TICRET, initial_count);

        early_console::print("LAPIC Timer initialized!");
        // @TODO: print also the frequency

        HANDLER = Some(Box::new(handler));
    }
}

pub unsafe fn interrupt_handler(frame: *mut InterruptFrame) -> *mut InterruptFrame {
    TICKS.fetch_add(1, Ordering::SeqCst);

    let task_frame = unsafe {
        Box::new(TaskFrame {
            hw_frame: frame,
        })
    };

    let return_frame = unsafe {
        #[allow(static_mut_refs)]
        if let Some(tick_handler) = HANDLER.as_mut() {
            tick_handler(task_frame)
        } else {
            task_frame
        }
    };

    return_frame.hw_frame
}

pub unsafe fn get_ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

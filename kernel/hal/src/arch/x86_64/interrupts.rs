use crate::arch::interface::interrupts::ArchInterrupts;
use core::arch::asm;

pub enum ArchInterruptsImpl {}

impl ArchInterrupts for ArchInterruptsImpl {
    #[inline(always)]
    unsafe fn enable() {
        unsafe {
            asm!("sti");
        }
    }

    #[inline(always)]
    unsafe fn disable() {
        unsafe {
            asm!("cli");
        }
    }

    #[inline(always)]
    unsafe fn are_enabled() -> bool {
        let res: u64;
        unsafe {
            asm!("pushfq", "pop {}", out(reg) res);
            (res & (1 << 9)) != 0
        }
    }
}

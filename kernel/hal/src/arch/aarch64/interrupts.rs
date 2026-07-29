use crate::arch::interface::interrupts::ArchInterrupts;
use core::arch::asm;

pub enum ArchInterruptsImpl {}

impl ArchInterrupts for ArchInterruptsImpl {
    #[inline(always)]
    unsafe fn enable() {
        unsafe {
            asm!("msr daifclr, #3");
        }
    }

    #[inline(always)]
    unsafe fn disable() {
        unsafe {
            asm!("msr daifset, #3", "dmb sy");
        }
    }

    #[inline(always)]
    unsafe fn are_enabled() -> bool {
        let res: u64;
        unsafe {
            asm!("mrs {}, daif", out(reg) res);
            // Bit 7 is the I (IRQ) mask bit.
            // If it is 0, interrupts are NOT masked (enabled).
            (res & (1 << 7)) == 0
        }
    }
}

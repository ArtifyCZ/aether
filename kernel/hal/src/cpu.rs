pub mod interrupts {
    use core::arch::asm;

    pub unsafe fn are_enabled() -> bool {
        unsafe {
            let res: u64;
            #[cfg(target_arch = "x86_64")]
            {
                asm!("pushfq", "pop {}", out(reg) res);
                (res & (1 << 9)) != 0
            }

            #[cfg(target_arch = "aarch64")]
            {
                asm!("mrs {}, daif", out(reg) res);
                // Bit 7 is the I (IRQ) mask bit.
                // If it is 0, interrupts are NOT masked (enabled).
                (res & (1 << 7)) == 0
            }
        }
    }

    pub unsafe fn enable() {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            asm!("sti");

            #[cfg(target_arch = "aarch64")]
            asm!("msr daifclr, #3");
        }
    }

    pub unsafe fn disable() {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            asm!("cli");

            #[cfg(target_arch = "aarch64")]
            asm!("msr daifset, #3", "dmb sy");
        }
    }
}

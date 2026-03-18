use core::arch::asm;

pub unsafe fn hcf() -> ! {
    unsafe {
        loop {
            asm!("msr daifset, #3", "wfi");
        }
    }
}

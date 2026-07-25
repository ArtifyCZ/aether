use core::arch::asm;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hcf() -> ! {
    unsafe {
        loop {
            asm!("msr daifset, #3", "wfi");
        }
    }
}

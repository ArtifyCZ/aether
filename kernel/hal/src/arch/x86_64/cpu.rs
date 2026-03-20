use core::arch::asm;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hcf() -> ! {
    loop {
        asm!("cli", "hlt");
    }
}

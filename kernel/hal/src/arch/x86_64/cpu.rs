use core::arch::asm;

pub unsafe fn hcf() -> ! {
    loop {
        asm!("cli", "hlt");
    }
}

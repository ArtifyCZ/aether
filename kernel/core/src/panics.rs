use core::{
    panic::PanicInfo,
    sync::atomic::{AtomicBool, Ordering},
};

use log::error;

static PANICS_INITIALIZED: AtomicBool = AtomicBool::new(false);

fn hcf() -> ! {
    unsafe {
        #[cfg(target_arch = "aarch64")]
        core::arch::asm!(
            "2:",
            "msr daifset, #0xf",
            "wfi",
            "b 2b",
            options(noreturn),
        );

        #[cfg(target_arch = "x86_64")]
        core::arch::asm!(
            "2:",
            "cli",
            "hlt",
            "jmp 2b",
            options(noreturn),
        );
    }
}

#[panic_handler]
fn panic_handler(info: &PanicInfo<'_>) -> ! {
    if !PANICS_INITIALIZED.load(Ordering::Acquire) {
        // No way to log it
        hcf();
    }

    error!("KERNEL PANIC!\n{}", info);

    hcf();
}

pub fn init() {
    PANICS_INITIALIZED
        .compare_exchange(false, true, Ordering::Release, Ordering::Relaxed)
        .expect("Could not initialize panic handler");
}

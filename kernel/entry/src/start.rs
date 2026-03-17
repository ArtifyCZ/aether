use core::arch::naked_asm;

#[unsafe(no_mangle)]
#[unsafe(naked)]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn _start() -> ! {
    naked_asm!(
        "cli",

        "call {main_func}",

        "3:",
        "cli",
        "hlt",
        "jmp 3b",

        main_func = sym crate::main,
    )
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
#[cfg(target_arch = "aarch64")]
pub unsafe extern "C" fn _start() -> ! {
    naked_asm!(
        "mov x0, sp",
        "bic x0, x0, #0xf", // Force 16-byte alignment

        "ldr x18, =0xDEADD00D",

        "msr spsel, #1",
        "isb",
        "msr daifset, #3",
        "isb",

        "mov sp, x0",
        "bl {main_func}",

        "3:",
        "msr daifset, #3",
        "wfi",
        "b 3b",

        main_func = sym crate::main,
    )
}

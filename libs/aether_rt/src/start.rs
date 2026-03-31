use core::arch::naked_asm;

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn _start() -> ! {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        naked_asm!(
            // Clear the frame pointer so backtraces stop here
            "xor rbp, rbp",
            // Align the stack to 16 bytes (System V ABI requirement)
            // The kernel gave us a stack top, but we ensure it's aligned
            // before we call into complex Rust code.
            "and rsp, -16",
            // The kernel passed the AetherProcInit pointer in RDI.
            // In System V ABI, RDI is the first argument for functions.
            // We just leave it there and jump to our Rust entry point.
            "call {start}",
            // Safety: If start ever returns, we're in trouble, so break.
            ".halt:",
            "int3",
            "jmp .halt",
            start = sym start,
        );
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        naked_asm!(
            "mov x1, sp",
            "bic x1, x1, #0xf", // Force 16-byte alignment
            "mov sp, x1",
            // Clear the Frame Pointer (x29) and Link Register (x30)
            "mov x29, #0",
            "mov x30, #0",
            // The kernel passed the pointer in x0.
            // In AAPCS64, x0 is the first argument.
            "bl {start}",
            // Safety: trap if it returns
            "udf #0",
            start = sym start,
        );
    }
}

unsafe extern "C" {
    fn main(arg: usize) -> !;
}

unsafe extern "C" fn start(arg: usize) -> ! {
    unsafe { main(arg) }
}

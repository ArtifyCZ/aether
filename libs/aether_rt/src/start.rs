use core::arch::naked_asm;

use crate::{process::StartupInfo, stack_allocator};

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn _start(startup_info: *const u8) -> ! {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        naked_asm!(
            // Clear the frame pointer so backtraces stop here
            "xor rbp, rbp",
            "mov rdi, rdi", // startup_info is already in rdi, but this is for clarity
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
            "mov x0, x0", // startup_info is already in x0, but this is for clarity
            "mov x1, sp",
            "and x1, x1, 0xfffffffffffffff0", // Force 16-byte alignment
            "sub x1, x1, #16",
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
    fn main() -> !;
}

unsafe extern "C" fn start(startup_info: *const u8) -> ! {
    unsafe {
        let startup_info = StartupInfo::from_ptr(startup_info).unwrap();
        stack_allocator::init(startup_info.stack_base);

        main();
    }
}

use crate::arch::aarch64::cpu_local;
use crate::arch::aarch64::interrupts::InterruptFrame;
use crate::mmu;
use crate::tasks::TaskFrame;
use alloc::boxed::Box;
use core::arch::asm;
use core::ffi::c_void;

pub unsafe fn setup_user(
    context: usize,
    entrypoint_vaddr: usize,
    user_stack_top: usize,
    kernel_stack_top: usize,
    arg: u64,
) -> Box<TaskFrame> {
    unsafe {
        let sp = kernel_stack_top & !0xF;
        let sp = sp - size_of::<InterruptFrame>();
        let frame_ptr = sp as *mut InterruptFrame;
        core::ptr::write_bytes(frame_ptr, 0, size_of::<InterruptFrame>());

        let frame = frame_ptr.as_mut().unwrap();

        // SPSR_EL1:
        // M[3:0] = 0000 (Return to EL0t)
        // Bit 6,7,8,9 = 0 (Unmask Debug, SError, IRQ, FIQ)
        frame.spsr = 0x00;

        frame.ttbr0 = context as u64;
        frame.elr = entrypoint_vaddr as u64;
        frame.sp_el0 = (user_stack_top & (!0xF)) as u64;
        frame.x[0] = arg;

        Box::new(TaskFrame {
            hw_frame: frame_ptr,
        })
    }
}

pub unsafe fn setup_kernel(
    stack_top: usize,
    f: unsafe extern "C" fn(arg: *mut c_void) -> !,
    arg: *mut c_void,
) -> Box<TaskFrame> {
    unsafe {
        let sp = stack_top & !0xF;
        let sp = sp - size_of::<InterruptFrame>();
        let frame_ptr = sp as *mut InterruptFrame;
        core::ptr::write_bytes(frame_ptr, 0, size_of::<InterruptFrame>());

        let frame = frame_ptr.as_mut().unwrap();

        // SPSR_EL1: M[3:0] = 0101 (Return to EL1h)
        frame.spsr = 0x05 | (0 << 6) | (0 << 7);
        frame.elr = core::mem::transmute::<_, *mut ()>(f) as u64;
        frame.ttbr0 = mmu::get_kernel_context() as u64;

        frame.x[0] = arg as u64;

        Box::new(TaskFrame {
            hw_frame: frame_ptr,
        })
    }
}

pub unsafe fn prepare_switch(kernel_stack_top: usize, task_id: u64) {
    unsafe {
        let _ = kernel_stack_top;
        let mut cpu_local = cpu_local::get().unwrap();
        let cpu_local = cpu_local.as_mut();
        cpu_local.task_id = task_id;
    }
}

pub unsafe fn get_current_id() -> u64 {
    unsafe {
        let cpu_local = cpu_local::get().unwrap();
        let cpu_local = cpu_local.as_ref();
        cpu_local.task_id
    }
}

pub unsafe fn switch_to(task_frame: Box<TaskFrame>) -> ! {
    unsafe {
        let ptr = task_frame.hw_frame;
        asm!(
            // Switch stack to the provided one
            "mov sp, x0",

            // Restore control registers
            "ldr x0, [sp, #248]", // sp_el0
            "ldr x1, [sp, #256]", // ttbr0
            "ldr x2, [sp, #264]", // spsr
            "ldr x3, [sp, #272]", // elr
            "msr sp_el0, x0", // Restore the user stack pointer
            "msr ttbr0_el1, x1", // Switch page tables
            "dsb ish", // Ensure write to ttbr0 is visible
            "tlbi vmalle1is", // Flush all EL1 TLB entries for safety during dev
            "dsb ish", // Data Synchronization Barrier
            "isb", // Instruction Barrier to ensure MMU is ready
            "msr spsr_el1, x2",
            "msr elr_el1, x3",

            // Restore general purpose registers
            "ldp x0, x1, [sp, #0]",
            "ldp x2, x3, [sp, #16]",
            "ldp x4, x5, [sp, #32]",
            "ldp x6, x7, [sp, #48]",
            "ldp x8, x9, [sp, #64]",
            "ldp x10, x11, [sp, #80]",
            "ldp x12, x13, [sp, #96]",
            "ldp x14, x15, [sp, #112]",
            "ldp x16, x17, [sp, #128]",
            "ldp x18, x19, [sp, #144]",
            "ldp x20, x21, [sp, #160]",
            "ldp x22, x23, [sp, #176]",
            "ldp x24, x25, [sp, #192]",
            "ldp x26, x27, [sp, #208]",
            "ldp x28, x29, [sp, #224]",
            "ldr x30, [sp, #240]",

            // Clean up the interrupt frame (the registers) from stack
            "add sp, sp, #288",

            "clrex",
            "eret",

            in("x0") ptr,
            options(noreturn),
        )
    }
}

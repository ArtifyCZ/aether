use crate::arch::x86_64::interrupts::InterruptFrame;
use crate::arch::x86_64::{gdt, msr};
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
        let kernel_stack_top = kernel_stack_top & (!0xF); // 16-byte alignment
        let sp = kernel_stack_top - size_of::<InterruptFrame>();
        let frame_ptr = sp as *mut InterruptFrame;
        core::ptr::write_bytes(frame_ptr, 0, size_of::<InterruptFrame>());

        let frame = frame_ptr.as_mut().unwrap();

        frame.ss = (gdt::USER_DATA_SEGMENT | 3) as u64;
        frame.rsp = (user_stack_top & (!0xF)) as u64;
        // @TODO: disable IO ports and implement an emulated MMIO through a page fault trap
        frame.rflags = 0x202 | (3 << 12); // Interrupts enabled and IO ports allowed
        frame.cs = (gdt::USER_CODE_SEGMENT | 3) as u64; // 0x23
        frame.rip = entrypoint_vaddr as u64;
        // for sysret compatibility
        frame.rcx = entrypoint_vaddr as u64;
        frame.r11 = 0x202;

        frame.cr3 = context as u64;

        frame.rdi = arg;

        Box::new(TaskFrame {
            hw_frame: frame_ptr,
        })
    }
}

pub unsafe fn setup_kernel(
    stack_top: usize,
    f: unsafe extern "C" fn(*mut c_void) -> !,
    arg: *mut c_void,
) -> Box<TaskFrame> {
    unsafe {
        let stack_top = stack_top & (!0xF); // 16-byte alignment
        let sp = stack_top - size_of::<InterruptFrame>();
        let frame_ptr = sp as *mut InterruptFrame;
        core::ptr::write_bytes(frame_ptr, 0, size_of::<InterruptFrame>());

        let frame = frame_ptr.as_mut().unwrap();

        frame.ss = gdt::KERNEL_DATA_SEGMENT as u64;
        frame.rsp = stack_top as u64;
        frame.rflags = 0x202;
        frame.cs = gdt::KERNEL_CODE_SEGMENT as u64;
        frame.rip = core::mem::transmute::<_, *mut ()>(f) as u64;
        frame.rdi = arg as u64;
        frame.cr3 = mmu::get_kernel_context() as u64;

        Box::new(TaskFrame {
            hw_frame: frame_ptr,
        })
    }
}

pub unsafe fn prepare_switch(kernel_stack_top: usize, task_id: u64) {
    unsafe {
        gdt::set_kernel_stack(kernel_stack_top);
        msr::set_kernel_stack(kernel_stack_top);
        msr::set_task_id(task_id);
    }
}

pub unsafe fn get_current_id() -> u64 {
    unsafe { msr::get_task_id() }
}

pub unsafe fn switch_to(task_frame: Box<TaskFrame>) -> ! {
    unsafe {
        let rsp = task_frame.hw_frame;
        asm!(
            // Switch stack to the provided one
            "mov rsp, rax",

            // Switch the page table address
            "pop rax",
            "mov cr3, rax",

            // Restore registers
            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop r11",
            "pop r10",
            "pop r9",
            "pop r8",
            "pop rbp",
            "pop rsi",
            "pop rdi",
            "pop rdx",
            "pop rcx",
            "pop rbx",
            "pop rax",

            // Clean up interrupt vector and error code
            "add rsp, 16",

            "iretq",

            in("rax") rsp,
            options(noreturn),
        ) }
}

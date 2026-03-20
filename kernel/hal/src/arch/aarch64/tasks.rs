use crate::arch::aarch64::cpu_local;
use crate::arch::aarch64::interrupts::InterruptFrame;
use crate::mmu;
use core::ffi::c_void;

#[unsafe(no_mangle)]
unsafe extern "C" fn task_setup_user(
    user_ctx: *const kernel_bindings_gen::vmm_context,
    entrypoint_vaddr: usize,
    user_stack_top: usize,
    kernel_stack_top: usize,
    arg: u64,
) -> *mut kernel_bindings_gen::interrupt_frame {
    unsafe {
        setup_user(
            user_ctx.read().root,
            entrypoint_vaddr,
            user_stack_top,
            kernel_stack_top,
            arg,
        )
        .cast()
    }
}

pub unsafe fn setup_user(
    context: usize,
    entrypoint_vaddr: usize,
    user_stack_top: usize,
    kernel_stack_top: usize,
    arg: u64,
) -> *mut InterruptFrame {
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

        frame_ptr
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn task_setup_kernel(
    stack_top: usize,
    f: kernel_bindings_gen::kernel_task_fn_t,
    arg: *mut c_void,
) -> *mut kernel_bindings_gen::interrupt_frame {
    unsafe { setup_kernel(stack_top, f, arg).cast() }
}

pub unsafe fn setup_kernel(
    stack_top: usize,
    f: kernel_bindings_gen::kernel_task_fn_t,
    arg: *mut c_void,
) -> *mut InterruptFrame {
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

        frame_ptr
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn task_prepare_switch(kernel_stack_top: usize, task_id: u64) {
    unsafe { prepare_switch(kernel_stack_top, task_id) }
}

pub unsafe fn prepare_switch(kernel_stack_top: usize, task_id: u64) {
    unsafe {
        let _ = kernel_stack_top;
        let mut cpu_local = cpu_local::get().unwrap();
        let cpu_local = cpu_local.as_mut();
        cpu_local.task_id = task_id;
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn task_get_current_id() -> u64 {
    unsafe { get_current_id() }
}

pub unsafe fn get_current_id() -> u64 {
    unsafe {
        let cpu_local = cpu_local::get().unwrap();
        let cpu_local = cpu_local.as_ref();
        cpu_local.task_id
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn task_set_syscall_return_value(
    frame: *mut kernel_bindings_gen::interrupt_frame,
    error_code: u64,
    value: u64,
) {
    unsafe {
        let value = if error_code == 0 {
            Ok(value)
        } else {
            Err(error_code)
        };
        set_syscall_return_value(frame.cast(), value);
    }
}

pub unsafe fn set_syscall_return_value(frame: *mut InterruptFrame, value: Result<u64, u64>) {
    unsafe {
        let frame = frame.as_mut().unwrap();
        let (value, error_code) = match value {
            Ok(value) => (value, 0),
            Err(error_code) => (0, error_code),
        };
        frame.x[0] = value;
        frame.x[1] = error_code;
    }
}

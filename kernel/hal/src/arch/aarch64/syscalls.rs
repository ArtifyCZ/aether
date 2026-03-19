use core::arch::asm;
use core::ffi::c_void;
use core::ptr::null_mut;
use crate::arch::aarch64::interrupts::InterruptFrame;

static mut HANDLER: kernel_bindings_gen::syscall_handler_t = None;
static mut HANDLER_ARG: *mut c_void = null_mut();

#[unsafe(no_mangle)]
unsafe extern "C" fn syscalls_init(
    handler: kernel_bindings_gen::syscall_handler_t,
    handler_arg: *mut c_void,
) {
    unsafe {
        HANDLER = handler;
        HANDLER_ARG = handler_arg;
    }
}

pub unsafe fn interrupt_handler(frame: *mut *mut InterruptFrame) {
    let (num, a) = unsafe {
        let frame = frame.read().as_ref().unwrap();
        (frame.x[8], [frame.x[0], frame.x[1], frame.x[2], frame.x[3], frame.x[4]])
    };
    let interrupt_frame = frame.cast();
    let mut syscall_frame = kernel_bindings_gen::syscall_frame {
        interrupt_frame,
        num,
        a,
    };
    unsafe {
        if let Some(handler) = HANDLER {
            handler(&raw mut syscall_frame, HANDLER_ARG);
            frame.write(syscall_frame.interrupt_frame.read().cast());
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn syscalls_raw(args: kernel_bindings_gen::syscall_args) -> u64 {
    unsafe { raw(args.num, args.a).unwrap_or(0) }
}

pub unsafe fn raw(num: u64, args: [u64; 5]) -> Result<u64, u64> {
    unsafe {
        let ret: u64;
        let error_code: u64;
        asm!(
            "svc #0",
            in("x8") num,
            inout("x0") args[0] => ret,
            inout("x1") args[1] => error_code,
            in("x2") args[2],
            in("x3") args[3],
            in("x4") args[4],
        );
        if error_code == 0 {
            Ok(ret)
        } else {
            Err(error_code)
        }
    }
}

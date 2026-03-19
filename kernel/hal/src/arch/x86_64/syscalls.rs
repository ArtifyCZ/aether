use crate::arch::x86_64::interrupts::InterruptFrame;
use core::arch::asm;
use core::ffi::c_void;
use core::ptr::null_mut;

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
#[unsafe(no_mangle)]
unsafe extern "C" fn syscalls_inner_handler(frame: *mut InterruptFrame) -> usize {
    let (num, a) = unsafe {
        let frame = frame.as_ref().unwrap();
        (
            frame.rax,
            [frame.rdi, frame.rsi, frame.rdx, frame.r10, frame.r8],
        )
    };
    let mut return_frame = frame;
    let interrupt_frame = (&mut return_frame as *mut *mut InterruptFrame).cast();
    let mut sf = kernel_bindings_gen::syscall_frame {
        interrupt_frame,
        num,
        a,
    };

    unsafe {
        if let Some(handler) = HANDLER {
            handler(&raw mut sf, HANDLER_ARG);
        }
    }

    return_frame as usize
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
            "syscall",
            inout("rax") num => ret,
            in("rdi") args[0],
            in("rsi") args[1],
            inout("rdx") args[2] => error_code,
            in("r10") args[3],
            in("r8") args[4],
        );
        if error_code == 0 {
            Ok(ret)
        } else {
            Err(error_code)
        }
    }
}

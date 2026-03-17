use crate::platform::tasks::TaskFrame;
use alloc::boxed::Box;
use core::arch::asm;
use kernel_bindings_gen::{
    interrupt_frame, interrupts_init, interrupts_mask_irq, interrupts_set_irq_handler,
    interrupts_unmask_irq,
};

pub struct Interrupts;

impl Interrupts {
    pub unsafe fn init() {
        unsafe {
            interrupts_init();
        }
    }

    pub unsafe fn set_irq_handler<F>(f: F)
    where
        F: FnMut(TaskFrame, u8) -> TaskFrame + 'static,
    {
        unsafe extern "C" fn trampoline<F>(
            frame: *mut *mut interrupt_frame,
            irq: u8,
            arg: *mut core::ffi::c_void,
        ) -> bool
        where
            F: FnMut(TaskFrame, u8) -> TaskFrame + 'static,
        {
            unsafe {
                let f: &mut F = &mut *arg.cast();
                let prev_frame: *mut interrupt_frame = frame.read();
                let prev_frame: TaskFrame = TaskFrame(prev_frame.cast());
                let next_frame: TaskFrame = f(prev_frame, irq);
                let next_frame: *mut interrupt_frame = next_frame.0.cast();
                frame.write(next_frame);

                true
            }
        }

        let f = Box::into_raw(Box::new(f));
        unsafe {
            interrupts_set_irq_handler(Some(trampoline::<F>), f as *mut _);
        }
    }

    pub unsafe fn mask_irq(irq: u8) {
        unsafe {
            interrupts_mask_irq(irq);
        }
    }

    pub unsafe fn unmask_irq(irq: u8) {
        unsafe {
            interrupts_unmask_irq(irq);
        }
    }

    pub unsafe fn are_enabled() -> bool {
        unsafe {
            let res: u64;
            #[cfg(target_arch = "x86_64")]
            {
                asm!("pushfq", "pop {}", out(reg) res);
                (res & (1 << 9)) != 0
            }
            #[cfg(target_arch = "aarch64")]
            {
                asm!("mrs {}, daif", out(reg) res);
                // Bit 7 is the I (IRQ) mask bit.
                // If it is 0, interrupts are NOT masked (enabled).
                (res & (1 << 7)) == 0
            }
        }
    }

    pub unsafe fn enable() {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            asm!("sti");
            #[cfg(target_arch = "aarch64")]
            asm!("msr daifclr, #3");
        }
    }

    pub unsafe fn disable() {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            asm!("cli");
            #[cfg(target_arch = "aarch64")]
            asm!(
                "msr daifset, #3",
                "dmb sy",
            );
        }
    }
}

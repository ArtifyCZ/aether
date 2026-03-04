use crate::platform::tasks::TaskFrame;
use alloc::boxed::Box;

mod bindings {
    include_bindings!("interrupts.rs");
}

pub struct Interrupts;

impl Interrupts {
    pub unsafe fn init() {
        unsafe {
            bindings::interrupts_init();
        }
    }

    pub unsafe fn set_irq_handler<F>(f: F)
    where
        F: FnMut(TaskFrame, u8) -> TaskFrame + 'static,
    {
        unsafe extern "C" fn trampoline<F>(
            frame: *mut *mut bindings::interrupt_frame,
            irq: u8,
            arg: *mut core::ffi::c_void,
        ) -> bool
        where
            F: FnMut(TaskFrame, u8) -> TaskFrame + 'static,
        {
            unsafe {
                let f: &mut F = &mut *arg.cast();
                let prev_frame: *mut bindings::interrupt_frame = frame.read();
                let prev_frame: TaskFrame = TaskFrame(prev_frame.cast());
                let next_frame: TaskFrame = f(prev_frame, irq);
                let next_frame: *mut bindings::interrupt_frame = next_frame.0.cast();
                frame.write(next_frame);

                true
            }
        }

        let f = Box::into_raw(Box::new(f));
        unsafe {
            bindings::interrupts_set_irq_handler(Some(trampoline::<F>), f as *mut _);
        }
    }

    pub unsafe fn mask_irq(irq: u8) {
        unsafe {
            bindings::interrupts_mask_irq(irq);
        }
    }

    pub unsafe fn unmask_irq(irq: u8) {
        unsafe {
            bindings::interrupts_unmask_irq(irq);
        }
    }

    pub unsafe fn enable() {
        unsafe {
            bindings::interrupts_enable();
        }
    }

    pub unsafe fn disable() {
        unsafe {
            bindings::interrupts_disable();
        }
    }
}

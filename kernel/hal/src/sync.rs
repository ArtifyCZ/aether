use crate::arch::current::interrupts::ArchInterruptsImpl;
use crate::arch::interface::interrupts::ArchInterrupts;

pub type Spinlock<T> = atom_primitives::sync::Spinlock<T, InterruptControlImpl>;

pub enum InterruptControlImpl {}

impl atom_primitives::sync::InterruptControl for InterruptControlImpl {
    #[inline(always)]
    unsafe fn enable() {
        unsafe {
            ArchInterruptsImpl::enable();
        }
    }

    #[inline(always)]
    unsafe fn disable() -> bool {
        unsafe {
            let were_enabled = ArchInterruptsImpl::are_enabled();
            ArchInterruptsImpl::disable();
            were_enabled
        }
    }
}

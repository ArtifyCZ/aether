use crate::arch::interrupts;

pub(crate) use interrupts::InterruptFrame;

/// Returns the architecture-specific interrupt vector table descriptor.
///
/// - **x86_64**: `(IDT base address, IDT limit)` — limit is 4095 for a full 256-entry IDT.
/// - **aarch64**: `(VBAR_EL1 current value, exception_vector_table address)` — both values
///   must be equal after a successful `interrupts_init()`.
pub unsafe fn read_vector_table_info() -> (usize, usize) {
    unsafe { arch::interrupts::read_vector_table_info() }
}

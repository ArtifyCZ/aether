mod interrupts;
mod memory;

/// Run the early kernel self-tests.
///
/// Covers physical memory management, virtual memory mapping, and interrupt
/// vector table setup.  Must be called after `Interrupts::init()` and
/// `switch_to_paged_allocator()`.
pub unsafe fn run_early() {
    println!("[self_test] ===== Running early kernel self-tests =====");
    unsafe {
        memory::run();
        interrupts::run_vector_table_check();
    }
    println!("[self_test] ===== Early self-tests passed =====");
}

/// Run the interrupt-delivery self-test.
///
/// - **x86_64**: fires a software `INT 0x80` through the IDT and verifies the
///   IRQ handler is invoked.  Must be called after `Ticker::init()` (so that
///   the LAPIC is mapped) and after `Interrupts::enable()`.
/// - **aarch64**: spins until at least one timer-driven IRQ exception is
///   observed, exercising the full GIC → `handle_irq_exception` path.  Must
///   be called after `Ticker::init()` and `Interrupts::enable()`.
///
/// The production IRQ handler (`Interrupts::set_irq_handler`) **must not**
/// have been called yet when this function is invoked on x86_64; the function
/// temporarily installs a test handler and it is then replaced by the caller.
pub unsafe fn run_interrupt_delivery() {
    println!("[self_test] ===== Testing interrupt delivery =====");
    unsafe {
        interrupts::run_delivery_check();
    }
    println!("[self_test] ===== Interrupt delivery test passed =====");
}

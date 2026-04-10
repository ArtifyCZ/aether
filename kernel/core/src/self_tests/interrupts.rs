use crate::println;

// ──────────────────────────────────────────────────────────────────────────────
// Static check: interrupt vector table is loaded and points at the right place
// ──────────────────────────────────────────────────────────────────────────────

/// Verify the CPU interrupt / exception vector table is correctly installed.
///
/// - **x86_64**: reads the IDTR via `sidt` and asserts the limit is 4095
///   (256 entries × 16 bytes − 1) and the base is non-null.
/// - **aarch64**: reads `VBAR_EL1` and asserts it equals the address of the
///   compiled `exception_vector_table` symbol.
pub unsafe fn run_vector_table_check() {
    println!("[self_test] --- Interrupt vector table check ---");
    let (a, b) = unsafe { kernel_hal::interrupts::read_vector_table_info() };

    #[cfg(target_arch = "x86_64")]
    {
        let base = a;
        let limit = b;
        assert_ne!(base, 0, "[self_test] IDT: base address must not be zero");
        assert_eq!(
            limit,
            4095,
            "[self_test] IDT: limit must be 4095 (256 entries × 16 bytes − 1), got {}",
            limit
        );
        println!(
            "[self_test] IDT check: PASS (base={:#x}, limit={})",
            base, limit
        );
    }

    #[cfg(target_arch = "aarch64")]
    {
        let vbar = a;
        let expected = b;
        assert_eq!(
            vbar,
            expected,
            "[self_test] VBAR_EL1 ({:#x}) must equal exception_vector_table ({:#x})",
            vbar,
            expected
        );
        println!("[self_test] VBAR_EL1 check: PASS ({:#x})", vbar);
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Dynamic check: an actual interrupt is delivered end-to-end
// ──────────────────────────────────────────────────────────────────────────────

/// Verify an interrupt is delivered end-to-end through the hardware path.
///
/// Must be called **after** `Ticker::init()` and `Interrupts::enable()`, and
/// **before** `Interrupts::set_irq_handler()` is called with the production
/// scheduler handler (x86_64 only — see module-level docs).
pub unsafe fn run_delivery_check() {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        run_delivery_check_x86_64();
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        run_delivery_check_aarch64();
    }
}

// ── x86_64 ────────────────────────────────────────────────────────────────────

/// Fire a software `INT 0x80` and verify the IRQ handler is called.
///
/// `INT 0x80` maps to IRQ number `0x80 − IRQ_INTERRUPT_VECTOR_OFFSET(0x30) = 0x50`
/// inside `x86_64_interrupt_dispatcher`.  The test installs a one-shot handler
/// via `Interrupts::set_irq_handler`, fires the software interrupt, and asserts
/// the static flag was set by the handler.
///
/// The test handler's heap allocation is intentionally leaked: the caller must
/// install the production handler with a second `Interrupts::set_irq_handler`
/// call immediately afterwards, which overwrites the stored function pointer.
#[cfg(target_arch = "x86_64")]
unsafe fn run_delivery_check_x86_64() {
    use crate::platform::interrupts::Interrupts;
    use core::sync::atomic::{AtomicBool, Ordering};

    println!("[self_test] --- x86_64 IRQ delivery check (INT 0x80) ---");

    static TEST_IRQ_FIRED: AtomicBool = AtomicBool::new(false);

    // INT 0x80 → dispatcher receives interrupt_vector = 0x80
    // irq = 0x80 − IRQ_INTERRUPT_VECTOR_OFFSET (0x30) = 0x50
    const TEST_IRQ_NUM: u8 = 0x80 - 0x30;

    TEST_IRQ_FIRED.store(false, core::sync::atomic::Ordering::SeqCst);

    unsafe {
        Interrupts::set_irq_handler(|frame, irq| {
            if irq == TEST_IRQ_NUM {
                TEST_IRQ_FIRED.store(true, Ordering::SeqCst);
            }
            frame
        });

        // Software interrupt — processed synchronously regardless of IF flag.
        core::arch::asm!("int 0x80");
    }

    assert!(
        TEST_IRQ_FIRED.load(core::sync::atomic::Ordering::SeqCst),
        "[self_test] x86_64 IRQ delivery: INT 0x80 was not caught by the IRQ handler"
    );
    println!("[self_test] x86_64 IRQ delivery (INT 0x80 → irq 0x50): PASS");
}

// ── aarch64 ───────────────────────────────────────────────────────────────────

/// Spin until the GIC delivers at least one virtual-timer IRQ exception and the
/// `Ticker` handler increments the tick counter.
///
/// This exercises the complete path:
/// `CNTV_TVAL` expiry → GIC INTID 0x1B → `handle_irq_exception` →
/// `timer::interrupt_handler` → `Ticker::tick_handler`.
///
/// Times out after ~10 M iterations (well above the 10 ms period at 100 Hz).
#[cfg(target_arch = "aarch64")]
unsafe fn run_delivery_check_aarch64() {
    println!("[self_test] --- aarch64 IRQ delivery check (timer tick via GIC) ---");

    let t0 = kernel_hal::timer::get_ticks();

    let mut caught = false;
    for _ in 0..10_000_000_u64 {
        if kernel_hal::timer::get_ticks() != t0 {
            caught = true;
            break;
        }
        core::hint::spin_loop();
    }

    assert!(
        caught,
        "[self_test] aarch64 IRQ delivery: no timer tick observed within timeout \
         (IRQ exception not delivered through GIC → handle_irq_exception)"
    );
    println!("[self_test] aarch64 IRQ delivery (GIC timer tick): PASS");
}

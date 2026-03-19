use super::ioapic;
use crate::arch::x86_64::{lapic, timer};
use core::arch::asm;
use core::ffi::c_void;
use core::mem::zeroed;
use core::ptr::null_mut;
use kernel_bindings_gen::irq_handler_new_t;

#[repr(C, packed)]
pub struct InterruptFrame {
    cr3: u64, // Pushed LAST in ASM (the lowest address)
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    pub(crate) r10: u64,
    r9: u64,
    pub(crate) r8: u64,
    rbp: u64,
    pub(crate) rsi: u64,
    pub(crate) rdi: u64,
    pub(crate) rdx: u64,
    rcx: u64,
    rbx: u64,
    pub(crate) rax: u64,
    // ^ rax was pushed FIRST among these, so it is at the highest address here
    interrupt_vector: u64,
    error_code: u64,
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

const IRQ_INTERRUPT_VECTOR_OFFSET: u64 = 0x30;
const LAPIC_TIMER_IRQ: u64 = 0;
pub(super) const LAPIC_TIMER_VECTOR: u64 = IRQ_INTERRUPT_VECTOR_OFFSET + LAPIC_TIMER_IRQ;

// extern uint64_t interrupt_stubs[]
unsafe extern "C" {
    static interrupt_stubs: [u64; 256];
}

#[repr(C, packed)]
struct IdtEntry {
    isr_low: u16,
    kernel_cs: u16,
    ist: u8,
    attributes: u8,
    isr_mid: u16,
    isr_high: u32,
    reserved: u32,
}

#[repr(C, packed)]
struct IdtPtr {
    limit: u16,
    base: u64,
}

static mut IDT: [IdtEntry; 256] = unsafe { zeroed() };

static mut IRQ_HANDLER: irq_handler_new_t = None;
static mut IRQ_HANDLER_ARG: *mut c_void = null_mut();

const KERNEL_CODE_SEGMENT: u16 = 0x08;

#[unsafe(no_mangle)]
unsafe extern "C" fn interrupts_init() {
    unsafe {
        init();
    }
}

#[allow(static_mut_refs)]
pub unsafe fn init() {
    unsafe {
        IRQ_HANDLER = None;
        IRQ_HANDLER_ARG = null_mut();

        for i in 0..IDT.len() {
            let isr: u64 = interrupt_stubs[i];
            let idt = &mut IDT[i];
            idt.isr_low = (isr & 0xFFFF) as u16;
            idt.kernel_cs = KERNEL_CODE_SEGMENT;
            idt.ist = if i == 0x0E {
                // Page fault
                // Use IST1 stack
                1
            } else {
                // No specific stack switching
                0
            };
            idt.attributes = 0x8E; // Present, Ring 0, Interrupt Gate
            idt.isr_mid = ((isr >> 16) & 0xFFFF) as u16;
            idt.isr_high = ((isr >> 32) & 0xFFFFFFFF) as u32;
            idt.reserved = 0;
        }

        let limit = (size_of_val(&IDT) - 1) as u16;
        assert_eq!(limit, 4095);

        let ptr = IdtPtr {
            limit,
            base: &IDT as *const _ as u64,
        };
        assert_eq!(size_of::<IdtPtr>(), 10);

        asm!("lidt [{}]", in(reg) &ptr);
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn interrupts_set_irq_handler(handler: irq_handler_new_t, arg: *mut c_void) {
    IRQ_HANDLER = handler;
    IRQ_HANDLER_ARG = arg;
}

#[unsafe(no_mangle)]
unsafe extern "C" fn interrupts_mask_irq(irq: u8) {
    unsafe {
        ioapic::set_mask(irq, true);
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn interrupts_unmask_irq(irq: u8) {
    unsafe {
        ioapic::set_mask(irq, false);
        ioapic::set_entry(irq, irq as u32 + IRQ_INTERRUPT_VECTOR_OFFSET as u32);
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn x86_64_interrupt_dispatcher(
    frame: *mut InterruptFrame,
) -> usize {
    let f = unsafe { &*frame };
    let error_code = f.error_code;
    let interrupt_vector = f.interrupt_vector;
    if error_code != 0 || interrupt_vector < 0x20 {
        // non-zero error code and/or exceptions
        // @TODO: dump the interrupt vector, received error code, cr2, stack and possibly other vars
        panic!("Unexpected/unhandled interrupt");
    }

    let mut return_frame: *mut InterruptFrame = frame;

    if interrupt_vector < IRQ_INTERRUPT_VECTOR_OFFSET {
        // @TODO: dump interrupt vector
        panic!("Legacy PIC interrupt");
    }

    let irq = interrupt_vector - IRQ_INTERRUPT_VECTOR_OFFSET;
    match irq {
        LAPIC_TIMER_IRQ => timer::interrupt_handler(&raw mut return_frame),
        _ => {
            if let Some(irq_handler) = IRQ_HANDLER {
                irq_handler((&mut return_frame as *mut *mut InterruptFrame).cast(), irq as u8, IRQ_HANDLER_ARG);
            }
        }
    };
    if IRQ_INTERRUPT_VECTOR_OFFSET <= interrupt_vector && interrupt_vector <= 0xFF {
        lapic::send_eoi();
    }

    return_frame as usize
}

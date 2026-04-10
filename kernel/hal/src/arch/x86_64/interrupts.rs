use super::ioapic;
use crate::arch::x86_64::{lapic, timer};
use core::arch::asm;
use core::ffi::c_void;
use core::fmt;
use core::mem::zeroed;
use core::ptr::null_mut;
use kernel_bindings_gen::irq_handler_new_t;

#[repr(C, align(16))]
pub(crate) struct InterruptFrame {
    pub(crate) cr3: u64, // Pushed LAST in ASM (the lowest address)
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    pub(crate) r11: u64,
    pub(crate) r10: u64,
    r9: u64,
    pub(crate) r8: u64,
    rbp: u64,
    pub(crate) rsi: u64,
    pub(crate) rdi: u64,
    pub(crate) rdx: u64,
    pub(crate) rcx: u64,
    rbx: u64,
    pub(crate) rax: u64,
    // ^ rax was pushed FIRST among these, so it is at the highest address here
    interrupt_vector: u64,
    error_code: u64,
    pub(crate) rip: u64,
    pub(crate) cs: u64,
    pub(crate) rflags: u64,
    pub(crate) rsp: u64,
    pub(crate) ss: u64,
}

impl fmt::Debug for InterruptFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InterruptFrame")
            .field("rip", &format_args!("0x{:016x}", self.rip))
            .field("rsp", &format_args!("0x{:016x}", self.rsp))
            .field("rax", &format_args!("0x{:016x}", self.rax))
            .field("rbx", &format_args!("0x{:016x}", self.rbx))
            .field("rcx", &format_args!("0x{:016x}", self.rcx))
            .field("rdx", &format_args!("0x{:016x}", self.rdx))
            .field("rdi", &format_args!("0x{:016x}", self.rdi))
            .field("rsi", &format_args!("0x{:016x}", self.rsi))
            .field("rbp", &format_args!("0x{:016x}", self.rbp))
            .field("r8", &format_args!("0x{:016x}", self.r8))
            .field("r9", &format_args!("0x{:016x}", self.r9))
            .field("r10", &format_args!("0x{:016x}", self.r10))
            .field("r11", &format_args!("0x{:016x}", self.r11))
            .field("r12", &format_args!("0x{:016x}", self.r12))
            .field("r13", &format_args!("0x{:016x}", self.r13))
            .field("r14", &format_args!("0x{:016x}", self.r14))
            .field("r15", &format_args!("0x{:016x}", self.r15))
            .field("err", &format_args!("0x{:x}", self.error_code))
            .field("num", &format_args!("0x{:x}", self.interrupt_vector))
            .field("flg", &format_args!("0x{:016x}", self.rflags))
            .finish()
    }
}

impl InterruptFrame {
    pub(crate) unsafe fn set_syscall_return_value(&mut self, value: Result<u64, u64>) {
        let (value, error_code) = match value {
            Ok(value) => (value, 0),
            Err(error_code) => (0, error_code),
        };

        self.rax = value;
        self.rdx = error_code;
    }
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
unsafe extern "C" fn x86_64_interrupt_dispatcher(frame: *mut InterruptFrame) -> usize {
    let f = unsafe { &*frame };
    let error_code = f.error_code;
    let interrupt_vector = f.interrupt_vector;
    if error_code != 0 || interrupt_vector < 0x20 {
        let cr2: u64;
        asm!("mov {}, cr2", out(reg) cr2);

        // Raw Serial Dump
        unsafe {
            crate::emergency_console::print("\n!! EXCEPTION ");
            crate::emergency_console::put_hex(interrupt_vector);
            crate::emergency_console::print(" !!\nRIP: ");
            crate::emergency_console::put_hex(f.rip);
            crate::emergency_console::print("\nCR2: ");
            crate::emergency_console::put_hex(cr2);
            crate::emergency_console::print("\nERR: ");
            crate::emergency_console::put_hex(f.error_code);
            crate::emergency_console::print("\n");
            panic!("CPU Exception {:#?}", frame.as_ref());
        }
    }

    let mut return_frame: *mut InterruptFrame = frame;

    if interrupt_vector < IRQ_INTERRUPT_VECTOR_OFFSET {
        // @TODO: dump interrupt vector
        panic!("Legacy PIC interrupt");
    }

    let irq = interrupt_vector - IRQ_INTERRUPT_VECTOR_OFFSET;
    match irq {
        LAPIC_TIMER_IRQ => {
            return_frame = timer::interrupt_handler(frame);
        }
        _ => {
            if let Some(irq_handler) = IRQ_HANDLER {
                irq_handler(
                    (&mut return_frame as *mut *mut InterruptFrame).cast(),
                    irq as u8,
                    IRQ_HANDLER_ARG,
                );
            }
        }
    };
    if IRQ_INTERRUPT_VECTOR_OFFSET <= interrupt_vector && interrupt_vector <= 0xFF {
        lapic::send_eoi();
    }

    return_frame as usize
}

/// Returns `(IDT base address, IDT limit)` read directly from the CPU via `sidt`.
///
/// The limit for a fully-populated 256-entry IDT is 4095 (256 × 16 − 1).
/// Used by kernel self-tests to verify the IDT has been loaded correctly.
pub unsafe fn read_vector_table_info() -> (usize, usize) {
    #[repr(C, packed)]
    struct IdtDescriptor {
        limit: u16,
        base: u64,
    }
    let mut desc = IdtDescriptor { limit: 0, base: 0 };
    unsafe {
        core::arch::asm!("sidt [{}]", in(reg) &mut desc);
    }
    // Use unaligned reads to avoid UB on packed-struct fields.
    let base = unsafe { core::ptr::addr_of!(desc.base).read_unaligned() } as usize;
    let limit = unsafe { core::ptr::addr_of!(desc.limit).read_unaligned() } as usize;
    (base, limit)
}

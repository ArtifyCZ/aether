use alloc::string::ToString;
use super::gic;
use crate::arch::aarch64::timer;
use crate::{early_console, emergency_console};
use core::arch::asm;
use core::ffi::c_void;
use core::ptr::null_mut;
use kernel_bindings_gen::irq_handler_new_t;

// /**
//  * AArch64 ESR_EL1 Exception Class (EC) definitions
//  */
// #define EC_UNKNOWN          0x00
// #define EC_SIMD_FP          0x07
// #define EC_SYSCALL          0x15
// #define EC_DATA_ABORT_LOWER 0x24
// #define EC_DATA_ABORT_SAME  0x25
// #define EC_INST_ABORT_LOWER 0x20
// #define EC_INST_ABORT_SAME  0x21
// #define EC_ALIGNED_FAULT    0x26

const EC_SYSCALL: u32 = 0x15;

// 0x00-0x0F are used for software-generated interrupts
// 0x10-0x1F are used for private-peripheral interrupts
// 0x20+ are used for shared peripheral interrupts
const IRQ_INTID_OFFSET: u32 = 0x20;

#[repr(C, align(16))]
pub struct InterruptFrame {
    x: [u64; 31],
    sp_el0: u64,
    ttbr0: u64,
    spsr: u64,
    elr: u64,
    esr: u64,
}

unsafe extern "C" {
    // SIZE UNKNOWN IN RUST, KNOWN IN ASSEMBLY
    static exception_vector_table: [c_void; 1];
}

static mut IRQ_HANDLER: irq_handler_new_t = None;
static mut IRQ_HANDLER_ARG: *mut c_void = null_mut();

pub const INTID_TIMER: u32 = 0x1B;

#[unsafe(no_mangle)]
unsafe extern "C" fn interrupts_init() {
    unsafe {
        init();
    }
}

#[allow(static_mut_refs)]
pub unsafe fn init() {
    unsafe {
        // Install the vector table
        let vbar: usize = &exception_vector_table as *const _ as usize;
        asm!("msr vbar_el1, {}", in(reg) vbar);

        // Enable Floating Point/SIMD (to prevent CPACR traps)
        let mut cpacr: u64;
        asm!("mrs {}, cpacr_el1", out(reg) cpacr);
        cpacr |= (3 << 20);
        asm!("msr cpacr_el1, {}", in(reg) cpacr);

        gic::init();
        IRQ_HANDLER = None;
        IRQ_HANDLER_ARG = null_mut();

        early_console::print("Interrupts initialized!");
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
        gic::mask_vector(irq as u32 + IRQ_INTID_OFFSET);
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn interrupts_unmask_irq(irq: u8) {
    unsafe {
        gic::unmask_vector(irq as u32 + IRQ_INTID_OFFSET);
    }
}

unsafe extern "C" {
    fn syscalls_interrupt_handler(frame: *mut *mut InterruptFrame) -> bool;
}

#[unsafe(no_mangle)]
unsafe extern "C" fn handle_sync_exception(frame: *mut InterruptFrame) -> usize {
    unsafe {
        let ec = (frame.read().esr >> 26) & 0x3F;

        if (ec == EC_SYSCALL as u64) {
            let mut return_frame = frame;
            syscalls_interrupt_handler(&raw mut return_frame);
            return return_frame as usize;
        }

        // @TODO: print task id, ESR_EL1, ELR_EL1 (PC), FAR_EL1 (Addr), SP_EL0 (Addr) and reason
        emergency_console::print("Synchronous abort!");
        let str = ec.to_string();
        emergency_console::print(&str);
        panic!("Synchronous abort!");
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn handle_irq_exception(frame: *mut InterruptFrame) -> usize {
    unsafe {
        let intid = gic::acknowledge_interrupt();

        let mut return_frame: *mut InterruptFrame = frame;

        match intid {
            INTID_TIMER => timer::interrupt_handler(&mut return_frame as *mut *mut InterruptFrame),
            IRQ_INTID_OFFSET..=0xFF => {
                if let Some(irq_handler) = IRQ_HANDLER {
                    let irq = intid - IRQ_INTID_OFFSET;
                    irq_handler(
                        (&mut return_frame as *mut *mut InterruptFrame).cast(),
                        irq as u8,
                        IRQ_HANDLER_ARG,
                    );
                }
            }
            _ => todo!(),
        }

        gic::send_eoi(intid);

        return_frame as usize
    }
}

// Fast Interrupts (FIQ) - Usually reserved for secure monitor or high-priority tasks
#[unsafe(no_mangle)]
unsafe extern "C" fn handle_fiq_exception(frame: *mut InterruptFrame) {
    todo!()
}

// System Errors (SERROR) - Usually asynchronous hardware errors (bad bus access)
#[unsafe(no_mangle)]
unsafe extern "C" fn handle_serror_exception(frame: *mut InterruptFrame) {
    todo!()
}

use crate::mmu;
use core::ptr::null_mut;
use kernel_bindings_gen::VMM_PAGE_SIZE;
use mmu::VirtualMemoryMappingFlags;

// Offsets - simplified for byte-addressing in the internal implementation
const REG_EOI: u32 = 0x0B0;
pub(crate) const REG_SVR: u32 = 0x0F0;
pub(crate) const REG_LVT_TMR: u32 = 0x320;
pub(crate) const REG_TICRET: u32 = 0x380;
pub(crate) const REG_TCCR: u32 = 0x390;
pub(crate) const REG_TDCR: u32 = 0x3E0;

static mut LAPIC_BASE: *mut u32 = null_mut();

pub unsafe fn init(phys_addr: usize) {
    unsafe {
        if LAPIC_BASE.is_null() {
            let virt = kernel_bindings_gen::vaa_alloc_range(VMM_PAGE_SIZE as usize);
            let kernel_context = mmu::get_kernel_context();
            mmu::map_page(
                kernel_context,
                virt,
                phys_addr,
                VirtualMemoryMappingFlags::PRESENT
                    | VirtualMemoryMappingFlags::WRITE
                    | VirtualMemoryMappingFlags::DEVICE,
            );
            LAPIC_BASE = virt as *mut u32;
        }
    }
}

/// Sends EOI (end of interrupt) to the Local APIC
pub unsafe fn send_eoi() {
    unsafe {
        write(REG_EOI, 0);
    }
}

pub unsafe fn read(reg: u32) -> u32 {
    unsafe {
        LAPIC_BASE.byte_add(reg as usize).read_volatile()
    }
}

pub unsafe fn write(reg: u32, value: u32) {
    unsafe {
        LAPIC_BASE.byte_add(reg as usize).write_volatile(value);
    }
}

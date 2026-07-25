use crate::mmu::VirtualMemoryMappingFlags;
use core::ptr::null_mut;
use kernel_bindings_gen::VMM_PAGE_SIZE;

static mut IOAPIC_PTR: *mut u32 = null_mut();

pub unsafe fn init(phys_addr: usize) {
    unsafe {
        let virt = kernel_bindings_gen::vaa_alloc_range(VMM_PAGE_SIZE as usize);

        let kernel_context = crate::mmu::get_kernel_context();
        crate::mmu::map_page(
            kernel_context,
            virt,
            phys_addr,
            VirtualMemoryMappingFlags::PRESENT
                | VirtualMemoryMappingFlags::WRITE
                | VirtualMemoryMappingFlags::DEVICE,
        );

        IOAPIC_PTR = virt as *mut u32;
    }
}

pub unsafe fn read(reg: u8) -> u32 {
    unsafe {
        IOAPIC_PTR.add(0).write_volatile(reg as u32);
        IOAPIC_PTR.add(4).read_volatile()
    }
}

pub unsafe fn write(reg: u8, value: u32) {
    unsafe {
        IOAPIC_PTR.add(0).write_volatile(reg as u32);
        IOAPIC_PTR.add(4).write_volatile(value);
    }
}

pub unsafe fn set_entry(pin: u8, vector: u32) {
    unsafe {
        let low_index = 0x10 + (pin * 2);
        let high_index = low_index + 1;

        // High 32 bits: Destination ID 0 in the top 8 bits
        write(high_index, 0x00000000);

        // Low 32 bits: Vector, and unmask (bit 16 = 0)
        // For ISA IRQs (like keyboard), they are usually Active High / Edge Triggered
        // So bits 13 and 15 stay 0.
        write(low_index, vector);
    }
}

pub unsafe fn set_mask(pin: u8, mask: bool) {
    unsafe {
        // The Redirection Table entry is 64 bits (two 32-bit registers)
        // Register 0x10 + (pin * 2) is the LOW 32 bits.
        // Bit 16 (Mask) is in the LOW 32 bits.

        let low_index = 0x10 + (pin * 2);

        let mut val = read(low_index);
        if mask {
            val |= (1 << 16); // Set bit 16 to MASK
        } else {
            val &= !(1 << 16); // Clear bit 16 to UNMASK
        }

        write(low_index, val);
    }
}

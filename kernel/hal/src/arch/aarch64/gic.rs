use crate::mmu;
use core::ptr::null_mut;
use mmu::VirtualMemoryMappingFlags;

static mut DIST_BASE: *mut u32 = null_mut();
static mut CPU_BASE: *mut u32 = null_mut();

pub unsafe fn init() {
    unsafe {
        let dist_base =
            kernel_bindings_gen::vaa_alloc_range(kernel_bindings_gen::VMM_PAGE_SIZE as usize);
        let cpu_base =
            kernel_bindings_gen::vaa_alloc_range(kernel_bindings_gen::VMM_PAGE_SIZE as usize);

        let kernel_context = mmu::get_kernel_context();
        mmu::map_page(
            kernel_context,
            dist_base,
            0x08000000,
            VirtualMemoryMappingFlags::PRESENT
                | VirtualMemoryMappingFlags::WRITE
                | VirtualMemoryMappingFlags::DEVICE,
        );
        mmu::map_page(
            kernel_context,
            cpu_base,
            0x08010000,
            VirtualMemoryMappingFlags::PRESENT
                | VirtualMemoryMappingFlags::WRITE
                | VirtualMemoryMappingFlags::DEVICE,
        );

        DIST_BASE = dist_base as *mut u32;
        CPU_BASE = cpu_base as *mut u32;

        // Disable Distributor while configuring
        DIST_BASE.byte_add(0x000).write_volatile(0x0);

        // Mask all interrupts initially (assuming 256 max for now)
        for i in 0..(256 / 32) {
            DIST_BASE
                .byte_add(0x180)
                .byte_add(i * 4)
                .write_volatile(0xFFFFFFFF);
        }

        // Set all interrupts to Group 1 (standard IRQs)
        for i in 0..(256 / 32) {
            DIST_BASE
                .byte_add(0x080)
                .byte_add(i * 4)
                .write_volatile(0xFFFFFFFF);
        }

        // Enable Distributor and CPU Interface
        DIST_BASE.byte_add(0x000).write_volatile(0x3);
        CPU_BASE.byte_add(0x000).write_volatile(0x1F);
        CPU_BASE.byte_add(0x004).write_volatile(0xF0); // Priority mask
    }
}

unsafe fn set_priority(intid: u32, priority: u8) {
    let prio_reg = intid / 4;
    let prio_off = (intid % 4) * 8;

    let reg = DIST_BASE.byte_add(0x400).byte_add((prio_reg * 4) as usize);
    let mut val = reg.read_volatile();
    val &= !(0xFF << prio_off);
    val |= (priority as u32) << prio_off;
    reg.write_volatile(val);
}

pub unsafe fn mask_vector(intid: u32) {
    unsafe {
        // GICD_ICENABLERn (Interrupt Clear-Enable Registers)
        // Offset: 0x180 + (reg * 4)
        let reg: u32 = intid / 32;
        let bit: u32 = intid % 32;

        // Writing a 1 to a bit in ICENABLER disables the corresponding interrupt.
        // Writing 0 has no effect.
        DIST_BASE
            .byte_add(0x180)
            .byte_add((reg * 4) as usize)
            .write_volatile(1 << bit);
    }
}

pub unsafe fn unmask_vector(intid: u32) {
    unsafe {
        // GICD_ISENABLERn (Interrupt Set-Enable Registers)
        // Offset: 0x100 + (reg * 4)
        let reg: u32 = intid / 32;
        let bit: u32 = intid % 32;

        // Writing a 1 to a bit in ISENABLER enables the corresponding interrupt.
        // Writing 0 has no effect.
        DIST_BASE
            .byte_add(0x100)
            .byte_add((reg * 4) as usize)
            .write_volatile(1 << bit);

        // Ensure priority is set to something "runnable" (e.g., 0xA0)
        // Higher numbers are lower priority in GIC.
        set_priority(intid, 0xA0);
    }
}

pub unsafe fn configure_interrupt(intid: u32, priority: u8) {
    unsafe {
        set_priority(intid, priority);

        // Set Group 1 (Standard IRQ routing)
        let group_reg: u32 = intid / 32;
        let group_bit: u32 = intid % 32;
        let reg = DIST_BASE.byte_add(0x080).byte_add(group_reg as usize * 4);
        let mut val = reg.read_volatile();
        val |= 1 << group_bit;
        reg.write_volatile(val);
    }
}

pub unsafe fn acknowledge_interrupt() -> u32 {
    (CPU_BASE.byte_add(0x00C).read_volatile()) & 0x3FF
}

pub unsafe fn send_eoi(intid: u32) {
    unsafe {
        CPU_BASE.byte_add(0x010).write_volatile(intid);
    }
}

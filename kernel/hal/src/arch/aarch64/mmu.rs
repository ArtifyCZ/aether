use crate::mmu::VirtualMemoryMappingFlags;
use bitflags::{bitflags, Flags};
use core::arch::asm;

static mut KERNEL_CONTEXT: usize = 0;
static mut HHDM_OFFSET: usize = 0;

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
static mut g_kernel_context: kernel_bindings_gen::vmm_context =
    kernel_bindings_gen::vmm_context { root: 0 };

const ATTR_DEVICE_IDX: usize = 0;
const ATTR_NORMAL_IDX: usize = 1;

const USERSPACE_PAGE_MASK: usize = 0x0000FFFFFFFFF000;
const IDX_MASK: usize = 0x1FF;

const L0_SHIFT: usize = 39;
const L1_SHIFT: usize = 30;
const L2_SHIFT: usize = 21;
const L3_SHIFT: usize = 12;

bitflags! {
    struct Aarch64MappingFlags: u64 {
        const VALID = 1 << 0;
        const TABLE = 1 << 1;
        const PAGE = 1 << 1;
        const AF = 1 << 10; // @TODO: rename
        const SH_INNER = 3 << 8;
        const NG = 1 << 11; // @TODO: rename
        const READONLY = 1 << 7;
        const USER = 1 << 6;
        const NO_EXEC = 1 << 53;
        const USER_NO_EXEC = 1 << 54;
        const NORMAL = (ATTR_NORMAL_IDX as u64) << 2;
        const DEVICE = (ATTR_DEVICE_IDX as u64) << 2;
    }
}

impl Aarch64MappingFlags {
    fn from_vmm_flags(vmm_flags: VirtualMemoryMappingFlags, virt: usize) -> Self {
        let mut flags = Self::empty();
        if vmm_flags.contains(VirtualMemoryMappingFlags::PRESENT) {
            flags |= Self::VALID;
            flags |= Self::AF;
            flags |= Self::SH_INNER;
        }
        if virt >= 0xffffffff80000000 {
            // Ensure bit 11 is NOT set
        } else {
            flags |= Self::NG;
        }
        if !vmm_flags.contains(VirtualMemoryMappingFlags::WRITE) {
            flags |= Self::READONLY;
        }
        if vmm_flags.contains(VirtualMemoryMappingFlags::USER) {
            flags |= Self::USER;
        }
        if !vmm_flags.contains(VirtualMemoryMappingFlags::EXEC) {
            flags |= Self::NO_EXEC;
            if vmm_flags.contains(VirtualMemoryMappingFlags::USER) {
                flags |= Self::USER_NO_EXEC;
            }
        }
        if vmm_flags.contains(VirtualMemoryMappingFlags::DEVICE)
            || vmm_flags.contains(VirtualMemoryMappingFlags::NO_CACHE)
        {
            flags |= Self::DEVICE;
        } else {
            flags |= Self::NORMAL;
        }

        flags
    }
}

unsafe fn init_system_regs() {
    unsafe {
        // TCR Setup is crucial for enabling TTBR0 (User space)
        // Attribute 0: Device, Attribute 1: Normal
        let mair: usize = (0x00 << (ATTR_DEVICE_IDX * 8)) | (0xFF << (ATTR_NORMAL_IDX * 8));
        asm!(
            "msr mair_el1, {0}",
            in(reg) mair,
        );

        /*
         * TCR_EL1:
         * T0SZ/T1SZ = 16 (48-bit address space)
         * TG0/TG1 = 4KB Granule
         * SH/IRGN/ORGN = Inner Shareable, Write-Back Read-Allocate Write-Allocate
         */
        let tcr: usize = 0
            | (16 << 0) | (16 << 16) // T0SZ, T1SZ
            | (0 << 14) | (2 << 30) // TG0=4KB, TG1=4KB
            | (3 << 12) | (3 << 28) // SH0, SH1 (Inner Sharable)
            | (1 << 8) | (1 << 24) // IRGN0, IRGN1 (WB WA)
            | (1 << 10) | (1 << 26) // ORGN0, ORGN1 (WB WA)
            | (2 << 32) // IPS = 40 bit PA
        ;

        asm!(
            "msr tcr_el1, {0}",
            "isb",
            in(reg) tcr,
        );
    }
}

pub unsafe fn init(hhdm_offset: usize) {
    unsafe {
        HHDM_OFFSET = hhdm_offset;
        init_system_regs();

        let current_root: usize;
        asm!(
            "mrs {0}, ttbr1_el1",
            out(reg) current_root,
        );

        KERNEL_CONTEXT = current_root & USERSPACE_PAGE_MASK;
        g_kernel_context.root = KERNEL_CONTEXT;
    }
}

pub unsafe fn get_kernel_context() -> usize {
    unsafe {
        KERNEL_CONTEXT
    }
}

pub unsafe fn create_context() -> usize {
    unsafe {
        // Allocate a physical frame for the new PML0 (Level 0 Table - root table)
        let pml0_phys = kernel_bindings_gen::pmm_alloc_frame();
        if pml0_phys == 0 {
            // return 0;
            // @TODO: handle defensively
            panic!("Could not allocate physical page for page directory!");
        }

        let new_pml4 = (pml0_phys + HHDM_OFFSET) as *mut u64;

        core::ptr::write_bytes(new_pml4, 0, 512);

        pml0_phys
    }
}

unsafe fn invalidate_tlb(virt: usize) {
    unsafe {
        // Invalidate TLB for the correct ASID/Address
        let v_idx = virt >> 12;
        asm!(
            "tlbi vaae1is, {v_idx}",
            "dsb sy",
            "ic ivau, {virt}", // Invalidate instruction cache by VA to Point of Unification
            "dsb sy",
            "isb", // Instruction Synchronization Barrier
            v_idx = in(reg) v_idx,
            virt = in(reg) virt,
        );
    }
}

pub unsafe fn map_page(
    table: usize,
    virt: usize,
    phys: usize,
    flags: VirtualMemoryMappingFlags,
) -> bool {
    unsafe {
        const SHIFTS: [usize; 3] = [L0_SHIFT, L1_SHIFT, L2_SHIFT];

        let mut table = (table + HHDM_OFFSET) as *mut u64;
        let hw_flags = Aarch64MappingFlags::from_vmm_flags(flags, virt);

        for i in 0..3 {
            let idx = (virt >> SHIFTS[i]) & IDX_MASK;
            let entry_val = table.add(idx).read();

            let (next_table_phys, mut current_flags) =
                if (entry_val & Aarch64MappingFlags::VALID.bits()) == 0 {
                    let new_table_phys = kernel_bindings_gen::pmm_alloc_frame();
                    if new_table_phys == 0 {
                        panic!("Could not allocate page frame for a page directory!");
                    }
                    let new_table_virt = (new_table_phys + HHDM_OFFSET) as *mut u64;

                    core::ptr::write_bytes(new_table_virt, 0, 512);

                    (
                        new_table_phys,
                        Aarch64MappingFlags::VALID | Aarch64MappingFlags::TABLE,
                    )
                } else {
                    (
                        entry_val as usize & USERSPACE_PAGE_MASK,
                        Aarch64MappingFlags::from_bits_retain(entry_val),
                    )
                };

            let entry = ((next_table_phys & USERSPACE_PAGE_MASK) as u64) | current_flags.bits();
            table.add(idx).write(entry);
            table = (next_table_phys + HHDM_OFFSET) as *mut u64;
        }

        let l3_idx = (virt >> L3_SHIFT) & IDX_MASK;
        let hw_flags = hw_flags | Aarch64MappingFlags::PAGE;
        let entry = ((phys & USERSPACE_PAGE_MASK) as u64) | hw_flags.bits();
        table.add(l3_idx).write(entry);

        invalidate_tlb(virt);

        true
    }
}

pub unsafe fn unmap_page(table: usize, virt: usize) -> bool {
    unsafe {
        const SHIFTS: [usize; 3] = [L0_SHIFT, L1_SHIFT, L2_SHIFT];

        let mut table = (table + HHDM_OFFSET) as *mut u64;

        for i in 0..3 {
            let idx = (virt >> SHIFTS[i]) & IDX_MASK;
            let entry_val = table.add(idx).read();
            let hw_flags = Aarch64MappingFlags::from_bits_retain(entry_val);
            if !hw_flags.contains(Aarch64MappingFlags::VALID) {
                // return false;
                panic!("Attempted to unmap a non-present page!");
            }

            // If we hit a block mapping before L3, we can't unmap just one 4kiB page
            // without "splitting" the block. For now, we return false.
            if !hw_flags.contains(Aarch64MappingFlags::TABLE) {
                // return false;
                panic!("Attempted to unmap a huge page!");
            }

            let next_table_phys = entry_val as usize & USERSPACE_PAGE_MASK;
            table = (next_table_phys + HHDM_OFFSET) as *mut u64;
        }

        let l3_idx = (virt >> L3_SHIFT) & IDX_MASK;
        table.add(l3_idx).write(0);

        invalidate_tlb(virt);

        true
    }
}

pub unsafe fn translate(table: usize, virt: usize) -> usize {
    unsafe {
        const SHIFTS: [usize; 4] = [L0_SHIFT, L1_SHIFT, L2_SHIFT, L3_SHIFT];

        let mut table = (table + HHDM_OFFSET) as *const u64;

        for i in 0..4 {
            let idx = (virt >> SHIFTS[i]) & IDX_MASK;
            let entry_val = table.add(idx).read();
            let hw_flags = Aarch64MappingFlags::from_bits_retain(entry_val);
            if !hw_flags.contains(Aarch64MappingFlags::VALID) {
                // return 0;
                panic!("Attempted to translate a non-present page!");
            }

            if (i == 3 || ((i > 0) && (!hw_flags.contains(Aarch64MappingFlags::TABLE)))) {
                let page_size_mask = (1 << SHIFTS[i]) - 1;
                let paddr_mask = USERSPACE_PAGE_MASK & !page_size_mask;
                return (entry_val as usize & paddr_mask) | (virt & page_size_mask);
            }

            let next_page = entry_val as usize & USERSPACE_PAGE_MASK;
            table = (next_page + HHDM_OFFSET) as *const u64;
        }

        // 0
        panic!("Attempted to translate a non-present address!");
    }
}

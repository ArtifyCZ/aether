use bitflags::bitflags;
use core::arch::asm;
use crate::mmu::VirtualMemoryMappingFlags;

static mut KERNEL_CONTEXT: usize = 0;
static mut HHDM_OFFSET: usize = 0;

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
static mut g_kernel_context: kernel_bindings_gen::vmm_context = kernel_bindings_gen::vmm_context {
    root: 0,
};

const X86_ADDR_MASK: usize = 0x000FFFFFFFFFF000;

bitflags! {
    struct X86MappingFlags: u64 {
        const PRESENT = 1 << 0;
        const WRITE = 1 << 1;
        const USER = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const CACHE_DISABLE = 1 << 4;
        const HUGE_PAGE = 1 << 7; // 1 GiB / 2 MiB
        const NO_EXEC = 1 << 63;
    }
}

impl X86MappingFlags {
    fn from_vmm_flags(vmm_flags: VirtualMemoryMappingFlags) -> Self {
        let mut flags = X86MappingFlags::empty();
        if vmm_flags.contains(VirtualMemoryMappingFlags::PRESENT) {
            flags.insert(Self::PRESENT);
        }
        if vmm_flags.contains(VirtualMemoryMappingFlags::WRITE) {
            flags.insert(Self::WRITE);
        }
        if vmm_flags.contains(VirtualMemoryMappingFlags::USER) {
            flags.insert(Self::USER);
        }
        if !vmm_flags.contains(VirtualMemoryMappingFlags::EXEC) {
            flags.insert(Self::NO_EXEC);
        }
        if vmm_flags.contains(VirtualMemoryMappingFlags::DEVICE)
            || vmm_flags.contains(VirtualMemoryMappingFlags::NO_CACHE)
        {
            flags.insert(Self::CACHE_DISABLE);
            flags.insert(Self::WRITE_THROUGH);
        }
        flags
    }
}

pub unsafe fn init(hhdm_offset: usize) {
    unsafe {
        HHDM_OFFSET = hhdm_offset;

        let cr3: usize;
        asm!(
        "mov {0}, cr3",
        out(reg) cr3,
        );
        KERNEL_CONTEXT = cr3 & X86_ADDR_MASK;
        g_kernel_context.root = KERNEL_CONTEXT;
    }
}

pub unsafe fn create_context() -> usize {
    unsafe {
        // Allocate a physical frame for the new PML4 (Level 4 Table)
        let pml4_phys = kernel_bindings_gen::pmm_alloc_frame();
        if pml4_phys == 0 {
            // return 0;
            // @TODO: handle defensively
            panic!("Could not allocate physical page for page directory!");
        }

        let new_pml4 = (pml4_phys + HHDM_OFFSET) as *mut u64;
        let kernel_pml4 = (KERNEL_CONTEXT + HHDM_OFFSET) as *const u64;

        // Clear the lower half (Userspace: 0x0000000000000000 - 0x00007FFFFFFFFFFF)
        // This covers PML4 entries 0 to 255.
        core::ptr::write_bytes(new_pml4, 0, 512);

        // Clone the upper half (Kernel: 0xFFFF800000000000 - 0xFFFFFFFFFFFFFFFF)
        // This covers PML4 entries 256 to 511.
        // By copying these pointers, the new context shares the kernel's PDPTs/PDs.
        core::ptr::copy_nonoverlapping(kernel_pml4.add(256), new_pml4.add(256), 256);

        pml4_phys
    }
}

unsafe fn invlpg(virt: usize) {
    unsafe {
        asm!(
        "invlpg [{}]",
        in(reg) virt as u64,
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
        const SHIFTS: [usize; 4] = [39, 30, 21, 12];

        let mut table = (table + HHDM_OFFSET) as *mut u64;
        let hw_flags = X86MappingFlags::from_vmm_flags(flags);

        for i in 0..3 {
            let idx = (virt >> SHIFTS[i]) & 0x1FF;
            let entry_val = table.add(idx).read();

            let (next_table_phys, mut current_flags) = if (entry_val & X86MappingFlags::PRESENT.bits()) == 0 {
                let new_table_phys = kernel_bindings_gen::pmm_alloc_frame();
                if new_table_phys == 0 {
                    panic!("Could not allocate page frame for a page directory!");
                }
                let new_table_virt = (new_table_phys + HHDM_OFFSET) as *mut u64;

                core::ptr::write_bytes(new_table_virt, 0, 512);

                (new_table_phys, X86MappingFlags::PRESENT | X86MappingFlags::WRITE)
            } else {
                (entry_val as usize & X86_ADDR_MASK, X86MappingFlags::from_bits_retain(entry_val))
            };

            if flags.contains(VirtualMemoryMappingFlags::USER) {
                current_flags |= X86MappingFlags::USER;
            }
            if flags.contains(VirtualMemoryMappingFlags::WRITE) {
                current_flags |= X86MappingFlags::WRITE;
            }

            let entry = ((next_table_phys & X86_ADDR_MASK) as u64) | current_flags.bits();
            table.add(idx).write(entry);
            table = (next_table_phys + HHDM_OFFSET) as *mut u64;
        }

        let idx = (virt >> SHIFTS[3]) & 0x1FF;
        table
            .add(idx)
            .write(((phys & X86_ADDR_MASK) as u64) | hw_flags.bits());

        invlpg(virt);
        true
    }
}

pub unsafe fn unmap_page(table: usize, virt: usize) -> bool {
    unsafe {
        const SHIFTS: [usize; 4] = [39, 30, 21, 12];
        let mut table = (table + HHDM_OFFSET) as *mut u64;

        for i in 0..3 {
            let idx = (virt >> SHIFTS[i]) & 0x1FF;
            let hw_flags = X86MappingFlags::from_bits_retain(table.add(idx).read());
            if !hw_flags.contains(X86MappingFlags::PRESENT) {
                // return false;
                panic!("Attempted to unmap a non-present page!");
            }
            let entry = table.add(idx).read() as usize;
            let next_table_phys = entry & X86_ADDR_MASK;
            table = (next_table_phys + HHDM_OFFSET) as *mut u64;
        }

        let idx = (virt >> SHIFTS[3]) & 0x1FF;
        table.add(idx).write(0);
        invlpg(virt);

        true
    }
}

pub unsafe fn translate(table: usize, virt: usize) -> usize {
    unsafe {
        const SHIFTS: [usize; 4] = [39, 30, 21, 12];
        let mut table = (table + HHDM_OFFSET) as *mut u64;

        for i in 0..3 {
            let idx = (virt >> SHIFTS[i]) & 0x1FF;
            let hw_flags = X86MappingFlags::from_bits_retain(table.add(idx).read());

            if !hw_flags.contains(X86MappingFlags::PRESENT) {
                // return 0;
                panic!("Attempted to translate a non-present address!");
            }

            // Handle huge pages (page size bit) if encountered
            if hw_flags.contains(X86MappingFlags::HUGE_PAGE) {
                let mask = (1usize << SHIFTS[i]) - 1;
                return (hw_flags.bits() as usize & !mask) | (virt & mask);
            }

            let entry = table.add(idx).read() as usize;
            let next_table_phys = entry & X86_ADDR_MASK;
            table = (next_table_phys + HHDM_OFFSET) as *mut u64;
        }

        let idx = (virt >> SHIFTS[3]) & 0x1FF;
        let hw_flags = X86MappingFlags::from_bits_retain(table.add(idx).read());
        if !hw_flags.contains(X86MappingFlags::PRESENT) {
            // return 0;
            panic!("Attempted to translate a non-present address!");
        }

        (hw_flags.bits() as usize & X86_ADDR_MASK) | (virt & 0xFFF)
    }
}

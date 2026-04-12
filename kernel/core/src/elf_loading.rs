use alloc::collections::BTreeMap;
use core::ffi::c_void;

use kernel_hal::mmu::VirtualMemoryMappingFlags;

use crate::{
    elf_parsing::{ElfFile, ElfType, ProgramSegment, ProgramSegmentFlags},
    platform::{
        memory_layout::PAGE_FRAME_SIZE, physical_memory_manager::PhysicalMemoryManager,
        virtual_memory_manager_context::VirtualMemoryManagerContext,
        virtual_page_address::VirtualPageAddress,
    },
};

const PAGE_MASK: usize = !(PAGE_FRAME_SIZE - 1);

fn elf_flags_to_vmm_flags(elf_flags: ProgramSegmentFlags) -> VirtualMemoryMappingFlags {
    let mut vmm_flags = VirtualMemoryMappingFlags::PRESENT | VirtualMemoryMappingFlags::USER;

    if elf_flags.contains(ProgramSegmentFlags::WRITABLE) {
        vmm_flags |= VirtualMemoryMappingFlags::WRITE;
    }
    if elf_flags.contains(ProgramSegmentFlags::EXECUTABLE) {
        vmm_flags |= VirtualMemoryMappingFlags::EXEC;
    }

    vmm_flags
}

pub unsafe fn load_elf_program(
    vmm_ctx: &VirtualMemoryManagerContext,
    elf: &ElfFile<'_>,
    hhdm_offset: usize,
) -> usize {
    let base_vaddr: usize = match elf.type_ {
        ElfType::Executable => 0, // Statically linked with absolute addresses
        ElfType::SharedObject => 0x5555_0000, // PIE base address (Can be randomized later for ASLR)
        _ => panic!("Unsupported ELF type for loading: {:?}", elf.type_),
    };

    // Track Virtual Page -> Physical Frame Inner Address
    let mut page_map: BTreeMap<usize, usize> = BTreeMap::new();

    // --- PASS 1: Map Memory and Copy Data ---
    for segment in &elf.segments {
        match segment {
            ProgramSegment::Load(seg) => {
                let flags = elf_flags_to_vmm_flags(seg.flags);
                let virt_start = seg.vaddr + base_vaddr;
                let page_start = virt_start & !0xFFF;
                let page_end = (virt_start + seg.memsz + PAGE_FRAME_SIZE - 1) & PAGE_MASK;

                for current_page_start in (page_start..page_end).step_by(PAGE_FRAME_SIZE) {
                    let phys = unsafe { PhysicalMemoryManager::alloc_frame().unwrap() };

                    // Save the mapping for the relocation pass
                    page_map.insert(current_page_start, phys.inner());

                    unsafe {
                        vmm_ctx
                            .map_page(
                                VirtualPageAddress::new(current_page_start).unwrap(),
                                phys,
                                flags,
                            )
                            .unwrap();
                    }

                    let dest_page = (hhdm_offset + phys.inner()) as *mut u8;
                    let copy_dst_v = if current_page_start < seg.vaddr + base_vaddr {
                        seg.vaddr + base_vaddr
                    } else {
                        current_page_start
                    };

                    let segment_end_v = seg.vaddr + base_vaddr + seg.data.len();
                    let copy_end_v = if current_page_start + PAGE_FRAME_SIZE < segment_end_v {
                        current_page_start + PAGE_FRAME_SIZE
                    } else {
                        segment_end_v
                    };

                    unsafe {
                        core::ptr::write_bytes(dest_page, 0, PAGE_FRAME_SIZE);

                        if copy_dst_v < copy_end_v {
                            let copy_len = copy_end_v - copy_dst_v;
                            let dest_offset = copy_dst_v - current_page_start;

                            let src_ptr =
                                seg.data.as_ptr().add(copy_dst_v - seg.vaddr - base_vaddr);
                            let dest_ptr = dest_page.add(dest_offset);

                            core::ptr::copy_nonoverlapping(src_ptr, dest_ptr, copy_len);
                        }
                    }
                }
            }
            ProgramSegment::Dynamic(_) | ProgramSegment::Unknown(_) => {
                // Ignored during the memory allocation pass
            }
        }
    }

    // --- PASS 2: Apply Relative Relocations ---
    for segment in &elf.segments {
        if let ProgramSegment::Dynamic(dyn_seg) = segment {
            for rela in &dyn_seg.relocations {
                // Calculate where the patch needs to be applied in virtual memory
                let patch_vaddr = base_vaddr + rela.offset;
                let page_start = patch_vaddr & !0xFFF;
                let page_offset = patch_vaddr & 0xFFF;

                // Look up the physical frame we allocated in Pass 1
                if let Some(&phys_inner) = page_map.get(&page_start) {
                    // Construct the absolute kernel pointer via the HHDM
                    let patch_ptr = (hhdm_offset + phys_inner + page_offset) as *mut u64;

                    // Calculate the new absolute value of the pointer
                    let new_value = (base_vaddr as i64 + rela.addend) as u64;

                    // Apply the patch
                    unsafe {
                        *patch_ptr = new_value;
                    }
                } else {
                    panic!(
                        "Relocation target address {:#X} is not mapped!",
                        patch_vaddr
                    );
                }
            }
        }
    }

    elf.entrypoint + base_vaddr
}

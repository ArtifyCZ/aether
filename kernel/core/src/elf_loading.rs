use core::ffi::c_void;

use kernel_hal::mmu::VirtualMemoryMappingFlags;

use crate::{
    elf_parsing::{ElfFile, PhdrType},
    platform::{
        memory_layout::PAGE_FRAME_SIZE, physical_memory_manager::PhysicalMemoryManager,
        virtual_memory_manager_context::VirtualMemoryManagerContext,
        virtual_page_address::VirtualPageAddress,
    },
};

const PAGE_MASK: usize = !(PAGE_FRAME_SIZE - 1);

fn elf_flags_to_vmm_flags(elf_flags: u32) -> VirtualMemoryMappingFlags {
    let mut vmm_flags = VirtualMemoryMappingFlags::PRESENT | VirtualMemoryMappingFlags::USER;
    if elf_flags & 0x02 != 0 {
        vmm_flags |= VirtualMemoryMappingFlags::WRITE;
    }
    if elf_flags & 0x01 != 0 {
        vmm_flags |= VirtualMemoryMappingFlags::EXEC;
    }
    vmm_flags
}

pub unsafe fn load_elf_program(
    vmm_ctx: &VirtualMemoryManagerContext,
    elf: &ElfFile<'_>,
    hhdm_offset: usize,
) -> usize {
    for phdr in &elf.phdrs {
        if phdr.type_ != PhdrType::Load {
            continue;
        }

        let flags = elf_flags_to_vmm_flags(phdr.flags);
        let virt_start = phdr.vaddr;
        let page_start = virt_start & !0xFFF;
        let page_offset = virt_start & 0xFFF;
        let page_end = (virt_start + phdr.memsz + PAGE_FRAME_SIZE - 1) & PAGE_MASK;

        for current_page_start in (page_start..page_end).step_by(PAGE_FRAME_SIZE) {
            let phys = unsafe { PhysicalMemoryManager::alloc_frame().unwrap() };
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
            let copy_dst_v = if current_page_start < phdr.vaddr {
                phdr.vaddr
            } else {
                current_page_start
            };
            let segment_end_v = phdr.vaddr + phdr.data.len();
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
                    let src_ptr = phdr.data.as_ptr().add(copy_dst_v - phdr.vaddr);
                    let dest_ptr = dest_page.add(dest_offset);

                    core::ptr::copy_nonoverlapping(src_ptr, dest_ptr, copy_len);
                }
            }
        }
    }

    elf.entrypoint
}

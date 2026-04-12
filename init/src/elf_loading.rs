use alloc::vec::Vec;
use core::ffi::c_void;

use crate::elf_parsing::{ElfFile, ElfType, ProgramSegment, ProgramSegmentFlags};
use aether_sys::{sys_proc_create, sys_proc_mmap, sys_proc_mprot};

struct MappedRegion {
    child_page_start: usize,
    mirror_chunk: *mut u8,
    map_len: usize,
    sys_flags: u32,
}

fn elf_flags_to_sys_prot(elf_flags: ProgramSegmentFlags) -> u32 {
    let mut sys_flags = aether_sys::SYS_PROT_READ;

    if elf_flags.contains(ProgramSegmentFlags::WRITABLE) {
        sys_flags |= aether_sys::SYS_PROT_WRITE;
    }
    if elf_flags.contains(ProgramSegmentFlags::EXECUTABLE) {
        sys_flags |= aether_sys::SYS_PROT_EXEC;
    }

    sys_flags
}

pub fn load_elf_program(elf: &ElfFile<'_>) -> (u64, usize) {
    let proc_handle = unsafe { sys_proc_create(0).unwrap() };

    let base_vaddr: usize = match elf.type_ {
        ElfType::Executable => 0,
        ElfType::SharedObject => 0x5555_0000, // PIE base address
        _ => panic!("Unsupported ELF type for loading: {:?}", elf.type_),
    };

    let mut mapped_regions = Vec::new();

    for segment in &elf.segments {
        if let ProgramSegment::Load(seg) = segment {
            let flags = elf_flags_to_sys_prot(seg.flags);
            let virt_start = seg.vaddr + base_vaddr;
            let page_start = virt_start & !0xFFF;
            let page_offset = virt_start & 0xFFF;

            let map_len = page_offset + seg.memsz;

            // Map as RW in our mirror so we can patch relocations safely later
            let memory_chunk = unsafe {
                sys_proc_mmap(
                    proc_handle,
                    page_start as *mut u8,
                    map_len as *mut u8,
                    aether_sys::SYS_PROT_READ | aether_sys::SYS_PROT_WRITE,
                    aether_sys::SYS_MMAP_FL_MIRROR,
                )
            }
            .unwrap() as *mut u8;

            unsafe {
                core::ptr::write_bytes(memory_chunk, 0, map_len);

                core::ptr::copy_nonoverlapping(
                    seg.data.as_ptr(),
                    memory_chunk.add(page_offset),
                    seg.data.len(),
                );
            }

            // Save the mapping metadata for Pass 2 and Pass 3
            mapped_regions.push(MappedRegion {
                child_page_start: page_start,
                mirror_chunk: memory_chunk,
                map_len,
                sys_flags: flags,
            });
        }
    }

    // --- PASS 2: Apply Relative Relocations via the Mirror ---
    for segment in &elf.segments {
        if let ProgramSegment::Dynamic(dyn_seg) = segment {
            for rela in &dyn_seg.relocations {
                let patch_vaddr = base_vaddr + rela.offset;

                // Find which mapped region contains this virtual address
                let mut region_found = false;
                for region in &mapped_regions {
                    if patch_vaddr >= region.child_page_start
                        && patch_vaddr < region.child_page_start + region.map_len
                    {
                        let offset_in_mirror = patch_vaddr - region.child_page_start;
                        let patch_ptr =
                            unsafe { region.mirror_chunk.add(offset_in_mirror) } as *mut u64;
                        let new_value = (base_vaddr as i64 + rela.addend) as u64;

                        unsafe {
                            *patch_ptr = new_value;
                        }
                        region_found = true;
                        break;
                    }
                }

                if !region_found {
                    panic!(
                        "Relocation target address {:#X} is not mapped!",
                        patch_vaddr
                    );
                }
            }
        }
    }

    // --- PASS 3: Seal Memory Protections ---
    for region in &mapped_regions {
        unsafe {
            sys_proc_mprot(
                proc_handle,
                region.child_page_start as *mut u8,
                region.map_len as *mut u8,
                region.sys_flags,
            )
            .unwrap();
        }
    }

    (proc_handle, elf.entrypoint + base_vaddr)
}

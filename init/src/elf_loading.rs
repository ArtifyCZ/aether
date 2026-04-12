use core::ffi::c_void;

use crate::elf_parsing::{ElfFile, ProgramSegment, ProgramSegmentFlags};
use aether_sys::{sys_proc_create, sys_proc_mmap, sys_proc_mprot};

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

    for segment in &elf.segments {
        match segment {
            ProgramSegment::Load(seg) => {
                let flags = elf_flags_to_sys_prot(seg.flags);
                let virt_start = seg.vaddr;
                let page_start = virt_start & !0xFFF;
                let page_offset = virt_start & 0xFFF;

                let map_len = page_offset + seg.memsz;

                let memory_chunk = unsafe {
                    sys_proc_mmap(
                        proc_handle,
                        page_start as *mut u8,
                        map_len as *mut u8, // Use map_len, not just memsz!
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

                    sys_proc_mprot(
                        proc_handle,
                        page_start as *mut u8,
                        map_len as *mut u8,
                        flags,
                    )
                    .unwrap();
                }
            }
            ProgramSegment::Unknown(_) => {
                // Safely ignored for now!
            }
        }
    }

    (proc_handle, elf.entrypoint)
}

use core::ffi::c_void;

use crate::elf_parsing::{ElfFile, PhdrType};
use aether_sys::{sys_proc_create, sys_proc_mmap, sys_proc_mprot};

fn elf_flags_to_vmm_prot(elf_flags: u32) -> u32 {
    let mut vmm_flags = 0x01; // PROT_READ
    if elf_flags & 0x02 != 0 {
        vmm_flags |= 0x02; // PROT_WRITE
    }
    if elf_flags & 0x01 != 0 {
        vmm_flags |= 0x04; // PROT_EXEC
    }
    vmm_flags
}

pub fn load_elf_program(elf: &ElfFile<'_>) -> (u64, usize) {
    let proc_handle = unsafe { sys_proc_create(0).unwrap() };

    for phdr in &elf.phdrs {
        if phdr.type_ != PhdrType::Load {
            continue;
        }

        let flags = elf_flags_to_vmm_prot(phdr.flags);
        let virt_start = phdr.vaddr;
        let page_start = virt_start & !0xFFF;
        let memory_chunk = unsafe {
            sys_proc_mmap(
                proc_handle,
                page_start as *mut c_void,
                phdr.memsz as *mut c_void,
                0x01 | 0x02,
                0x01,
            )
        }
        .unwrap() as *mut u8;
        unsafe { core::ptr::write_bytes(memory_chunk, 0, phdr.memsz) };

        let page_offset = virt_start & 0xFFF;
        unsafe {
            core::ptr::copy_nonoverlapping(
                phdr.data.as_ptr(),
                memory_chunk.add(page_offset),
                phdr.data.len(),
            );
            sys_proc_mprot(
                proc_handle,
                page_start as *mut c_void,
                phdr.memsz as *mut c_void,
                flags,
            )
            .unwrap();
        }
    }

    (proc_handle, elf.entrypoint)
}

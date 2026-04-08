use crate::elf_parsing::{ElfFile, PhdrType};
use core::arch::asm;

unsafe fn sys_proc_create(fl: u64) -> Result<u64, u64> {
    unsafe {
        let result: u64;
        let error_code: u64;

        #[cfg(target_arch = "x86_64")]
        asm!(
            "syscall",
            inout("rax") 0x07u64 => result,
            in("rdi") fl,
            lateout("rdx") error_code,
        );

        #[cfg(target_arch = "aarch64")]
        asm!(
            "svc #0",
            in("x8") 0x07u64,
            in("x0") fl,
            lateout("x0") result,
            lateout("x1") error_code,
        );

        if error_code == 0 {
            Ok(result)
        } else {
            Err(error_code)
        }
    }
}

unsafe fn sys_proc_mmap(
    proc_handle: u64,
    addr: usize,
    len: usize,
    pr: u32,
    fl: u32,
) -> Result<usize, u64> {
    unsafe {
        let result: u64;
        let error_code: u64;

        #[cfg(target_arch = "x86_64")]
        asm!(
            "syscall",
            inout("rax") 0x08u64 => result,
            in("rdi") proc_handle,
            in("rsi") addr as u64,
            inout("rdx") len as u64 => error_code,
            in("r10") pr as u64,
            in("r8") fl as u64,
        );

        #[cfg(target_arch = "aarch64")]
        asm!(
            "svc #0",
            in("x8") 0x08u64,
            in("x0") proc_handle,
            in("x1") addr as u64,
            in("x2") len as u64,
            in("x3") pr as u64,
            in("x4") fl as u64,
            lateout("x0") result,
            lateout("x1") error_code,
        );

        if error_code == 0 {
            Ok(result as usize)
        } else {
            Err(error_code)
        }
    }
}

unsafe fn sys_proc_mprot(proc_handle: u64, addr: usize, len: usize, pr: u32) -> Result<(), u64> {
    unsafe {
        let result: u64;
        let error_code: u64;

        #[cfg(target_arch = "x86_64")]
        asm!(
            "syscall",
            inout("rax") 0x09u64 => result,
            in("rdi") proc_handle,
            in("rsi") addr as u64,
            in("rdx") len as u64,
            in("r10") pr as u64,
            lateout("rdx") error_code,
        );

        #[cfg(target_arch = "aarch64")]
        asm!(
            "svc #0",
            in("x8") 0x09u64,
            in("x0") proc_handle,
            in("x1") addr as u64,
            in("x2") len as u64,
            in("x3") pr as u64,
            lateout("x0") result,
            lateout("x1") error_code,
        );

        if error_code == 0 {
            Ok(())
        } else {
            Err(error_code)
        }
    }
}

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
        let memory_chunk =
            unsafe { sys_proc_mmap(proc_handle, page_start, phdr.memsz, 0x01 | 0x02, 0x01) }
                .unwrap() as *mut u8;
        unsafe { core::ptr::write_bytes(memory_chunk, 0, phdr.memsz) };

        let page_offset = virt_start & 0xFFF;
        unsafe {
            core::ptr::copy_nonoverlapping(
                phdr.data.as_ptr(),
                memory_chunk.add(page_offset),
                phdr.data.len(),
            );
            sys_proc_mprot(proc_handle, page_start, phdr.memsz, flags).unwrap();
        }
    }

    (proc_handle, elf.entrypoint)
}

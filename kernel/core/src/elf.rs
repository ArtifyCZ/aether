use crate::platform::virtual_memory_manager_context::{VirtualMemoryManagerContext, VirtualMemoryMappingFlags};
use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::physical_memory_manager::PhysicalMemoryManager;
use crate::platform::virtual_page_address::VirtualPageAddress;

pub struct Elf {
    hhdm_offset: u64,
}

#[cfg(target_arch = "aarch64")]
const MACHINE_TYPE: u16 = 0xB7;
#[cfg(target_arch = "x86_64")]
const MACHINE_TYPE: u16 = 0x3E;

const EI_DATA: usize = 0x5;

const ELFDATA2LSB: u8 = 0x1;

const PT_LOAD: u32 = 0x01;

const ELF_PF_W: u32 = 0x2;
const ELF_PF_X: u32 = 0x1;

pub type Elf64_Addr = u64;
pub type Elf64_Off = u64;
pub type Elf64_Half = u16;
pub type Elf64_Word = u32;
pub type Elf64_Xword = u64;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Elf64_Ehdr {
    pub e_ident: [::core::ffi::c_uchar; 16usize],
    pub e_type: Elf64_Half,
    pub e_machine: Elf64_Half,
    pub e_version: Elf64_Word,
    pub e_entry: Elf64_Addr,
    pub e_phoff: Elf64_Off,
    pub e_shoff: Elf64_Off,
    pub e_flags: Elf64_Word,
    pub e_ehsize: Elf64_Half,
    pub e_phentsize: Elf64_Half,
    pub e_phnum: Elf64_Half,
    pub e_shentsize: Elf64_Half,
    pub e_shnum: Elf64_Half,
    pub e_shstrndx: Elf64_Half,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Elf64_Phdr {
    pub p_type: Elf64_Word,
    pub p_flags: Elf64_Word,
    pub p_offset: Elf64_Off,
    pub p_vaddr: Elf64_Addr,
    pub p_paddr: Elf64_Addr,
    pub p_filesz: Elf64_Xword,
    pub p_memsz: Elf64_Xword,
    pub p_align: Elf64_Xword,
}

// @FIXME: This impl is just awful
// @TODO: Refactor it to use parser combinators (probably `nom` lib) and separate parsing from loading
impl Elf {
    pub fn init(hhdm_offset: u64) -> Self {
        Self { hhdm_offset }
    }

    pub unsafe fn load(
        &self,
        vmm_ctx: &VirtualMemoryManagerContext,
        data: *const u8,
    ) -> Option<usize> {
        unsafe {
            let header: *const Elf64_Ehdr = data.cast();
            if (*header).e_machine != MACHINE_TYPE {
                // Incompatible with the architecture
                return None;
            }

            if (*header).e_ident[EI_DATA] != ELFDATA2LSB {
                // If ELF is not little endian
                return None;
            }

            let phdrs: *const Elf64_Phdr = data.offset((*header).e_phoff as isize).cast();

            for i in 0..(*header).e_phnum {
                let phdr = phdrs.offset(i as isize);
                if (*phdr).p_type != PT_LOAD {
                    continue;
                }

                let flags = {
                    let p_flags = (*phdr).p_flags;
                    let mut flags = VirtualMemoryMappingFlags::PRESENT | VirtualMemoryMappingFlags::USER;
                    if p_flags & ELF_PF_W != 0 {
                        flags.insert(VirtualMemoryMappingFlags::WRITE);
                    } else if p_flags & ELF_PF_X != 0 {
                        flags.insert(VirtualMemoryMappingFlags::EXEC);
                    }
                    flags
                };

                let virt_start = (*phdr).p_vaddr as usize;
                let virt_end = virt_start + (*phdr).p_memsz as usize;

                const PAGE_MASK: usize = !(PAGE_FRAME_SIZE - 1);
                let page_start = virt_start & PAGE_MASK;
                let page_end = (virt_end + PAGE_FRAME_SIZE - 1) & PAGE_MASK;

                let pages_to_map = (page_end - page_start) / PAGE_FRAME_SIZE;

                for p in 0..pages_to_map {
                    let phys = PhysicalMemoryManager::alloc_frame().unwrap();
                    let virt = page_start + p * PAGE_FRAME_SIZE;

                    vmm_ctx.map_page(VirtualPageAddress::new(virt).unwrap(), phys, flags).unwrap();

                    let dest = (self.hhdm_offset as *mut u8).add(phys.inner());
                    core::ptr::write_bytes(dest, 0, PAGE_FRAME_SIZE);

                    let copy_dst_v = if virt < (*phdr).p_vaddr as usize { (*phdr).p_vaddr as usize } else { virt };
                    let segment_v_end = (*phdr).p_vaddr as usize + (*phdr).p_filesz as usize;
                    let copy_end_v = if virt + PAGE_FRAME_SIZE < segment_v_end { virt + PAGE_FRAME_SIZE } else { segment_v_end };

                    if copy_dst_v < copy_end_v {
                        let copy_len = copy_end_v - copy_dst_v;
                        let dest_offset = copy_dst_v - virt;
                        let src_offset = (*phdr).p_offset as usize + (copy_dst_v - (*phdr).p_vaddr as usize);

                        core::ptr::copy_nonoverlapping(data.add(src_offset), dest.add(dest_offset), copy_len);
                    }
                }
            }

            Some((*header).e_entry as usize)
        }
    }
}

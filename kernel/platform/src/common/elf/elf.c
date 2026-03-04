#include "elf.h"

#include <stddef.h>

#include "physical_memory_manager.h"
#include "string.h"
#include "virtual_memory_manager.h"

static uintptr_t g_hhdm_offset = 0;

static vmm_flags_t p_flags_to_vmm(Elf64_Word p_flags) {
    vmm_flags_t flags = VMM_FLAG_PRESENT | VMM_FLAG_USER;
    if (p_flags & ELF_PF_W) {
        flags |= VMM_FLAG_WRITE;
    } else if (p_flags & ELF_PF_X) {
        flags |= VMM_FLAG_EXEC;
    }

    return flags;
}

void elf_init(uintptr_t hhdm_offset) {
    g_hhdm_offset = hhdm_offset;
}

int elf_load(const struct vmm_context *ctx, void *data, uintptr_t *out_v_addr_entrypoint) {
    if (g_hhdm_offset == 0x0) {
        return -1; // ELF hasn't been initialized!
    }

    const Elf64_Ehdr *header = (Elf64_Ehdr *) data;
    if (!elf_arch_is_supported(header->e_machine)) {
        return -1;
    }

    if (header->e_ident[EI_DATA] != ELFDATA2LSB) {
        // If ELF is not little endian
        return -1; // Big endian not supported
    }

    Elf64_Phdr *phdrs = (Elf64_Phdr *) ((uintptr_t) data + header->e_phoff);

    for (uint16_t i = 0; i < header->e_phnum; i++) {
        Elf64_Phdr *phdr = &phdrs[i];
        if (phdr->p_type == PT_LOAD) {
            vmm_flags_t flags = p_flags_to_vmm(phdr->p_flags);
            uintptr_t virt_start = phdr->p_vaddr;
            uintptr_t virt_end = virt_start + phdr->p_memsz;

            uintptr_t page_start = virt_start & ~0xFFFULL;
            uintptr_t page_end = (virt_end + 0xFFF) & ~0xFFFULL;

            size_t pages_to_map = (page_end - page_start) / VMM_PAGE_SIZE;

            for (size_t p = 0; p < pages_to_map; p++) {
                uintptr_t phys = pmm_alloc_frame();
                uintptr_t virt = page_start + p * VMM_PAGE_SIZE;

                vmm_map_page(ctx, virt, phys, flags);

                uint8_t *dest = (uint8_t *) (phys + g_hhdm_offset);

                memset(dest, 0, VMM_PAGE_SIZE);

                uintptr_t copy_dst_v = (virt < phdr->p_vaddr) ? phdr->p_vaddr : virt;
                uintptr_t segment_v_end = phdr->p_vaddr + phdr->p_filesz;
                uintptr_t copy_end_v = (virt + VMM_PAGE_SIZE < segment_v_end) ? (virt + VMM_PAGE_SIZE) : segment_v_end;

                if (copy_dst_v < copy_end_v) {
                    size_t copy_len = copy_end_v - copy_dst_v;
                    size_t dest_offset = copy_dst_v - virt;
                    size_t src_offset = phdr->p_offset + (copy_dst_v - phdr->p_vaddr);

                    memcpy(dest + dest_offset, (uint8_t *) data + src_offset, copy_len);
                }
            }
        }
    }

    *out_v_addr_entrypoint = header->e_entry;
    return 0;
}

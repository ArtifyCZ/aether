#pragma once

#include <stdbool.h>
#include <stdint.h>
#include "virtual_memory_manager.h"

typedef uint64_t Elf64_Addr;
typedef uint64_t Elf64_Off;
typedef uint16_t Elf64_Half;
typedef uint32_t Elf64_Word;
typedef uint64_t Elf64_Xword;

#define ELFMAG "\177ELF"
#define SELFMAG 4

#define EI_DATA 0x5
#define ELFDATANONE 0x0
#define ELFDATA2LSB 0x1
#define ELFDATA2MSB 0x2

#define ELF_PF_X 0x1
#define ELF_PF_W 0x2
#define ELF_PF_R 0x4

typedef struct {
    unsigned char e_ident[16]; /* Magic number and other info */
    Elf64_Half e_type; /* Object file type */
    Elf64_Half e_machine; /* Architecture */
    Elf64_Word e_version; /* Object file version */
    Elf64_Addr e_entry; /* Entry point virtual address */
    Elf64_Off e_phoff; /* Program header table file offset */
    Elf64_Off e_shoff; /* Section header table file offset */
    Elf64_Word e_flags; /* Processor-specific flags */
    Elf64_Half e_ehsize; /* ELF header size */
    Elf64_Half e_phentsize; /* Program header table entry size */
    Elf64_Half e_phnum; /* Program header table entry count */
    Elf64_Half e_shentsize; /* Section header table entry size */
    Elf64_Half e_shnum; /* Section header table entry count */
    Elf64_Half e_shstrndx; /* Section header string table index */
} Elf64_Ehdr;

#define PT_LOAD 1

typedef struct {
    Elf64_Word p_type; /* Segment type */
    Elf64_Word p_flags; /* Segment flags (R, W, X) */
    Elf64_Off p_offset; /* Segment file offset */
    Elf64_Addr p_vaddr; /* Segment virtual address */
    Elf64_Addr p_paddr; /* Segment physical address */
    Elf64_Xword p_filesz; /* Segment size in file */
    Elf64_Xword p_memsz; /* Segment size in memory */
    Elf64_Xword p_align; /* Segment alignment */
} Elf64_Phdr;

void elf_init(uintptr_t hhdm_offset);

int elf_load(const struct vmm_context *ctx, void *data, uintptr_t *out_v_addr_entrypoint);

bool elf_arch_is_supported(Elf64_Half arch);

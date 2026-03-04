#include "elf.h"

#define EM_X86_64 0x3E

bool elf_arch_is_supported(const Elf64_Half arch) {
    return arch == EM_X86_64;
}

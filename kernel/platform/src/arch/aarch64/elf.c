#include "elf.h"

#define EM_AARCH64 0xB7

bool elf_arch_is_supported(const Elf64_Half arch) {
    return arch == EM_AARCH64;
}

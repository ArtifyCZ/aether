#include "virtual_memory_manager.h"
#include "virtual_memory_manager_priv.h"
#include "physical_memory_manager.h"
#include "string.h"

static uint64_t g_hhdm_offset = 0;
struct vmm_context g_kernel_context;

// Internal helpers
static inline void* get_vaddr(uintptr_t phys) {
    return (void*)(phys + g_hhdm_offset);
}

static inline void invlpg(uintptr_t virt) {
    __asm__ volatile("invlpg (%0)" : : "r"(virt) : "memory");
}

// Convert our generic VMM flags to x86 hardware bits
static uint64_t flags_to_x86(vmm_flags_t flags) {
    uint64_t x86_flags = 0;
    if (flags & VMM_FLAG_PRESENT) x86_flags |= X86_PTE_PRESENT;
    if (flags & VMM_FLAG_WRITE)   x86_flags |= X86_PTE_WRITE;
    if (flags & VMM_FLAG_USER)    x86_flags |= X86_PTE_USER;
    if (!(flags & VMM_FLAG_EXEC)) x86_flags |= X86_PTE_NX;

    // For Device memory (MMIO) on x86, we usually disable caching
    if (flags & VMM_FLAG_DEVICE || flags & VMM_FLAG_NOCACHE) {
        x86_flags |= X86_PTE_PCD | X86_PTE_PWT;
    }
    return x86_flags;
}

/**
 * Creates a new VMM context for a userspace process.
 * Clones the kernel's high-memory mappings so the kernel stays visible.
 */
struct vmm_context vmm_context_create(void) {
    struct vmm_context context = { .root = 0 };

    // 1. Allocate a physical frame for the new PML4 (Level 4 Table)
    uintptr_t pml4_phys = pmm_alloc_frame();
    if (!pml4_phys) return context;

    context.root = pml4_phys;
    uint64_t* new_pml4 = (uint64_t*)get_vaddr(pml4_phys);
    uint64_t* kernel_pml4 = (uint64_t*)get_vaddr(g_kernel_context.root);

    // 2. Clear the lower half (Userspace: 0x0000000000000000 - 0x00007FFFFFFFFFFF)
    // This covers PML4 entries 0 to 255.
    memset(new_pml4, 0, 256 * sizeof(uint64_t));

    // 3. Clone the upper half (Kernel: 0xFFFF800000000000 - 0xFFFFFFFFFFFFFFFF)
    // This covers PML4 entries 256 to 511.
    // By copying these pointers, the new context shares the kernel's PDPTs/PDs.
    for (int i = 256; i < 512; i++) {
        new_pml4[i] = kernel_pml4[i];
    }

    return context;
}

void vmm_init(uint64_t hhdm_offset) {
    g_hhdm_offset = hhdm_offset;

    uint64_t cr3;
    __asm__ volatile("mov %%cr3, %0" : "=r"(cr3));

    // x86_64 CR3 contains the physical address of the PML4
    g_kernel_context.root = cr3 & X86_ADDR_MASK;
}

bool vmm_map_page(const struct vmm_context *context, uintptr_t virt, uintptr_t phys, vmm_flags_t flags) {
    static const int shifts[] = {39, 30, 21, 12};
    uint64_t* table = get_vaddr(context->root);
    uint64_t hw_flags = flags_to_x86(flags);

    for (int i = 0; i < 3; i++) {
        int idx = (virt >> shifts[i]) & 0x1FF;

        if (!(table[idx] & X86_PTE_PRESENT)) {
            uintptr_t new_table_phys = pmm_alloc_frame();
            if (!new_table_phys) return false;

            memset(get_vaddr(new_table_phys), 0, VMM_PAGE_SIZE);

            // INITIAL LINK: We set PRESENT and WRITE here.
            // But we MUST set USER here if the final target is a user page.
            table[idx] = new_table_phys | X86_PTE_PRESENT | X86_PTE_WRITE;
        }

        // UPDATE PARENT BITS: If the mapping we are creating is USER or WRITE,
        // we must ensure the existing parent entry also allows it.
        if (flags & VMM_FLAG_USER)  table[idx] |= X86_PTE_USER;
        if (flags & VMM_FLAG_WRITE) table[idx] |= X86_PTE_WRITE;

        table = get_vaddr(table[idx] & X86_ADDR_MASK);
    }

    int idx = (virt >> shifts[3]) & 0x1FF;
    table[idx] = (phys & X86_ADDR_MASK) | hw_flags;

    invlpg(virt);
    return true;
}

bool vmm_unmap_page(const struct vmm_context *context, uintptr_t virt) {
    static const int shifts[] = {39, 30, 21, 12};
    uint64_t* table = get_vaddr(context->root);

    for (int i = 0; i < 3; i++) {
        int idx = (virt >> shifts[i]) & 0x1FF;
        if (!(table[idx] & X86_PTE_PRESENT)) return false;
        table = get_vaddr(table[idx] & X86_ADDR_MASK);
    }

    table[(virt >> shifts[3]) & 0x1FF] = 0;
    invlpg(virt);
    return true;
}

uintptr_t vmm_translate(const struct vmm_context *context, uintptr_t virt) {
    static const int shifts[] = {39, 30, 21, 12};
    uint64_t* table = get_vaddr(context->root);

    for (int i = 0; i < 3; i++) {
        int idx = (virt >> shifts[i]) & 0x1FF;
        uint64_t entry = table[idx];
        if (!(entry & X86_PTE_PRESENT)) return 0;

        // Handle huge pages (PS bit) if encountered
        if (entry & X86_PTE_PS) {
            uintptr_t mask = (1ULL << shifts[i]) - 1;
            return (entry & X86_ADDR_MASK & ~mask) | (virt & mask);
        }
        table = get_vaddr(entry & X86_ADDR_MASK);
    }

    uint64_t entry = table[(virt >> 12) & 0x1FF];
    if (!(entry & X86_PTE_PRESENT)) return 0;

    return (entry & X86_ADDR_MASK) | (virt & 0xFFF);
}

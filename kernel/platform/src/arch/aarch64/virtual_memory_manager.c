#include "virtual_memory_manager.h"
#include "virtual_memory_manager_priv.h"
#include "physical_memory_manager.h"
#include "string.h"

struct vmm_context g_kernel_context;
static uint64_t g_hhdm = 0;

static inline uint64_t* get_vaddr(uintptr_t paddr) {
    return (uint64_t*)(paddr + g_hhdm);
}

static uint64_t flags_to_desc(vmm_flags_t flags) {
    uint64_t desc = DESC_VALID | DESC_AF | DESC_SH_INNER;

    if (!(flags & VMM_FLAG_WRITE)) desc |= DESC_RO;

    // On AArch64, the User bit (bit 6) in the Page Descriptor
    if (flags & VMM_FLAG_USER) desc |= DESC_USER;

    // Execute Never bits
    if (!(flags & VMM_FLAG_EXEC)) {
        desc |= DESC_PXN;
        if (flags & VMM_FLAG_USER) desc |= DESC_UXN;
    }

    // Memory Attributes (MAIR)
    desc |= ((flags & (VMM_FLAG_DEVICE | VMM_FLAG_NOCACHE)) ?
            (ATTR_DEVICE_IDX << 2) : (ATTR_NORMAL_IDX << 2));

    return desc;
}

// TCR Setup is crucial for enabling TTBR0 (User space)
static void init_system_regs(void) {
    // Attribute 0: Device, Attribute 1: Normal
    uint64_t mair = (0x00ULL << (ATTR_DEVICE_IDX * 8)) | (0xFFULL << (ATTR_NORMAL_IDX * 8));
    __asm__ volatile("msr mair_el1, %0" : : "r"(mair));

    /**
     * TCR_EL1:
     * T0SZ/T1SZ = 16 (48-bit address space)
     * TG0/TG1 = 4KB Granule
     * SH/IRGN/ORGN = Inner Shareable, Write-Back Read-Allocate Write-Allocate
     */
    uint64_t tcr = (16ULL << 0)  | (16ULL << 16) | // T0SZ, T1SZ
                   (0ULL << 14)  | (2ULL << 30)  | // TG0=4KB, TG1=4KB
                   (3ULL << 12)  | (3ULL << 28)  | // SH0, SH1 (Inner Shareable)
                   (1ULL << 8)   | (1ULL << 24)  | // IRGN0, IRGN1 (WB WA)
                   (1ULL << 10)  | (1ULL << 26)  | // ORGN0, ORGN1 (WB WA)
                   (2ULL << 32);                   // IPS = 40 bit PA

    __asm__ volatile("msr tcr_el1, %0; isb" : : "r"(tcr));
}

void vmm_init(uint64_t hhdm_offset) {
    g_hhdm = hhdm_offset;
    init_system_regs();

    uintptr_t current_root;
    __asm__ volatile("mrs %0, ttbr1_el1" : "=r"(current_root));
    g_kernel_context.root = current_root & 0x0000FFFFFFFFF000ULL;
}

/**
 * Creates a new context for a User process.
 * Crucially, TTBR0 roots should NOT contain kernel mappings.
 */
struct vmm_context vmm_context_create(void) {
    struct vmm_context ctx;
    ctx.root = pmm_alloc_frame();
    if (ctx.root) {
        memset(get_vaddr(ctx.root), 0, VMM_PAGE_SIZE);
    }
    return ctx;
}

bool vmm_map_page(const struct vmm_context *context, uintptr_t virt, uintptr_t phys, vmm_flags_t flags) {
    uint64_t* table = get_vaddr(context->root);
    static const int shifts[] = {L0_SHIFT, L1_SHIFT, L2_SHIFT};

    for (int i = 0; i < 3; i++) {
        int idx = (virt >> shifts[i]) & IDX_MASK;

        if (!(table[idx] & DESC_VALID)) {
            uintptr_t new_table_phys = pmm_alloc_frame();
            if (!new_table_phys) return false;
            memset(get_vaddr(new_table_phys), 0, VMM_PAGE_SIZE);

            // On AArch64, intermediate tables don't usually need the USER bit
            // unless you are using APTable bits for strict hierarchical permissions.
            // Standard DESC_TABLE | DESC_VALID is enough.
            table[idx] = new_table_phys | DESC_TABLE | DESC_VALID;
        }
        table = get_vaddr(table[idx] & 0x0000FFFFFFFFF000ULL);
    }

    int l3_idx = (virt >> L3_SHIFT) & IDX_MASK;
    table[l3_idx] = (phys & 0x0000FFFFFFFFF000ULL) | DESC_PAGE | flags_to_desc(flags);

    // Invalidate TLB for the correct ASID/Address
    __asm__ volatile("tlbi vaae1is, %0; dsb sy; isb" : : "r"(virt >> 12));

    return true;
}

/**
 * Unmaps a virtual page and invalidates the TLB.
 */
bool vmm_unmap_page(const struct vmm_context *context, uintptr_t virt) {
    uint64_t* table = get_vaddr(context->root);
    const static int shifts[] = {L0_SHIFT, L1_SHIFT, L2_SHIFT};

    // Navigate to the L3 table
    for (int i = 0; i < 3; i++) {
        int idx = (virt >> shifts[i]) & IDX_MASK;
        uint64_t entry = table[idx];

        if (!(entry & DESC_VALID)) {
            return false; // Path doesn't exist
        }

        // If we hit a block mapping before L3, we can't unmap just one 4K page
        // without "splitting" the block. For now, we return false.
        if (!(entry & DESC_TABLE)) {
            return false;
        }

        table = get_vaddr(entry & 0x0000FFFFFFFFF000ULL);
    }

    int l3_idx = (virt >> L3_SHIFT) & IDX_MASK;

    if (!(table[l3_idx] & DESC_VALID)) {
        return false;
    }

    table[l3_idx] = 0;

    // TLB Invalidation is mandatory after unmapping.
    // 'vaae1is' invalidates the VA for all ASIDs in the Inner Shareable domain.
    __asm__ volatile("tlbi vaae1is, %0" : : "r"(virt >> 12));
    __asm__ volatile("dsb sy; isb");

    return true;
}

/**
 * Translates a virtual address to a physical address by walking the page tables.
 */
uintptr_t vmm_translate(const struct vmm_context *context, uintptr_t virt) {
    uint64_t* table = get_vaddr(context->root);
    const static int shifts[] = {L0_SHIFT, L1_SHIFT, L2_SHIFT, L3_SHIFT};

    for (int i = 0; i < 4; i++) {
        int idx = (virt >> shifts[i]) & IDX_MASK;
        uint64_t entry = table[idx];

        if (!(entry & DESC_VALID)) {
            return 0;
        }

        // Check for Block Mappings (L1 and L2 only) or Terminal Page (L3)
        // AArch64: In L1/L2, bit 1 = 0 means Block, bit 1 = 1 means Table.
        // In L3, bit 1 = 1 means Page.
        if (i == 3 || (i > 0 && !(entry & DESC_TABLE))) {
            uintptr_t page_size_mask = (1ULL << shifts[i]) - 1;
            uintptr_t paddr_mask = 0x0000FFFFFFFFF000ULL & ~page_size_mask;
            return (entry & paddr_mask) | (virt & page_size_mask);
        }

        table = get_vaddr(entry & 0x0000FFFFFFFFF000ULL);
    }

    return 0;
}

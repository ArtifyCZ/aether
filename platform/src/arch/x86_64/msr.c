#include "msr.h"

#include <stdint.h>

#define MSR_EFER  0xC0000080
#define MSR_STAR  0xC0000081
#define MSR_LSTAR 0xC0000082
#define MSR_FMASK 0xC0000084

static inline void wrmsr(uint32_t msr, uint64_t val) {
    uint32_t low = (uint32_t)val;
    uint32_t high = (uint32_t)(val >> 32);
    __asm__ volatile("wrmsr" : : "c"(msr), "a"(low), "d"(high));
}

struct cpu_local_storage {
    uint64_t kernel_stack;
    uint64_t user_rsp_scratch;
};

static struct cpu_local_storage g_cpu_local_storage;

extern void syscalls_raw_handler(void);

void msr_init(void) {
    // Point to the syscall assembly entry
    wrmsr(MSR_LSTAR, (uint64_t)syscalls_raw_handler);

    // Setup segments
    // Base is now Index 3 (User 32 Code)
    // SS = Index 4 (0x23), CS = Index 5 (0x2B)
    const uint64_t star = ((uint64_t)0x001B << 48) | ((uint64_t)0x0008 << 32);
    wrmsr(MSR_STAR, star);

    // Disable interrupts on entry (IF bit = 0x200)
    wrmsr(MSR_FMASK, 0x200);

    // Enable system call extension
    // We should read first to be safe, but bit 0 is SCE
    __asm__ volatile(
        "mov $0xC0000080, %%ecx\n"
        "rdmsr\n"
        "or $1, %%eax\n"
        "wrmsr\n"
        : : : "eax", "ecx", "edx"
    );

    // Set the KERNEL_GS_BASE to our local storage
    // When swapgs is called in kernel entry, GS will point here
    wrmsr(0xC0000102, (uint64_t)&g_cpu_local_storage);
}

void msr_set_kernel_stack(const uint64_t stack) {
    g_cpu_local_storage.kernel_stack = stack;
}

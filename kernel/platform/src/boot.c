#include "boot.h"

// Halt and catch fire function.
_Noreturn void hcf(void) {
    for (;;) {
#if defined (__x86_64__)
        __asm__ volatile ("cli");
        __asm__ volatile ("hlt");
#elif defined (__aarch64__) || defined (__riscv)
        __asm__ volatile ("msr daifset, #3");
        __asm__ volatile ("wfi");
#elif defined (__loongarch64)
        asm ("idle 0");
#endif
    }
}

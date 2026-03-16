#include "boot.h"

#include <stdint.h>
#include <limine.h>

#include "interrupts.h"

// Halt and catch fire function.
_Noreturn void hcf(void) {
    interrupts_disable(); // prevent any switches

    for (;;) {
#if defined (__x86_64__)
        asm ("hlt");
#elif defined (__aarch64__) || defined (__riscv)
        asm ("wfi");
#elif defined (__loongarch64)
        asm ("idle 0");
#endif
    }
}

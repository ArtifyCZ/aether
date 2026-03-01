#include "cpu_local.h"

static inline void msr_tpidr_el1_write(uintptr_t ptr) {
    __asm__ volatile("msr tpidr_el1, %0" : : "r"(ptr));
}

static inline uintptr_t msr_tpidr_el1_read(void) {
    uintptr_t ptr;
    __asm__ volatile("mrs %0, tpidr_el1" : "=r"(ptr));
    return ptr;
}

static struct cpu_local_storage g_cpu_local = {
    .task_id = 0,
};

void cpu_local_init(void) {
    msr_tpidr_el1_write((uintptr_t) &g_cpu_local);
}

struct cpu_local_storage *cpu_local_get(void) {
    const uintptr_t ptr = msr_tpidr_el1_read();
    return (struct cpu_local_storage *) ptr;
}

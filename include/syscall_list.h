#pragma once

// All syscalls in a macro list
#define SYSCALLS_LIST(X) \
    /* Name         NAME        Num     Ret         Cnt   Args... */ \
    /* Exits the current task immediately */ \
    X(exit,         EXIT,       0x00,   void,       0) \
    /* Writes bytes to the specific file descriptor */ \
    X(write,        WRITE,      0x01,   void,       3,  int, fd, const void*, buf, size_t, count) \
    /* Creates a new task with shared address space at the ip address (function ptr) with the sp as the stack top ptr */ \
    X(clone,        CLONE,      0x02,   uint64_t,   3,  uint64_t, flags, const void*, sp, const void*, ip) \
    /* Maps a new page-aligned memory chunk into the address space with the flags and the protection flags */ \
    X(mmap,         MMAP,       0x03,   uintptr_t,  4,  uint64_t, addr, uint64_t, len, uint32_t, pr, uint32_t, fl) \
    /* Blocks the current task till an interrupt with the provided IRQ is fired */ \
    X(irq_wait,     IRQ_WAIT,   0x04,   void,       1,  uint8_t, irq) \
    /* Unmasks interrupts of the specific IRQ, usually used before the IRQ_WAIT syscall */ \
    X(irq_unmask,   IRQ_UNMASK, 0x05,   void,       1,  uint8_t, irq) \
    /* Maps a new page-aligned MMIO device memory chunk into the address space to the provided physical address */ \
    X(mmap_dev,     MMAP_DEV,   0x06,   uintptr_t,  5,  uintptr_t, addr, uintptr_t, len, uintptr_t, phys, uint32_t, pr, uint32_t, fl) \
    /* End of the list */

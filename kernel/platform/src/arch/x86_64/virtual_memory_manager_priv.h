#pragma once

// x86_64 Hardware Page Table Bits
#define X86_PTE_PRESENT  (1ULL << 0)
#define X86_PTE_WRITE    (1ULL << 1)
#define X86_PTE_USER     (1ULL << 2)
#define X86_PTE_PWT      (1ULL << 3) // Page Write-Through
#define X86_PTE_PCD      (1ULL << 4) // Page Cache Disable
#define X86_PTE_PS       (1ULL << 7) // Page Size (1GB/2MB)
#define X86_PTE_NX       (1ULL << 63)// No Execute

#define X86_ADDR_MASK    0x000FFFFFFFFFF000ULL

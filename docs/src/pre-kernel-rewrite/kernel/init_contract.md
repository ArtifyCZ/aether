# Aether Kernel and the Init Program Contract

## Data provided by the kernel and initial stack

Upon process entry, the kernel provides the following resources mapped into the `init` address space:

- The `boot_info` structure.
- The `initrd` (tarball archive).
- A single page-sized (4096 bytes) bootstrap stack.

### Memory Layout Convention
All kernel-provided data shall reside at the high end of the userspace virtual address range (the "Top of the Lower Half"). 

1. **Growth Direction:** Data blocks are placed contiguously, starting from the highest available canonical address (e.g., `0x00007FFFFFFFF000`) and growing downwards.
2. **Bootstrap Stack:** The one-page initial stack is placed as the **final** (lowest address) item in this kernel-provided data block. 
3. **Allocation Boundary:** The lowest address of the bootstrap stack serves as the **Stack Limit**. The `init` program can rely on this address as the hard upper bound for its own self-allocated "real" stack.

### Execution State at Entry
- **RSP/SP:** Shall point to the top of the bootstrap stack.
- **RDI/X0:** Shall contain the virtual address of the `boot_info` structure.
- **Alignment:** The initial stack pointer must be 16-byte aligned to satisfy the SysV ABI before the first function call.

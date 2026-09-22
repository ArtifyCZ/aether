use crate::requests;
use atom_hal::mmu::{ExecutableAddress, ExecutableRegion, HhdmRegion, HhdmRegionKind};
use limine::memmap::{
    MEMMAP_ACPI_NVS, MEMMAP_ACPI_RECLAIMABLE, MEMMAP_BAD_MEMORY, MEMMAP_BOOTLOADER_RECLAIMABLE,
    MEMMAP_EXECUTABLE_AND_MODULES, MEMMAP_FRAMEBUFFER, MEMMAP_MAPPED_RESERVED, MEMMAP_RESERVED,
    MEMMAP_USABLE,
};

unsafe extern "C" {
    static __kernel_start: u8;
    static __kernel_requests_end: u8;
    static __kernel_text_start: u8;
    static __kernel_text_end: u8;
    static __kernel_rodata_start: u8;
    static __kernel_rodata_end: u8;
    static __kernel_data_start: u8;
    static __kernel_end: u8;
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn _start() -> ! {
    use core::arch::naked_asm;

    naked_asm!(
        "cli",
        "call {init_func}",
        "2:",
        "cli",
        "hlt",
        "jmp 2b",
        init_func = sym init,
    )
}

#[unsafe(naked)]
#[cfg(target_arch = "x86_64")]
unsafe extern "C" fn secondary_entry() -> ! {
    use core::arch::naked_asm;

    naked_asm!(
        "cli",
        "call {main_func}",
        "2:",
        "cli",
        "hlt",
        "jmp 2b",
        main_func = sym crate::main,
    )
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
#[cfg(target_arch = "aarch64")]
pub unsafe extern "C" fn _start() -> ! {
    use core::arch::naked_asm;

    naked_asm!(
        "mov x0, sp",
        "bic x0, x0, #0xf",
        "ldr x18, =0xDEADD00D",
        "msr spsel, #1",
        "isb",
        "msr daifset, #0xf",
        "isb",
        "mov sp, x0",
        "bl {init_func}",
        "2:",
        "msr daifset, #0xf",
        "wfi",
        "b 2b",
        init_func = sym init,
    )
}

#[unsafe(naked)]
#[cfg(target_arch = "aarch64")]
unsafe extern "C" fn secondary_entry() -> ! {
    use core::arch::naked_asm;

    naked_asm!(
        "msr daifset, #0xf",
        "bl {main_func}",
        "2:",
        "msr daifset, #0xf",
        "wfi",
        "b 2b",
        main_func = sym crate::main,
    )
}

fn hhdm_region_kind(memory_type: u64) -> Option<HhdmRegionKind> {
    match memory_type {
        MEMMAP_USABLE
        | MEMMAP_ACPI_RECLAIMABLE
        | MEMMAP_ACPI_NVS
        | MEMMAP_BOOTLOADER_RECLAIMABLE
        | MEMMAP_EXECUTABLE_AND_MODULES => Some(HhdmRegionKind::Normal),
        MEMMAP_FRAMEBUFFER => Some(HhdmRegionKind::Framebuffer),
        MEMMAP_MAPPED_RESERVED if cfg!(target_arch = "x86_64") => Some(HhdmRegionKind::Device),
        MEMMAP_RESERVED | MEMMAP_BAD_MEMORY | MEMMAP_MAPPED_RESERVED => None,
        _ => None,
    }
}

unsafe extern "C" fn init() -> ! {
    requests::assert_compatible_base_revision();
    let hhdm_offset = requests::hhdm_offset();
    let (executable_physical_base, executable_virtual_base) = requests::executable_address();
    let kernel_virtual_start = &raw const __kernel_start as usize;
    let kernel_virtual_end = &raw const __kernel_end as usize;
    assert_eq!(executable_virtual_base, kernel_virtual_start);
    let executable = ExecutableAddress {
        physical_base: executable_physical_base,
        virtual_base: executable_virtual_base,
        length: kernel_virtual_end
            .checked_sub(kernel_virtual_start)
            .expect("invalid kernel image range"),
    };
    let executable_regions = [
        ExecutableRegion {
            virtual_start: kernel_virtual_start,
            length: (&raw const __kernel_requests_end as usize) - kernel_virtual_start,
            writable: true,
            executable: false,
        },
        ExecutableRegion {
            virtual_start: &raw const __kernel_text_start as usize,
            length: (&raw const __kernel_text_end as usize)
                - (&raw const __kernel_text_start as usize),
            writable: false,
            executable: true,
        },
        ExecutableRegion {
            virtual_start: &raw const __kernel_rodata_start as usize,
            length: (&raw const __kernel_rodata_end as usize)
                - (&raw const __kernel_rodata_start as usize),
            writable: false,
            executable: false,
        },
        ExecutableRegion {
            virtual_start: &raw const __kernel_data_start as usize,
            length: kernel_virtual_end - (&raw const __kernel_data_start as usize),
            writable: true,
            executable: false,
        },
    ];

    let hhdm_regions = requests::memory_regions().iter().filter_map(|entry| {
        Some(HhdmRegion {
            physical_start: entry.base as usize,
            length: entry.length as usize,
            kind: hhdm_region_kind(entry.type_)?,
        })
    });
    #[cfg(target_arch = "aarch64")]
    let hhdm_regions = hhdm_regions.chain(core::iter::once(HhdmRegion {
        physical_start: 0x0900_0000,
        length: 0x1000,
        kind: HhdmRegionKind::Device,
    }));

    unsafe {
        atom_hal::mmu::init(
            hhdm_offset,
            hhdm_regions,
            executable,
            executable_regions,
            secondary_entry as *const (),
        )
    }
}

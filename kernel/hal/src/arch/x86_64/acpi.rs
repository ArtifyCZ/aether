use crate::arch::x86_64::ioapic;
use crate::mmu::VirtualMemoryMappingFlags;
use crate::{early_console, mmu};
use alloc::format;
use core::ffi::CStr;
use core::ptr::NonNull;

unsafe fn map_physical_table(phys_addr: usize) -> usize {
    unsafe {
        let page_phys = phys_addr & !(kernel_bindings_gen::VMM_PAGE_SIZE as usize - 1);
        let page_offset = phys_addr & (kernel_bindings_gen::VMM_PAGE_SIZE as usize - 1);

        // Allocate a virtual range
        let virt_base =
            kernel_bindings_gen::vaa_alloc_range(kernel_bindings_gen::VMM_PAGE_SIZE as usize);

        let kernel_context = mmu::get_kernel_context();
        mmu::map_page(
            kernel_context,
            virt_base,
            page_phys,
            VirtualMemoryMappingFlags::PRESENT
                | VirtualMemoryMappingFlags::WRITE
                | VirtualMemoryMappingFlags::DEVICE,
        );

        virt_base + page_offset
    }
}

#[repr(C, packed)]
struct RsdpDescriptor {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32, // 32-bit pointer (for ACPI 1.0)

    // The following fields only exist if revision >= 2 (ACPI 2.0+)
    length: u32,
    xsdt_address: u64, // 64-bit pointer! This is what we want.
    extended_checksum: u8,
    reserved: [u8; 3],
}

#[repr(C, packed)]
struct AcpiHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

#[repr(C, packed)]
struct Rsdt {
    header: AcpiHeader,
    pointer_to_other_tables: u32, // Array of 32-bit physical pointers
}

#[repr(C, packed)]
struct MadtHeader {
    header: AcpiHeader,
    lapic_addr: u32, // Local APIC physical address
    flags: u32,      // 1 = Dual 8259s installed (Legacy PIC)
}

#[repr(C, packed)]
struct MadtEntryHeader {
    type_: u8,
    length: u8,
}

// Type 1: I/O APIC
#[repr(C, packed)]
struct MadtIoapic {
    header: MadtEntryHeader,
    ioapic_id: u8,
    reserved: u8,
    address: u32,
    gsiv_base: u32,
}

// Type 2: Interrupt Source Override
#[repr(C, packed)]
struct MadtIso {
    header: MadtEntryHeader,
    bus_source: u8,
    irq_source: u8,
    gsi: u32,
    flags: u16,
}

unsafe fn acpi_find_table(
    rsdp: &RsdpDescriptor,
    target_signature: &CStr,
) -> Option<NonNull<AcpiHeader>> {
    unsafe {
        // Map the RSDT header first to find out how big it is
        let rsdt_phys = rsdp.rsdt_address as usize;
        let rsdt_ptr = map_physical_table(rsdt_phys) as *mut Rsdt;
        let rsdt = rsdt_ptr.as_ref().unwrap();

        // Calculate how many pointers are in the table
        // (Total length - header size) / size of a 32-bit pointer
        let entries = (rsdt.header.length as usize - size_of::<AcpiHeader>()) / 4;

        for i in 0..entries {
            let message = format!("RSDT entry: {:x}\n", i);
            early_console::print(&message);
            let other_table = core::ptr::addr_of!((*rsdt_ptr).pointer_to_other_tables)
                .add(i)
                .read() as usize;

            let header = map_physical_table(other_table) as *mut AcpiHeader;
            let header_ref = header.as_ref().unwrap();

            if header_ref.signature == target_signature.to_bytes()[0..4] {
                return NonNull::new(header);
            }

            // @TODO: If not found, unmap the header to save virtual space
        }

        None
    }
}

pub unsafe fn init(rsdp: usize) {
    unsafe {
        let rsdp = map_physical_table(rsdp) as *const RsdpDescriptor;
        let rsdp = rsdp.as_ref().unwrap();

        if rsdp.revision != 0 {
            // @TODO: handle different RSDP revisions
            panic!("RSDP revision not implemented!");
        }

        let madt = acpi_find_table(rsdp, c"APIC");
        let Some(madt) = madt else {
            panic!("MADT not found!");
        };
        let madt: NonNull<MadtHeader> = madt.cast();
        let mut ptr: NonNull<u8> = madt.add(1).cast();
        let end: NonNull<u8> = madt.byte_add(madt.read().header.length as usize).cast();

        let mut ioapic = 0;
        while ptr < end {
            let entry: NonNull<MadtEntryHeader> = ptr.cast();
            let entry = entry.as_ref();

            match entry.type_ {
                1 => {
                    // I/O APIC
                    let io: NonNull<MadtIoapic> = ptr.cast();
                    let io = io.as_ptr();
                    let address = core::ptr::addr_of!((*io).address).read() as usize;
                    let message = format!("Found I/O APIC at phys: {:x}\n", address);
                    early_console::print(&message);
                    ioapic = address;
                }
                2 => {
                    // Interrupt Source Override
                    let iso: NonNull<MadtIso> = ptr.cast();
                    let iso = iso.as_ptr();
                    let irq_source = core::ptr::addr_of!((*iso).irq_source).read();
                    let gsi = core::ptr::addr_of!((*iso).gsi).read();
                    let message = format!("ISO: IRQ {:x} -> GSI {:x}\n", irq_source, gsi);
                    early_console::print(&message);
                }
                _ => {}
            }

            ptr = ptr.add(entry.length as usize);
        }

        ioapic::init(ioapic);
    }
}

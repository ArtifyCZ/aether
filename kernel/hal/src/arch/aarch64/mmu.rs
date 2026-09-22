use crate::arch::interface::mmu::{
    ArchMMU, ExecutableAddress, ExecutableRegion, HhdmRegion, HhdmRegionKind, MMUMappingFlags,
    PageSize,
};
use atom_primitives::HhdmOffset;
use core::arch::asm;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

const NORMAL_PAGE_SIZE: usize = 4096;
const LARGE_PAGE_SIZE: usize = 512 * NORMAL_PAGE_SIZE;
const HUGE_PAGE_SIZE: usize = 512 * LARGE_PAGE_SIZE;

const VALID: u64 = 1 << 0;
const TABLE_OR_PAGE: u64 = 1 << 1;
const ATTR_INDEX_SHIFT: usize = 2;
const AP_USER: u64 = 1 << 6;
const AP_READ_ONLY: u64 = 1 << 7;
const SH_OUTER: u64 = 2 << 8;
const SH_INNER: u64 = 3 << 8;
const ACCESS_FLAG: u64 = 1 << 10;
const NOT_GLOBAL: u64 = 1 << 11;
const PRIVILEGED_EXECUTE_NEVER: u64 = 1 << 53;
const USER_EXECUTE_NEVER: u64 = 1 << 54;

const ATTR_NORMAL: u64 = 0;
const ATTR_FRAMEBUFFER: u64 = 1;
const ATTR_DEVICE: u64 = 2;

const SHIFTS: [usize; 4] = [39, 30, 21, 12];
const INDEX_MASK: usize = 0x1ff;

#[repr(C, align(4096))]
struct Table {
    entries: [u64; 512],
}

const _: () = [(); 1][size_of::<Table>() - NORMAL_PAGE_SIZE];
const _: () = [(); 1][align_of::<Table>() - NORMAL_PAGE_SIZE];

pub struct ArchMMUImpl;

pub struct InitResult {
    ttbr0: usize,
    ttbr1: usize,
    mair: usize,
    tcr: usize,
}

static HHDM_OFFSET: AtomicUsize = AtomicUsize::new(0);
static KERNEL_PHYSICAL_BASE: AtomicUsize = AtomicUsize::new(0);
static KERNEL_VIRTUAL_BASE: AtomicUsize = AtomicUsize::new(0);
static KERNEL_LENGTH: AtomicUsize = AtomicUsize::new(0);
static PHYSICAL_ADDRESS_MASK: AtomicUsize = AtomicUsize::new(0);
static FRAMEBUFFER_IS_DEVICE: AtomicBool = AtomicBool::new(false);
static MMU_ACTIVE: AtomicBool = AtomicBool::new(false);

const SPARE_PAGE_TABLES_COUNT: usize = 128;
static mut SPARE_PAGE_TABLES_MEM: [Table; SPARE_PAGE_TABLES_COUNT] =
    unsafe { MaybeUninit::zeroed().assume_init() };
static NEXT_SPARE_PAGE_TABLE_IDX: AtomicUsize = AtomicUsize::new(0);

fn hhdm_offset() -> Option<HhdmOffset> {
    match HHDM_OFFSET.load(Ordering::Relaxed) {
        0 => None,
        offset => Some(unsafe { HhdmOffset::new(offset) }),
    }
}

fn align_down(value: usize, alignment: usize) -> usize {
    value & !(alignment - 1)
}

fn align_up(value: usize, alignment: usize) -> usize {
    value
        .checked_add(alignment - 1)
        .expect("address alignment overflow")
        & !(alignment - 1)
}

fn page_size_bytes(page_size: PageSize) -> usize {
    match page_size {
        PageSize::Normal => NORMAL_PAGE_SIZE,
        PageSize::Large => LARGE_PAGE_SIZE,
        PageSize::Huge => HUGE_PAGE_SIZE,
    }
}

fn page_size_to_level(page_size: PageSize) -> usize {
    match page_size {
        PageSize::Huge => 1,
        PageSize::Large => 2,
        PageSize::Normal => 3,
    }
}

fn kernel_virt_to_phys(virtual_address: usize) -> usize {
    let virtual_base = KERNEL_VIRTUAL_BASE.load(Ordering::Relaxed);
    let physical_base = KERNEL_PHYSICAL_BASE.load(Ordering::Relaxed);
    let kernel_length = KERNEL_LENGTH.load(Ordering::Relaxed);
    let offset = virtual_address
        .checked_sub(virtual_base)
        .expect("page table is below the kernel virtual base");
    assert!(
        offset < kernel_length,
        "page table is outside the kernel image"
    );
    physical_base
        .checked_add(offset)
        .expect("kernel physical address overflow")
}

fn alloc_page_table() -> Option<*mut Table> {
    let page_idx = NEXT_SPARE_PAGE_TABLE_IDX.fetch_add(1, Ordering::Relaxed);
    if page_idx >= SPARE_PAGE_TABLES_COUNT {
        return None;
    }

    let table = unsafe { &raw mut SPARE_PAGE_TABLES_MEM[page_idx] };
    unsafe {
        core::ptr::write_bytes(table.cast::<u8>(), 0, NORMAL_PAGE_SIZE);
    }
    Some(table)
}

fn initialize_cpu_features() {
    let memory_model: usize;
    unsafe {
        asm!("mrs {value}, id_aa64mmfr0_el1", value = out(reg) memory_model);
    }
    let physical_range = memory_model & 0xf;
    let physical_address_bits = match physical_range {
        0 => 32,
        1 => 36,
        2 => 40,
        3 => 42,
        4 => 44,
        5 => 48,
        6 => 52,
        _ => panic!("unsupported AArch64 physical address range"),
    };
    PHYSICAL_ADDRESS_MASK.store((1usize << physical_address_bits) - 1, Ordering::Relaxed);
}

fn select_page_size(physical_address: usize, virtual_address: usize, remaining: usize) -> PageSize {
    if remaining >= HUGE_PAGE_SIZE
        && physical_address.is_multiple_of(HUGE_PAGE_SIZE)
        && virtual_address.is_multiple_of(HUGE_PAGE_SIZE)
    {
        PageSize::Huge
    } else if remaining >= LARGE_PAGE_SIZE
        && physical_address.is_multiple_of(LARGE_PAGE_SIZE)
        && virtual_address.is_multiple_of(LARGE_PAGE_SIZE)
    {
        PageSize::Large
    } else {
        PageSize::Normal
    }
}

fn leaf_descriptor(physical_address: usize, page_size: PageSize, flags: MMUMappingFlags) -> u64 {
    let (attribute_index, shareability) = if flags.contains(MMUMappingFlags::FRAMEBUFFER) {
        let shareability = if FRAMEBUFFER_IS_DEVICE.load(Ordering::Relaxed) {
            SH_OUTER
        } else {
            SH_INNER
        };
        (ATTR_FRAMEBUFFER, shareability)
    } else if flags.contains(MMUMappingFlags::DEVICE) {
        (ATTR_DEVICE, SH_OUTER)
    } else {
        (ATTR_NORMAL, SH_INNER)
    };

    let mut descriptor = physical_address as u64
        | VALID
        | ACCESS_FLAG
        | shareability
        | (attribute_index << ATTR_INDEX_SHIFT);
    if page_size == PageSize::Normal {
        descriptor |= TABLE_OR_PAGE;
    }
    if !flags.contains(MMUMappingFlags::WRITE) {
        descriptor |= AP_READ_ONLY;
    }
    if flags.contains(MMUMappingFlags::USER) {
        descriptor |= AP_USER | NOT_GLOBAL | PRIVILEGED_EXECUTE_NEVER;
        if !flags.contains(MMUMappingFlags::EXEC) {
            descriptor |= USER_EXECUTE_NEVER;
        }
    } else {
        descriptor |= USER_EXECUTE_NEVER;
        if !flags.contains(MMUMappingFlags::EXEC) {
            descriptor |= PRIVILEGED_EXECUTE_NEVER;
        }
    }
    descriptor
}

unsafe fn invalidate_tlb_address(virtual_address: usize) {
    let operand = virtual_address >> 12;
    unsafe {
        asm!(
            "dsb ishst",
            "tlbi vaae1is, {operand}",
            "dsb ish",
            "isb",
            operand = in(reg) operand,
            options(nostack),
        );
    }
}

unsafe fn map_range(
    root_page_table: usize,
    mut physical_address: usize,
    mut virtual_address: usize,
    length: usize,
    flags: MMUMappingFlags,
) {
    let end = physical_address
        .checked_add(length)
        .expect("mapping range overflow");
    while physical_address < end {
        let page_size = select_page_size(physical_address, virtual_address, end - physical_address);
        unsafe {
            ArchMMUImpl::map_page(
                root_page_table,
                physical_address,
                virtual_address,
                page_size,
                flags,
            );
        }
        let step = page_size_bytes(page_size);
        physical_address += step;
        virtual_address += step;
    }
}

impl ArchMMU for ArchMMUImpl {
    type InitResult = InitResult;

    unsafe fn init(
        hhdm_offset: HhdmOffset,
        hhdm_regions: &mut dyn Iterator<Item = HhdmRegion>,
        executable: ExecutableAddress,
        executable_regions: &mut dyn Iterator<Item = ExecutableRegion>,
    ) -> Self::InitResult {
        if HHDM_OFFSET.load(Ordering::Relaxed) != 0
            || KERNEL_PHYSICAL_BASE.load(Ordering::Relaxed) != 0
            || KERNEL_VIRTUAL_BASE.load(Ordering::Relaxed) != 0
            || KERNEL_LENGTH.load(Ordering::Relaxed) != 0
            || NEXT_SPARE_PAGE_TABLE_IDX.load(Ordering::Relaxed) != 0
        {
            panic!("MMU abstraction already initialized");
        }

        assert_ne!(executable.physical_base, 0);
        assert_ne!(executable.virtual_base, 0);
        assert_ne!(executable.length, 0);
        assert!(executable.physical_base.is_multiple_of(NORMAL_PAGE_SIZE));
        assert!(executable.virtual_base.is_multiple_of(NORMAL_PAGE_SIZE));

        initialize_cpu_features();
        HHDM_OFFSET.store(*hhdm_offset, Ordering::Relaxed);
        KERNEL_PHYSICAL_BASE.store(executable.physical_base, Ordering::Relaxed);
        KERNEL_VIRTUAL_BASE.store(executable.virtual_base, Ordering::Relaxed);
        KERNEL_LENGTH.store(executable.length, Ordering::Relaxed);

        let current_mair: usize;
        let current_tcr: usize;
        unsafe {
            asm!("mrs {value}, mair_el1", value = out(reg) current_mair);
            asm!("mrs {value}, tcr_el1", value = out(reg) current_tcr);
        }
        let framebuffer_attribute = (current_mair >> 8) & 0xff;
        FRAMEBUFFER_IS_DEVICE.store(framebuffer_attribute & 0xf0 == 0, Ordering::Relaxed);
        let mair = current_mair & !(0xff << 16);
        let tcr = current_tcr | (1 << 7); // EPD0: disable lower-half walks during bootstrap.

        let ttbr1_table = alloc_page_table().expect("early page-table pool exhausted");
        let ttbr0_table = alloc_page_table().expect("early page-table pool exhausted");
        let ttbr1 = kernel_virt_to_phys(ttbr1_table as usize);
        let ttbr0 = kernel_virt_to_phys(ttbr0_table as usize);
        let physical_address_mask = PHYSICAL_ADDRESS_MASK.load(Ordering::Relaxed);
        assert_eq!(ttbr0 & !physical_address_mask, 0);
        assert_eq!(ttbr1 & !physical_address_mask, 0);

        let executable_end = executable
            .virtual_base
            .checked_add(executable.length)
            .expect("kernel executable range overflow");
        for region in executable_regions {
            if region.length == 0 {
                continue;
            }
            let virtual_start = align_down(region.virtual_start, NORMAL_PAGE_SIZE);
            let virtual_end = align_up(
                region
                    .virtual_start
                    .checked_add(region.length)
                    .expect("kernel region overflow"),
                NORMAL_PAGE_SIZE,
            );
            assert!(virtual_start >= executable.virtual_base);
            assert!(virtual_end <= align_up(executable_end, NORMAL_PAGE_SIZE));
            let physical_start = executable
                .physical_base
                .checked_add(virtual_start - executable.virtual_base)
                .expect("kernel region physical address overflow");
            let mut flags = MMUMappingFlags::PRESENT;
            if region.writable {
                flags |= MMUMappingFlags::WRITE;
            }
            if region.executable {
                flags |= MMUMappingFlags::EXEC;
            }
            unsafe {
                map_range(
                    ttbr1,
                    physical_start,
                    virtual_start,
                    virtual_end - virtual_start,
                    flags,
                );
            }
        }

        for region in hhdm_regions {
            if region.length == 0 {
                continue;
            }
            let physical_start = align_down(region.physical_start, NORMAL_PAGE_SIZE);
            let physical_end = align_up(
                region
                    .physical_start
                    .checked_add(region.length)
                    .expect("HHDM region overflow"),
                NORMAL_PAGE_SIZE,
            );
            let virtual_start = physical_start
                .checked_add(*hhdm_offset)
                .expect("HHDM virtual address overflow");
            let mut flags = MMUMappingFlags::PRESENT | MMUMappingFlags::WRITE;
            match region.kind {
                HhdmRegionKind::Normal => {}
                HhdmRegionKind::Device => flags |= MMUMappingFlags::DEVICE,
                HhdmRegionKind::Framebuffer => flags |= MMUMappingFlags::FRAMEBUFFER,
            }
            unsafe {
                map_range(
                    ttbr1,
                    physical_start,
                    virtual_start,
                    physical_end - physical_start,
                    flags,
                );
            }
        }

        InitResult {
            ttbr0,
            ttbr1,
            mair,
            tcr,
        }
    }

    unsafe fn activate(init_result: Self::InitResult, next_entrypoint: *const ()) -> ! {
        MMU_ACTIVE.store(true, Ordering::Relaxed);
        unsafe {
            asm!(
                "msr daifset, #0xf",
                "dsb ishst",
                "msr mair_el1, {mair}",
                "msr ttbr0_el1, {ttbr0}",
                "msr ttbr1_el1, {ttbr1}",
                "msr tcr_el1, {tcr}",
                "isb",
                "tlbi vmalle1is",
                "dsb ish",
                "isb",
                "br {next}",
                mair = in(reg) init_result.mair,
                ttbr0 = in(reg) init_result.ttbr0,
                ttbr1 = in(reg) init_result.ttbr1,
                tcr = in(reg) init_result.tcr,
                next = in(reg) next_entrypoint,
                options(nostack, noreturn),
            )
        }
    }

    unsafe fn map_page(
        root_page_table: usize,
        physical_address: usize,
        virtual_address: usize,
        page_size: PageSize,
        flags: MMUMappingFlags,
    ) {
        let hhdm_offset = hhdm_offset().expect("MMU abstraction is not initialized");
        let physical_address_mask = PHYSICAL_ADDRESS_MASK.load(Ordering::Relaxed);
        assert_ne!(physical_address_mask, 0);
        assert_eq!(root_page_table & !physical_address_mask, 0);

        let page_level = page_size_to_level(page_size);
        let page_size_bytes = page_size_bytes(page_size);
        assert!(flags.contains(MMUMappingFlags::PRESENT));
        assert!(virtual_address.is_multiple_of(page_size_bytes));
        assert!(physical_address.is_multiple_of(page_size_bytes));
        assert_eq!(physical_address & !physical_address_mask, 0);

        let mut table = (*hhdm_offset + root_page_table) as *mut Table;
        for shift in SHIFTS.iter().take(page_level) {
            let index = (virtual_address >> shift) & INDEX_MASK;
            let entry = unsafe { &mut (*table).entries[index] };
            if *entry & VALID == 0 {
                let new_table = alloc_page_table().expect("early page-table pool exhausted");
                let new_table_phys = kernel_virt_to_phys(new_table as usize);
                assert_eq!(new_table_phys & !physical_address_mask, 0);
                *entry = new_table_phys as u64 | VALID | TABLE_OR_PAGE;
            }
            assert_ne!(*entry & TABLE_OR_PAGE, 0, "mapping collides with a block");
            let table_address_mask = physical_address_mask & !(NORMAL_PAGE_SIZE - 1);
            let table_phys_addr = *entry as usize & table_address_mask;
            table = (*hhdm_offset + table_phys_addr) as *mut Table;
        }

        let index = (virtual_address >> SHIFTS[page_level]) & INDEX_MASK;
        let entry = unsafe { &mut (*table).entries[index] };
        assert!(
            page_size == PageSize::Normal || *entry & VALID == 0 || *entry & TABLE_OR_PAGE == 0,
            "block mapping collides with a lower-level table"
        );
        if *entry & VALID != 0 && MMU_ACTIVE.load(Ordering::Relaxed) {
            *entry = 0;
            unsafe { invalidate_tlb_address(virtual_address) };
        }
        *entry = leaf_descriptor(physical_address, page_size, flags);
        if MMU_ACTIVE.load(Ordering::Relaxed) {
            unsafe {
                asm!("dsb ishst", "isb", options(nostack));
            }
        }
    }
}

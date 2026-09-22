use crate::arch::interface::mmu::{
    ArchMMU, ExecutableAddress, ExecutableRegion, HhdmRegion, HhdmRegionKind, MMUMappingFlags,
    PageSize,
};
use atom_primitives::HhdmOffset;
use bitflags::bitflags;
use core::arch::{asm, x86_64::__cpuid};
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

bitflags! {
    #[derive(Copy, Clone)]
    struct ArchImplFlags: u64 {
        /// Tells MMU this entry is valid; MMU page-faults if not enabled
        const PRESENT = 1 << 0;
        /// Enables write access (vs. read-only otherwise)
        const WRITE = 1 << 1;
        /// Enables access from unprivileged mode (userspace)
        const USER = 1 << 2;
        /// Write through cache (immediately flush the memory write to RAM)
        const WRITE_THROUGH = 1 << 3;
        /// Disables caching completely (any read always goes to RAM)
        const CACHE_DISABLE = 1 << 4;
        /// MMU sets to 1 if it has been accessed; usually used for some page eviction algorithms
        const ACCESSED = 1 << 5;
        /// MMU sets to 1 if it has been written to; usually used to find pages to flush to disk (e.g., mem-mapped files)
        const DIRTY = 1 << 6;
        /// MMU interprets the entry as a bigger mapped page (if 0, it interprets the entry as another table's address)
        const PAGE_SIZE = 1 << 7; // large - 2 MiB; huge - 1 GiB
        /// Allows MMU to preserve TLB cache across different root page tables (different CR3 values)
        const GLOBAL = 1 << 8;
        /// Disables execution of the memory as instructions
        const NO_EXEC = 1 << 63;
    }
}

unsafe fn invlpg(virtual_address: usize) {
    unsafe {
        asm!(
            "invlpg [{}]",
            in(reg) virtual_address as u64,
            options(nostack, preserves_flags),
        );
    }
}

const NORMAL_PAGE_SIZE: usize = 4096; // 4 KiB
const LARGE_PAGE_SIZE: usize = 512 * NORMAL_PAGE_SIZE; // 2 MiB
const HUGE_PAGE_SIZE: usize = 512 * LARGE_PAGE_SIZE; // 1 GiB

static NX_SUPPORTED: AtomicBool = AtomicBool::new(false);
static HUGE_PAGES_SUPPORTED: AtomicBool = AtomicBool::new(false);
static PHYSICAL_ADDRESS_MASK: AtomicUsize = AtomicUsize::new(0);

impl From<MMUMappingFlags> for ArchImplFlags {
    fn from(flags: MMUMappingFlags) -> Self {
        let mut arch_flags = Self::empty();
        if flags.contains(MMUMappingFlags::PRESENT) {
            arch_flags.insert(Self::PRESENT);
        }
        if flags.contains(MMUMappingFlags::WRITE) {
            arch_flags.insert(Self::WRITE);
        }
        if !flags.contains(MMUMappingFlags::EXEC) && NX_SUPPORTED.load(Ordering::Relaxed) {
            arch_flags.insert(Self::NO_EXEC);
        }
        if flags.contains(MMUMappingFlags::USER) {
            arch_flags.insert(Self::USER);
        } else {
            arch_flags.insert(Self::GLOBAL);
        }
        if flags.intersects(MMUMappingFlags::DEVICE | MMUMappingFlags::FRAMEBUFFER) {
            arch_flags.insert(Self::WRITE_THROUGH | Self::CACHE_DISABLE);
        }
        arch_flags
    }
}

pub struct ArchMMUImpl;

#[repr(C, align(4096))]
struct Table {
    entries: [ArchImplFlags; 512],
}

/// Assert size of [`Table`]
const _: () = [(); 1][size_of::<Table>() - NORMAL_PAGE_SIZE];
/// Assert alignment of [`Table`]
const _: () = [(); 1][align_of::<Table>() - NORMAL_PAGE_SIZE];

static HHDM_OFFSET: AtomicUsize = AtomicUsize::new(0);
static KERNEL_PHYSICAL_BASE: AtomicUsize = AtomicUsize::new(0);
static KERNEL_VIRTUAL_BASE: AtomicUsize = AtomicUsize::new(0);
static KERNEL_LENGTH: AtomicUsize = AtomicUsize::new(0);

fn hhdm_offset() -> Option<HhdmOffset> {
    match HHDM_OFFSET.load(Ordering::Relaxed) {
        0 => None,
        x => Some(unsafe { HhdmOffset::new(x) }),
    }
}

// The early page tables live in the executable's .bss. This pool must be large enough to
// construct the initial address space before the physical page allocator is available.
const SPARE_PAGE_TABLES_COUNT: usize = 128;
static mut SPARE_PAGE_TABLES_MEM: [Table; SPARE_PAGE_TABLES_COUNT] =
    unsafe { MaybeUninit::zeroed().assume_init() };
static NEXT_SPARE_PAGE_TABLE_IDX: AtomicUsize = AtomicUsize::new(0);

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

fn align_down(n: usize, alignment: usize) -> usize {
    n & !(alignment - 1)
}

fn align_up(n: usize, alignment: usize) -> usize {
    n.checked_add(alignment - 1)
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

const SHIFTS: [usize; 4] = [39, 30, 21, 12];
const HIGHER_HALF_FIRST_ADDR: usize = 0xFFFF800000000000;

fn initialize_cpu_features() {
    let maximum_extended_leaf = __cpuid(0x8000_0000).eax;
    let extended_features = if maximum_extended_leaf >= 0x8000_0001 {
        __cpuid(0x8000_0001).edx
    } else {
        0
    };
    NX_SUPPORTED.store(extended_features & (1 << 20) != 0, Ordering::Relaxed);
    HUGE_PAGES_SUPPORTED.store(extended_features & (1 << 26) != 0, Ordering::Relaxed);

    let physical_address_bits = if maximum_extended_leaf >= 0x8000_0008 {
        __cpuid(0x8000_0008).eax as usize & 0xff
    } else {
        36
    };
    assert!((36..=52).contains(&physical_address_bits));
    PHYSICAL_ADDRESS_MASK.store((1usize << physical_address_bits) - 1, Ordering::Relaxed);
}

fn select_page_size(physical_address: usize, virtual_address: usize, remaining: usize) -> PageSize {
    if HUGE_PAGES_SUPPORTED.load(Ordering::Relaxed)
        && remaining >= HUGE_PAGE_SIZE
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

// TODO: add process id handling so that we can retain TLB cache even when switching between processes
impl ArchMMU for ArchMMUImpl {
    type InitResult = usize;

    unsafe fn init(
        hhdm_offset: HhdmOffset,
        hhdm_regions: &mut dyn Iterator<Item = HhdmRegion>,
        executable: ExecutableAddress,
        executable_regions: &mut dyn Iterator<Item = ExecutableRegion>,
    ) -> usize {
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

        let root_page_table = alloc_page_table().expect("early page-table pool exhausted");
        let root_page_table_phys = kernel_virt_to_phys(root_page_table as usize);
        let physical_address_mask = PHYSICAL_ADDRESS_MASK.load(Ordering::Relaxed);
        assert_eq!(root_page_table_phys & !physical_address_mask, 0);

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
                    root_page_table_phys,
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
                    root_page_table_phys,
                    physical_start,
                    virtual_start,
                    physical_end - physical_start,
                    flags,
                );
            }
        }

        root_page_table_phys
    }

    unsafe fn activate(root_page_table_phys: usize, next_entrypoint: *const ()) -> ! {
        unsafe {
            asm!(
                "mov cr3, {root}",
                "jmp {next}",
                root = in(reg) root_page_table_phys,
                next = in(reg) next_entrypoint,
                options(nostack, preserves_flags, noreturn),
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
        // TODO: add locking to prevent concurrent access from different cores.
        let hhdm_offset = hhdm_offset().expect("MMU abstraction is not initialized");
        let physical_address_mask = PHYSICAL_ADDRESS_MASK.load(Ordering::Relaxed);
        assert_ne!(physical_address_mask, 0);
        assert_eq!(root_page_table & !physical_address_mask, 0);

        let mut table = (*hhdm_offset + root_page_table) as *mut Table;
        let page_level = page_size_to_level(page_size);
        let page_size_bytes = page_size_bytes(page_size);
        assert!(flags.contains(MMUMappingFlags::PRESENT));
        assert!(virtual_address < HIGHER_HALF_FIRST_ADDR || !flags.contains(MMUMappingFlags::USER));
        assert!(virtual_address.is_multiple_of(page_size_bytes));
        assert!(physical_address.is_multiple_of(page_size_bytes));
        assert_eq!(physical_address & !physical_address_mask, 0);
        let mut hw_flags = ArchImplFlags::from(flags);

        for shift in SHIFTS.iter().take(page_level) {
            let idx = (virtual_address >> shift) & 0x1ff;
            let entry = unsafe { &mut (*table).entries[idx] };
            if !entry.contains(ArchImplFlags::PRESENT) {
                let new_table = alloc_page_table().expect("early page-table pool exhausted");
                let new_table_phys = kernel_virt_to_phys(new_table as usize);
                assert_eq!(new_table_phys & !physical_address_mask, 0);
                *entry = ArchImplFlags::from_bits_retain(new_table_phys as u64)
                    | ArchImplFlags::PRESENT
                    | ArchImplFlags::WRITE;
            }

            assert!(
                !entry.contains(ArchImplFlags::PAGE_SIZE),
                "mapping collides with a larger page"
            );

            if flags.contains(MMUMappingFlags::USER) {
                *entry |= ArchImplFlags::USER;
            }
            if flags.contains(MMUMappingFlags::WRITE) {
                *entry |= ArchImplFlags::WRITE;
            }

            let table_address_mask = physical_address_mask & !(NORMAL_PAGE_SIZE - 1);
            let table_phys_addr = entry.bits() as usize & table_address_mask;
            table = (*hhdm_offset + table_phys_addr) as *mut Table;
        }

        let idx = (virtual_address >> SHIFTS[page_level]) & 0x1ff;
        let entry = unsafe { &mut (*table).entries[idx] };
        assert!(
            page_size == PageSize::Normal
                || !entry.contains(ArchImplFlags::PRESENT)
                || entry.contains(ArchImplFlags::PAGE_SIZE),
            "large mapping collides with a lower-level page table"
        );
        if page_size != PageSize::Normal {
            hw_flags |= ArchImplFlags::PAGE_SIZE;
        }
        *entry = hw_flags | ArchImplFlags::from_bits_retain(physical_address as u64);
        unsafe {
            invlpg(virtual_address);
        }
    }
}

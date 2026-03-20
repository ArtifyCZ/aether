use core::arch::asm;
use core::mem::zeroed;

const GDT_ENTRIES: usize = 10;
pub const KERNEL_CODE_SEGMENT: u16 = 0x08;
pub const KERNEL_DATA_SEGMENT: u16 = 0x10;
pub const USER_DATA_SEGMENT: u16 = 0x20;
pub const USER_CODE_SEGMENT: u16 = 0x28;

#[repr(C, packed)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

#[repr(C, packed)]
struct TssDescriptor {
    low: GdtEntry,
    base_upper32: u32,
    reserved: u32,
}

#[repr(C, packed)]
struct TssEntry {
    reserved0: u32,
    rsp0: u64, // Stack pointer for Ring 0
    rsp1: u64,
    rsp2: u64,
    reserved1: u64,
    ist: [u64; 7], // Interrupt Stack Table
    reserved2: u64,
    reserved3: u16,
    iopb_offset: u16,
}

#[repr(C, packed)]
struct GdtPtr {
    limit: u16,
    base: u64,
}

static mut GDT: [GdtEntry; GDT_ENTRIES] = unsafe { zeroed() };
static mut TSS: TssEntry = unsafe { zeroed() };

const IST_STACKS_COUNT: usize = 1;
const IST_STACK_SIZE: usize = 8192;
static mut IST_STACKS: [[u8; IST_STACK_SIZE]; IST_STACK_SIZE] = unsafe { zeroed() };

unsafe fn set_entry(num: u16, base: u32, limit: u32, access: u8, gran: u8) {
    unsafe {
        let gdt = &mut GDT[num as usize];

        gdt.base_low = (base & 0xFFFF) as u16;
        gdt.base_middle = ((base >> 16) & 0xFF) as u8;
        gdt.base_high = ((base >> 24) & 0xFF) as u8;

        gdt.limit_low = (limit & 0xFFFF) as u16;
        gdt.granularity = ((limit >> 16) & 0x0F) as u8;

        gdt.granularity |= gran & 0xF0;
        gdt.access = access;
    }
}

unsafe fn set_tss_descriptor(num: u16, base: u64, limit: u32) {
    unsafe {
        set_entry(num, base as u32, limit, 0x89, 0x40);

        let tss_desc: &mut TssDescriptor = core::mem::transmute(&mut GDT[num as usize]);
        tss_desc.base_upper32 = (base >> 32) as u32;
        tss_desc.reserved = 0;
    }
}

#[allow(static_mut_refs)]
pub unsafe fn init() {
    unsafe {
        // Index 0: Null
        // Index 1: Kernel Code (0x08)
        set_entry(1, 0, 0xFFFFFFFF, 0x9A, 0x20);

        // Index 2: Kernel Data (0x10)
        set_entry(2, 0, 0xFFFFFFFF, 0x92, 0x00);

        // Index 3: Dummy/User 32-bit Code (0x18) - Required by some Intel sysret implementations
        set_entry(3, 0, 0xFFFFFFFF, 0xFA, 0x00);

        // Index 4: User Data (0x20) - sysret uses this for SS (STAR[63:48] + 8)
        set_entry(4, 0, 0xFFFFFFFF, 0xF2, 0x00);

        // Index 5: User Code 64 (0x28) - sysret uses this for CS (STAR[63:48] + 16)
        set_entry(5, 0, 0xFFFFFFFF, 0xFA, 0x20);

        // Index 6 & 7: TSS (0x30)
        set_tss_descriptor(6, &raw mut TSS as u64, (size_of::<TssEntry>() - 1) as u32);

        for i in 0..IST_STACKS_COUNT {
            let stack = &raw mut IST_STACKS[i];
            let stack_top = stack.add(IST_STACK_SIZE);
            TSS.ist[i] = stack_top as u64;
        }

        // 0x28: TSS (Occupies two slots: 5 and 6)
        TSS.iopb_offset = size_of::<TssEntry>() as u16;
        set_tss_descriptor(6, &raw const TSS as u64, (size_of::<TssEntry>() - 1) as u32);

        let limit = (size_of_val(&GDT) - 1) as u16;
        let base = &raw const GDT as u64;
        let gdt_ptr = GdtPtr {
            limit,
            base,
        };

        asm!("lgdt [{}]", in(reg) &gdt_ptr);

        asm!(
            // Reload data segments
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            "xor ax, ax",
            "mov fs, ax",
            "mov gs, ax",

            // Reload Code Segment (CS) using lretq
            "push 0x08",         // Push new CS (Selector)
            "lea {tmp}, [66f + rip]",
            "push {tmp}",        // Push new RIP (Label 66)
            ".byte 0x48, 0xcb",  // Manual 'lretq' (REX.W + 0xCB)
            "66:",               // Target label
            tmp = out(reg) _,    // Let Rust choose a scratch register
            options(readonly, nostack, preserves_flags)
        );

        asm!(
            "ltr ax",
            in("ax") 0x30u16,
        );
    }
}

pub unsafe fn set_kernel_stack(stack: usize) {
    unsafe {
        TSS.rsp0 = stack as u64;
    }
}

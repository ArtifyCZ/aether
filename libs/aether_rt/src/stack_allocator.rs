use core::{
    arch::asm,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

static NEXT_AVAILABLE_STACK_TOP: AtomicPtr<u8> = AtomicPtr::new(null_mut());

/// # SAFETY
///
/// Must be called exactly once, during in the init phase of a process.
pub unsafe fn init(stack_base: *mut u8) {
    let next_available_stack_top = unsafe { stack_base.byte_sub(4096) }; // Empty stack guard page
    assert_eq!(
        next_available_stack_top.addr() % 4096,
        0,
        "Next available stack top is not page-aligned!",
    );
    NEXT_AVAILABLE_STACK_TOP.store(next_available_stack_top, Ordering::Relaxed);
}

fn alloc_stack_addr_range(stack_size: usize) -> *mut u8 {
    loop {
        let stack_top = NEXT_AVAILABLE_STACK_TOP.load(Ordering::Acquire);
        if stack_top.is_null() {
            panic!("The stack allocator hasn't been initialized!");
        }
        let stack_base = unsafe {
            stack_top
                .byte_sub(stack_size)
                .map_addr(|stack_base| stack_base & (!0xFFF))
        };
        let next_available_stack_top = unsafe { stack_base.byte_sub(4096) }; // Empty stack guard page
        if NEXT_AVAILABLE_STACK_TOP
            .compare_exchange(
                stack_top,
                next_available_stack_top,
                Ordering::Release,
                Ordering::Relaxed,
            )
            .is_ok()
        {
            return stack_base;
        }
    }
}

/// Allocates a new stack of the given size and returns the new stack's top address
/// (the next byte after the last included)
pub fn allocate(stack_size: usize) -> *mut u8 {
    let stack_base = alloc_stack_addr_range(stack_size);
    let stack_base = unsafe {
        aether_sys::sys_proc_mmap(
            0,
            stack_base,
            stack_size as *mut u8,
            aether_sys::SYS_PROT_READ | aether_sys::SYS_PROT_WRITE,
            0,
        )
        .unwrap()
    };
    if stack_base.addr() < 0x400000 {
        panic!("Mmap failed or returned invalid address!");
    }
    let stack_top = unsafe { stack_base.byte_add(stack_size) };
    stack_top
}

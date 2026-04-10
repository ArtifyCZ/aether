use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::physical_memory_manager::PhysicalMemoryManager;
use crate::platform::virtual_address_allocator::VirtualAddressAllocator;
use crate::platform::virtual_memory_manager_context::VirtualMemoryManagerContext;
use crate::println;
use kernel_hal::mmu::VirtualMemoryMappingFlags;

pub unsafe fn run() {
    println!("[self_test] --- Memory management tests ---");
    unsafe {
        test_pmm_alloc_is_valid();
        test_pmm_alloc_distinct_frames();
        test_pmm_free_and_realloc();
        test_vmm_map_translate_unmap();
    }
    println!("[self_test] --- Memory management tests: PASS ---");
}

/// Allocate one frame and verify it is non-zero and page-aligned.
unsafe fn test_pmm_alloc_is_valid() {
    let frame = unsafe {
        PhysicalMemoryManager::alloc_frame()
            .expect("[self_test] PMM: alloc_frame failed unexpectedly")
    };
    assert_ne!(
        frame.inner(),
        0,
        "[self_test] PMM: allocated frame address must not be zero"
    );
    assert_eq!(
        frame.inner() % PAGE_FRAME_SIZE,
        0,
        "[self_test] PMM: allocated frame must be page-aligned"
    );
    unsafe { PhysicalMemoryManager::free_frame(frame) };
    println!("[self_test] PMM alloc validity: PASS");
}

/// Two consecutive allocations must return distinct frames.
unsafe fn test_pmm_alloc_distinct_frames() {
    let f1 = unsafe {
        PhysicalMemoryManager::alloc_frame()
            .expect("[self_test] PMM: alloc f1 failed")
    };
    let f2 = unsafe {
        PhysicalMemoryManager::alloc_frame()
            .expect("[self_test] PMM: alloc f2 failed")
    };
    assert_ne!(
        f1.inner(),
        f2.inner(),
        "[self_test] PMM: two consecutive allocations must return different frames"
    );
    unsafe {
        // Free in LIFO order to restore original stack state.
        PhysicalMemoryManager::free_frame(f2);
        PhysicalMemoryManager::free_frame(f1);
    }
    println!("[self_test] PMM distinct allocations: PASS");
}

/// Free a frame and reallocate — the LIFO stack must return the same address.
unsafe fn test_pmm_free_and_realloc() {
    let frame = unsafe {
        PhysicalMemoryManager::alloc_frame()
            .expect("[self_test] PMM: alloc failed")
    };
    let addr = frame.inner();
    unsafe { PhysicalMemoryManager::free_frame(frame) };

    let reallocated = unsafe {
        PhysicalMemoryManager::alloc_frame()
            .expect("[self_test] PMM: realloc failed")
    };
    assert_eq!(
        reallocated.inner(),
        addr,
        "[self_test] PMM: frame freed then reallocated must be the same address (LIFO stack)"
    );
    unsafe { PhysicalMemoryManager::free_frame(reallocated) };
    println!("[self_test] PMM free-and-realloc round-trip: PASS");
}

/// Map a physical frame to a fresh virtual page, verify translation, then unmap.
unsafe fn test_vmm_map_translate_unmap() {
    let ctx = unsafe { VirtualMemoryManagerContext::get_kernel_context() };

    let phys = unsafe {
        PhysicalMemoryManager::alloc_frame()
            .expect("[self_test] VMM: failed to allocate physical frame")
    };

    // Allocate a fresh virtual page that is not yet mapped.
    let virt = unsafe { VirtualAddressAllocator::alloc_range(PAGE_FRAME_SIZE) };

    unsafe {
        ctx.map_page(
            virt,
            phys,
            VirtualMemoryMappingFlags::PRESENT | VirtualMemoryMappingFlags::WRITE,
        )
        .expect("[self_test] VMM: map_page failed");
    }

    let translated = unsafe {
        ctx.translate(virt)
            .expect("[self_test] VMM: translate returned an error")
            .expect("[self_test] VMM: translate returned None for a mapped page")
    };

    assert_eq!(
        translated.inner(),
        phys.inner(),
        "[self_test] VMM: translated physical address must match the one used during mapping"
    );

    unsafe {
        ctx.unmap_page(virt).expect("[self_test] VMM: unmap_page failed");
    }

    // The physical frame used for the page content is intentionally not freed
    // here: the intermediate page-table frames allocated during map_page are
    // also not freed (the PMM has no free-list compaction), so returning just
    // the leaf frame would leave the page-table nodes dangling.  These few
    // pages are acceptable overhead for a one-time boot self-test.

    println!("[self_test] VMM map / translate / unmap: PASS");
}

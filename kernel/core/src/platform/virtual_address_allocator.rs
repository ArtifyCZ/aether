use crate::platform::virtual_page_address::VirtualPageAddress;

mod bindings {
    include_bindings!("virtual_address_allocator.rs");
}

pub struct VirtualAddressAllocator;

impl VirtualAddressAllocator {
    pub unsafe fn alloc_range(size: usize) -> VirtualPageAddress {
        unsafe { bindings::vaa_alloc_range(size) }
            .try_into()
            .unwrap()
    }
}

use crate::platform::physical_page_frame::{PhysicalPageFrame, PhysicalPageFrameParseError};
use crate::platform::virtual_page_address::VirtualPageAddress;
use bitflags::bitflags;

mod bindings {
    include_bindings!("virtual_memory_manager.rs");
}

pub(super) const VMM_PAGE_SIZE: usize = bindings::VMM_PAGE_SIZE as usize;

bitflags! {
    pub struct VirtualMemoryMappingFlags: u32 {
        const PRESENT = bindings::vmm_flags_t_VMM_FLAG_PRESENT;
        const WRITE = bindings::vmm_flags_t_VMM_FLAG_WRITE;
        const USER = bindings::vmm_flags_t_VMM_FLAG_USER;
        const EXEC = bindings::vmm_flags_t_VMM_FLAG_EXEC;
        const DEVICE = bindings::vmm_flags_t_VMM_FLAG_DEVICE;
        const NO_CACHE = bindings::vmm_flags_t_VMM_FLAG_NOCACHE;
    }
}

#[derive(Debug)]
pub struct VirtualMemoryManagerContext {
    context: bindings::vmm_context,
}

impl VirtualMemoryManagerContext {
    pub unsafe fn get_kernel_context() -> VirtualMemoryManagerContext {
        unsafe {
            VirtualMemoryManagerContext {
                context: bindings::g_kernel_context,
            }
        }
    }

    pub unsafe fn create() -> VirtualMemoryManagerContext {
        unsafe {
            VirtualMemoryManagerContext {
                context: bindings::vmm_context_create(),
            }
        }
    }

    pub(super) unsafe fn inner(&self) -> &bindings::vmm_context {
        &self.context
    }

    pub(super) unsafe fn inner_mut(&mut self) -> &mut bindings::vmm_context {
        &mut self.context
    }

    /// @TODO: add better errors
    pub unsafe fn map_page(
        &self,
        virtual_page_address: VirtualPageAddress,
        physical_address: PhysicalPageFrame,
        flags: VirtualMemoryMappingFlags,
    ) -> Result<(), ()> {
        if unsafe {
            bindings::vmm_map_page(
                &self.context,
                virtual_page_address.inner(),
                physical_address.inner(),
                flags.bits(),
            )
        } {
            Ok(())
        } else {
            Err(())
        }
    }

    pub unsafe fn translate(
        &self,
        virtual_page_address: VirtualPageAddress,
    ) -> Result<Option<PhysicalPageFrame>, PhysicalPageFrameParseError> {
        let physical_page_frame =
            unsafe { bindings::vmm_translate(&self.context, virtual_page_address.inner()) };
        if physical_page_frame == 0 {
            return Ok(None);
        }

        Ok(Some(PhysicalPageFrame::new(physical_page_frame)?))
    }
}

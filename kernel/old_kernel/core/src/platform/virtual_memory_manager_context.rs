use crate::platform::physical_page_frame::{PhysicalPageFrame, PhysicalPageFrameParseError};
use crate::platform::virtual_page_address::VirtualPageAddress;
use kernel_hal::mmu;
use kernel_hal::mmu::VirtualMemoryMappingFlags;

pub(super) const VMM_PAGE_SIZE: usize = kernel_bindings_gen::VMM_PAGE_SIZE as usize;

#[derive(Debug)]
pub struct VirtualMemoryManagerContext {
    context: usize,
}

impl VirtualMemoryManagerContext {
    pub unsafe fn get_kernel_context() -> VirtualMemoryManagerContext {
        unsafe {
            VirtualMemoryManagerContext {
                context: mmu::get_kernel_context(),
            }
        }
    }

    pub unsafe fn create() -> VirtualMemoryManagerContext {
        unsafe {
            VirtualMemoryManagerContext {
                context: mmu::create_context(),
            }
        }
    }

    pub(super) unsafe fn inner(&self) -> usize {
        self.context
    }

    /// @TODO: add better errors
    pub unsafe fn map_page(
        &self,
        virtual_page_address: VirtualPageAddress,
        physical_address: PhysicalPageFrame,
        flags: VirtualMemoryMappingFlags,
    ) -> Result<(), ()> {
        if unsafe {
            mmu::map_page(
                self.context,
                virtual_page_address.inner(),
                physical_address.inner(),
                flags,
            )
        } {
            Ok(())
        } else {
            Err(())
        }
    }

    pub unsafe fn unmap_page(&self, virtual_page_address: VirtualPageAddress) -> Result<(), ()> {
        if unsafe { mmu::unmap_page(self.context, virtual_page_address.inner()) } {
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
            unsafe { mmu::translate(self.context, virtual_page_address.inner()) };
        if physical_page_frame == 0 {
            return Ok(None);
        }

        Ok(Some(PhysicalPageFrame::new(physical_page_frame)?))
    }
}

pub struct VirtualMemoryManager;

impl VirtualMemoryManager {
    pub unsafe fn init(hhdm_offset: u64) {
        unsafe {
            kernel_hal::mmu::init(hhdm_offset as usize);
        }
    }
}

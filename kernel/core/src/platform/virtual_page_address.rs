use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::virtual_address::{VirtualAddress, VirtualAddressParseError};
use thiserror_no_std::Error;

#[derive(Debug, Copy, Clone)]
pub struct VirtualPageAddress(VirtualAddress);

#[derive(Debug, Error)]
pub enum VirtualPageAddressParseError {
    #[error(transparent)]
    InvalidAddress(#[from] VirtualAddressParseError),
    #[error("Virtual page address is not aligned to the page size")]
    NotAligned,
}

impl VirtualPageAddress {
    pub const fn new(value: usize) -> Result<Self, VirtualPageAddressParseError> {
        if value % PAGE_FRAME_SIZE != 0 {
            return Err(VirtualPageAddressParseError::NotAligned);
        }

        match VirtualAddress::new(value) {
            Ok(virtual_address) => Ok(VirtualPageAddress(virtual_address)),
            Err(err) => Err(VirtualPageAddressParseError::InvalidAddress(err)),
        }
    }

    pub const fn inner(&self) -> usize {
        self.0.inner()
    }

    pub const fn start(&self) -> VirtualAddress {
        self.0
    }

    // Returns the virtual address of the last byte of this virtual page
    pub fn end(&self) -> VirtualAddress {
        self.0 + (PAGE_FRAME_SIZE - 1)
    }

    pub fn next_page(&self) -> VirtualPageAddress {
        Self::new(self.0.inner() + PAGE_FRAME_SIZE).unwrap()
    }
}

impl TryFrom<usize> for VirtualPageAddress {
    type Error = VirtualPageAddressParseError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<VirtualAddress> for VirtualPageAddress {
    type Error = VirtualPageAddressParseError;

    fn try_from(value: VirtualAddress) -> Result<Self, Self::Error> {
        Self::new(value.inner())
    }
}

impl Into<usize> for VirtualPageAddress {
    fn into(self) -> usize {
        self.0.inner()
    }
}

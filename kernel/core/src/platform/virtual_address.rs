use thiserror_no_std::Error;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtualAddress(usize);

#[derive(Debug, Error)]
pub enum VirtualAddressParseError {}

impl VirtualAddress {
    pub const fn new(value: usize) -> Result<Self, VirtualAddressParseError> {
        // @TODO: possibly add validation that the address is in user-space or kernel-space, but not in the hole between
        Ok(VirtualAddress(value))
    }

    pub const fn inner(&self) -> usize {
        self.0
    }
}

impl TryFrom<usize> for VirtualAddress {
    type Error = VirtualAddressParseError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Into<usize> for VirtualAddress {
    fn into(self) -> usize {
        self.0
    }
}

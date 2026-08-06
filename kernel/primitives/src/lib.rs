#![no_std]

use core::ops::Deref;

pub mod sync;

#[repr(transparent)]
pub struct HhdmOffset(usize);

impl HhdmOffset {
    /// # Safety
    ///
    /// The caller has to ensure the offset is valid and the higher-half direct map is set up
    /// and usable.
    pub unsafe fn new(offset: usize) -> HhdmOffset {
        HhdmOffset(offset)
    }
}

impl Deref for HhdmOffset {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<HhdmOffset> for usize {
    fn from(value: HhdmOffset) -> Self {
        value.0
    }
}

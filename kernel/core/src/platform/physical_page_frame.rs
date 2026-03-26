use crate::platform::memory_layout::PAGE_FRAME_SIZE;

#[derive(Debug, Copy, Clone)]
pub struct PhysicalPageFrame(usize);

#[derive(Debug)]
pub enum PhysicalPageFrameParseError {
    NotAligned,
    NullPageFrame,
}

impl PhysicalPageFrame {
    pub const fn new(value: usize) -> Result<Self, PhysicalPageFrameParseError> {
        if value == 0 {
            return Err(PhysicalPageFrameParseError::NullPageFrame);
        }

        if value % PAGE_FRAME_SIZE != 0 {
            return Err(PhysicalPageFrameParseError::NotAligned);
        }

        Ok(PhysicalPageFrame(value))
    }

    pub const fn inner(&self) -> usize {
        self.0
    }

    pub const fn start(&self) -> usize {
        self.0
    }

    pub const fn end(&self) -> usize {
        self.0 + PAGE_FRAME_SIZE - 1
    }
}

impl TryFrom<usize> for PhysicalPageFrame {
    type Error = PhysicalPageFrameParseError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Into<usize> for PhysicalPageFrame {
    fn into(self) -> usize {
        self.0
    }
}

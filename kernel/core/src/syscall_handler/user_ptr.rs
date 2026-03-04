use crate::platform::syscalls::{SyscallError, SyscallReturnValue, SyscallReturnable};
use core::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserPtr<T>(T);

impl SyscallReturnable for UserPtr<usize> {
    fn into_return_value(self) -> SyscallReturnValue {
        SyscallReturnValue(self.0 as u64)
    }
}

impl<T> TryFrom<u64> for UserPtr<*mut T> {
    type Error = SyscallError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value < 0x800000000000 {
            Ok(Self(value as *mut T))
        } else {
            Err(SyscallError::SYS_EFAULT)
        }
    }
}

impl<T> TryFrom<u64> for UserPtr<*const T> {
    type Error = SyscallError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value < 0x800000000000 {
            Ok(Self(value as *const T))
        } else {
            Err(SyscallError::SYS_EFAULT)
        }
    }
}

impl<T> TryFrom<usize> for UserPtr<*mut T> {
    type Error = SyscallError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::try_from(value as u64)
    }
}

impl<T> TryFrom<usize> for UserPtr<*const T> {
    type Error = SyscallError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::try_from(value as u64)
    }
}

impl TryFrom<u64> for UserPtr<usize> {
    type Error = SyscallError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value < 0x800000000000 {
            Ok(Self(value as usize))
        } else {
            Err(SyscallError::SYS_EFAULT)
        }
    }
}

impl TryFrom<usize> for UserPtr<usize> {
    type Error = SyscallError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::try_from(value as u64)
    }
}

impl<T> Deref for UserPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for UserPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

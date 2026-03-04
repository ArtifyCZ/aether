use crate::platform::syscalls::SyscallError;
use crate::syscall_handler::user_ptr::UserPtr;
use core::ops::{Deref, DerefMut};
use core::ptr::slice_from_raw_parts;

pub struct UserSlice<T>(T);

impl<T> TryFrom<(UserPtr<*const T>, usize)> for UserSlice<*const [T]> {
    type Error = SyscallError;

    fn try_from((base, len): (UserPtr<*const T>, usize)) -> Result<Self, Self::Error> {
        let _end: UserPtr<*const T> = UserPtr::try_from(base.addr() + len)?;
        Ok(Self(slice_from_raw_parts(*base, len)))
    }
}

impl<T> UserSlice<*const [T]> {
    pub unsafe fn as_slice(&self) -> &[T] {
        unsafe {
            &*self.0
        }
    }
}

impl<T> Deref for UserSlice<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for UserSlice<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

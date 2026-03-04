use crate::platform::interrupts::Interrupts;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub struct InterruptSafeSpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> InterruptSafeSpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&'_ self) -> InterruptSafeSpinLockGuard<'_, T> {
        InterruptSafeSpinLockGuard::acquire(self)
    }
}

impl<T> Default for InterruptSafeSpinLock<T> where T: Default {
    fn default() -> Self {
        Self::new(T::default())
    }
}

unsafe impl<T: Send> Send for InterruptSafeSpinLock<T> {}

unsafe impl<T: Send> Sync for InterruptSafeSpinLock<T> {}

pub struct InterruptSafeSpinLockGuard<'a, T>(&'a InterruptSafeSpinLock<T>);

impl<'a, T> InterruptSafeSpinLockGuard<'a, T> {
    fn acquire(lock: &'a InterruptSafeSpinLock<T>) -> Self {
        unsafe {
            Interrupts::disable();
        }

        while lock
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        InterruptSafeSpinLockGuard(lock)
    }
}

impl<'a, T> Deref for InterruptSafeSpinLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.data.get() }
    }
}

impl<'a, T> DerefMut for InterruptSafeSpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.data.get() }
    }
}

impl<'a, T> Drop for InterruptSafeSpinLockGuard<'a, T> {
    fn drop(&mut self) {
        self.0.locked.store(false, Ordering::Release);
        unsafe {
            Interrupts::enable();
        }
    }
}

unsafe impl<T: Sync> Sync for InterruptSafeSpinLockGuard<'_, T> {}

use crate::platform::interrupts::Interrupts;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use crate::task_id::TaskId;

#[derive(Debug)]
pub struct InterruptSafeSpinLock<T> {
    locked: AtomicU64,
    data: UnsafeCell<T>,
}

impl<T> InterruptSafeSpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicU64::new(u64::MAX),
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

        let task_id = TaskId::get_current().map(|id| id.get()).unwrap_or(0);

        if lock.locked.load(Ordering::Acquire) == task_id {
            panic!("Attempted to lock on the same thread where this is already locked!");
        }

        while lock
            .locked
            .compare_exchange(u64::MAX, task_id, Ordering::Acquire, Ordering::Relaxed)
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
        self.0.locked.store(u64::MAX, Ordering::Release);
        unsafe {
            Interrupts::enable();
        }
    }
}

unsafe impl<T: Sync> Sync for InterruptSafeSpinLockGuard<'_, T> {}

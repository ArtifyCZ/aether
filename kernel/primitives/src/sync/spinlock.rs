use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct Spinlock<T, TInterruptControl: InterruptControl> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
    _marker: PhantomData<TInterruptControl>,
}

pub trait InterruptControl {
    unsafe fn enable();
    /// Disables interrupts if they were enabled.
    /// Returns whether the interrupts were enabled.
    unsafe fn disable() -> bool;
}

impl<T, TInterruptControl> Spinlock<T, TInterruptControl>
where
    TInterruptControl: InterruptControl,
{
    pub const fn new(inner: T) -> Spinlock<T, TInterruptControl> {
        Spinlock {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(inner),
            _marker: PhantomData,
        }
    }

    pub fn lock(&self) -> SpinlockGuard<'_, T, TInterruptControl> {
        let were_interrupts_enabled = unsafe { TInterruptControl::disable() };

        while self.locked.load(Ordering::Relaxed)
            || self
                .locked
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
        {
            core::hint::spin_loop();
        }

        SpinlockGuard {
            enable_interrupts: were_interrupts_enabled,
            lock: self,
        }
    }
}

unsafe impl<T, TInterruptControl> Send for Spinlock<T, TInterruptControl>
where
    T: Send,
    TInterruptControl: InterruptControl,
{
}

unsafe impl<T, TInterruptControl> Sync for Spinlock<T, TInterruptControl>
where
    T: Send,
    TInterruptControl: InterruptControl,
{
}

pub struct SpinlockGuard<'lock, T: 'lock, TInterruptControl: InterruptControl> {
    enable_interrupts: bool,
    lock: &'lock Spinlock<T, TInterruptControl>,
}

impl<'lock, T, TInterruptControl> Deref for SpinlockGuard<'lock, T, TInterruptControl>
where
    TInterruptControl: InterruptControl,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'lock, T, TInterruptControl> DerefMut for SpinlockGuard<'lock, T, TInterruptControl>
where
    TInterruptControl: InterruptControl,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'lock, T, TInterruptControl> Drop for SpinlockGuard<'lock, T, TInterruptControl>
where
    TInterruptControl: InterruptControl,
{
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);

        if self.enable_interrupts {
            unsafe {
                TInterruptControl::enable();
            }
        }
    }
}

use core::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub struct SpinLock(AtomicBool);

impl SpinLock {
    pub const fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    pub fn lock(&self) -> SpinLockGuard<'_> {
        while self
            .0
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        SpinLockGuard(self)
    }
}

pub struct SpinLockGuard<'a>(&'a SpinLock);

impl Drop for SpinLockGuard<'_> {
    fn drop(&mut self) {
        self.0.0.store(false, Ordering::Release);
    }
}

impl Default for SpinLock {
    fn default() -> Self {
        Self::new()
    }
}

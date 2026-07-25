use core::arch::asm;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

fn interrupts_enabled() -> bool {
    use core::arch::asm;

    unsafe {
        let res: u64;
        #[cfg(target_arch = "x86_64")]
        {
            asm!("pushfq", "pop {}", out(reg) res);
            (res & (1 << 9)) != 0
        }

        #[cfg(target_arch = "aarch64")]
        {
            asm!("mrs {}, daif", out(reg) res);
            // Bit 7 is the I (IRQ) mask bit.
            // If it is 0, interrupts are NOT masked (enabled).
            (res & (1 << 7)) == 0
        }
    }
}

unsafe fn interrupts_enable() {
    use core::arch::asm;

    unsafe {
        #[cfg(target_arch = "x86_64")]
        asm!("sti");

        #[cfg(target_arch = "aarch64")]
        asm!("msr daifclr, #3");
    }
}

unsafe fn interrupts_disable() {
    unsafe {
        #[cfg(target_arch = "x86_64")]
        asm!("cli");

        #[cfg(target_arch = "aarch64")]
        asm!("msr daifset, #3", "dmb sy");
    }
}

pub struct Spinlock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> Spinlock<T> {
    pub const fn new(inner: T) -> Spinlock<T> {
        Spinlock {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(inner),
        }
    }

    pub fn lock(&self) -> SpinlockGuard<'_, T> {
        let were_interrupts_enabled = interrupts_enabled();
        if were_interrupts_enabled {
            unsafe {
                interrupts_disable();
            }
        }

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

unsafe impl<T: Send> Send for Spinlock<T> {}
unsafe impl<T: Send> Sync for Spinlock<T> {}

pub struct SpinlockGuard<'lock, T: 'lock> {
    enable_interrupts: bool,
    lock: &'lock Spinlock<T>,
}

unsafe impl<T: Send> Send for SpinlockGuard<'_, T> {}
unsafe impl<T: Sync> Sync for SpinlockGuard<'_, T> {}

impl<'lock, T> Deref for SpinlockGuard<'lock, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'lock, T> DerefMut for SpinlockGuard<'lock, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'lock, T> Drop for SpinlockGuard<'lock, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);

        if self.enable_interrupts {
            unsafe {
                interrupts_enable();
            }
        }
    }
}

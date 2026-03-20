use core::arch::asm;
use core::ptr::{NonNull, null_mut};

unsafe fn msr_tpidr_el1_write(ptr: NonNull<CpuLocalStorage>) {
    let ptr = ptr.as_ptr();
    asm!("msr tpidr_el1, {}", in(reg) ptr);
}

unsafe fn msr_tpidr_el1_read() -> Option<NonNull<CpuLocalStorage>> {
    let ptr: *mut CpuLocalStorage;
    asm!("mrs {}, tpidr_el1", out(reg) ptr);
    NonNull::new(ptr)
}

#[repr(C)]
pub struct CpuLocalStorage {
    task_id: u64,
}

static mut CPU_LOCAL: CpuLocalStorage = CpuLocalStorage { task_id: 0 };

pub unsafe fn init() {
    unsafe {
        let ptr = &raw mut CPU_LOCAL;
        let ptr = NonNull::new(ptr).unwrap();
        msr_tpidr_el1_write(ptr);
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn cpu_local_get() -> *mut CpuLocalStorage {
    unsafe { get().map(|ptr| ptr.as_ptr()).unwrap_or(null_mut()) }
}

pub unsafe fn get() -> Option<NonNull<CpuLocalStorage>> {
    unsafe { msr_tpidr_el1_read() }
}

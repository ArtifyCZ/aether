#![no_std]

mod panic;
mod start;

#[unsafe(no_mangle)]
pub extern "C" fn __rust_probestack() {
    // This is called by the compiler to ensure
    // stack pages are touched in order.
}

#[unsafe(no_mangle)]
pub extern "C" fn _Unwind_Resume() {
    loop {}
}

// Sometimes required by formatting logic when SSE is disabled
// but the library was compiled expecting it.
#[unsafe(no_mangle)]
pub extern "C" fn __errno_location() -> *mut i32 {
    static mut ERRNO: i32 = 0;
    unsafe { core::ptr::addr_of_mut!(ERRNO) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if dest.addr() < src.addr() {
        return memcpy(dest, src, n);
    }
    let mut i = n;
    while i > 0 {
        i -= 1;
        *dest.add(i) = *src.add(i);
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *s.add(i) = c as u8;
        i += 1;
    }
    s
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return a as i32 - b as i32;
        }
    }
    0
}

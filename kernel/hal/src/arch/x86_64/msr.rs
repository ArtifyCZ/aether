use core::arch::asm;
use core::mem::zeroed;

const MSR_EFER: u32 = 0xC0000080;
const MSR_STAR: u32 = 0xC0000081;
const MSR_LSTAR: u32 = 0xC0000082;
const MSR_FMASK: u32 = 0xC0000084;
const EFER_SCE: u32 = 1 << 0;

unsafe fn wrmsr(msr: u32, val: u64) {
    let low: u32 = (val & 0xFFFFFFFF) as u32;
    let high: u32 = ((val >> 32) & 0xFFFFFFFF) as u32;
    unsafe {
        asm!("wrmsr", in("ecx") msr, in("eax") low, in("edx") high);
    }
}

#[repr(C)]
struct CpuLocalStorage {
    kernel_stack: u64,
    user_rsp_scratch: u64,
    task_id: u64,
}

static mut CPU_LOCAL_STORAGE: CpuLocalStorage = unsafe { zeroed() };

unsafe extern "C" {
    fn syscalls_raw_handler();
}

pub unsafe fn init() {
    unsafe {
        // Point to the syscall assembly entry
        wrmsr(MSR_LSTAR, syscalls_raw_handler as *const () as u64);

        // Setup segments
        // Base is now Index 3 (User 32 Code)
        // SS = Index 4 (0x23), CS = Index 5 (0x2B)
        let star: u64 = 0 | (0x001B << 48) | (0x0008 << 32);
        wrmsr(MSR_STAR, star);

        // Disable interrupts on entry (IF bit = 0x200)
        wrmsr(MSR_FMASK, 0x200);

        // Enable system call extension
        // We should read first to be safe, but bit 0 is SCE
        asm!(
            "rdmsr",
            "or eax, {sce}",
            "wrmsr",
            in("ecx") MSR_EFER,
            sce = const EFER_SCE,
            out("eax") _, // rdmsr writes to eax/edx
            out("edx") _,
            options(preserves_flags, nomem)
        );

        // Set the KERNEL_GS_BASE to our local storage
        // When swapgs is called in kernel entry, GS will point here
        wrmsr(0xC0000101, &raw mut CPU_LOCAL_STORAGE as u64);
        wrmsr(0xC0000102, &raw mut CPU_LOCAL_STORAGE as u64);
        CPU_LOCAL_STORAGE.kernel_stack = 0;
        CPU_LOCAL_STORAGE.user_rsp_scratch = 0;
        CPU_LOCAL_STORAGE.task_id = 0;
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn msr_set_kernel_stack(stack: u64) {
    unsafe {
        set_kernel_stack(stack as usize);
    }
}

pub unsafe fn set_kernel_stack(stack: usize) {
    unsafe {
        CPU_LOCAL_STORAGE.kernel_stack = stack as u64;
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn msr_get_task_id() -> u64 {
    unsafe {
        get_task_id()
    }
}

pub unsafe fn get_task_id() -> u64 {
    unsafe {
        CPU_LOCAL_STORAGE.task_id
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn msr_set_task_id(task_id: u64) {
    unsafe {
        set_task_id(task_id);
    }
}

pub unsafe fn set_task_id(task_id: u64) {
    unsafe {
        CPU_LOCAL_STORAGE.task_id = task_id;
    }
}

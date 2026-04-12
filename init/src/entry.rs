use core::{
    arch::asm,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use aether_rt::process::StartupInfo;
use init_contract_rust::boot_info;

/// Entrypoint to the init program
///
/// Note that normally, aether_rt the runtime library would provide the entrypoint.
/// However, the usual Aether userspace ABI requires StartupInfo to be passed,
/// which the kernel does not know about and neither should.
/// Therefore, in the init program we have to make an exception
/// where the actual entrypoint commits to an Aether init program ABI.
/// The Aether init program ABI, otherwise also called init_contract,
/// does get provided with the boot_info struct.
///
/// Because we don't want to reimplement the Aether runtime library's initialization,
/// we create the StartupInfo struct ourselves and pass it to the Aether runtime library.
/// Specifically, we create a StartupInfo struct on its own page,
/// and then call the usual entrypoint _start provided by the Aether runtime library.
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn _entry(boot_info: *mut boot_info) -> ! {
    #[cfg(target_arch = "x86_64")]
    core::arch::naked_asm!(
        "xor rbp, rbp",      // Clear frame pointer for clean backtrace
        "mov rdi, rdi",      // boot_info is already in rdi, but this is for clarity
        "and rsp, -16",      // Align stack to 16 bytes
        "sub rsp, 8",        // Standard ABI: stack should be (16n + 8) at function entry
                             // (because the 'call' instruction pushes an 8-byte return address)
        "sub rsp, 16",
        "mov [rsp + 8], rdi", // Save the boot_info ptr on the stack
        "call {prep_stack}",
        "mov rdi, [rsp + 8]", // Restore the boot_info ptr from the stack

        "mov rsi, rsp",
        "and rsi, -0x1000", // Align the previous stack to page boundary (so that we get the base)

        "and rax, -16", // Align the new stack to 16 bytes
        "sub rax, 8", // Subtract 8 bytes to account for the stack frame
        "mov rsp, rax", // Switch to the new stack

        "mov rdi, rdi", // Prepare the boot_info ptr for entry (1st arg)
        "mov rsi, rsi", // Prepare the prev_stack_base ptr for entry (2nd arg)
        "call {entry}",
        entry = sym entry,
        prep_stack = sym prep_stack,
    );

    #[cfg(target_arch = "aarch64")]
    core::arch::naked_asm!(
        "mov x29, #0",
        "mov x30, #0",
        "mov x0, x0", // boot_info is already in x0, but this is for clarity
        "mov x1, sp",
        "and x1, x1, #0xfffffffffffffff0",
        "sub x1, x1, #16",
        "mov sp, x1",
        "sub sp, sp, #16",

        "str x0, [sp, #8]", // Save the boot_info ptr on the stack before call
        "bl {prep_stack}",
        "ldr x1, [sp, #8]", // Restore the boot_info ptr from the stack after call

        "mov x2, sp", // Prepare the prev_stack_base ptr for entry (2nd arg)
        "and x2, x2, #0xfffffffffffff000", // Align the previous stack ptr to page boundary (so that we get the base)

        "mov x29, #0", // Set the frame pointer to 0
        "mov x30, #0", // Set the link register to 0
        "and x0, x0, #0xfffffffffffffff0", // Align the stack to 16 bytes
        "sub x0, x0, #16", // Subtract 16 bytes to account for the stack frame
        "mov sp, x0", // Switch to the new stack

        "mov x0, x1", // Prepare the boot_info ptr for entry (1st arg)
        "mov x1, x2", // Prepare the prev_stack_base ptr for entry (2nd arg)
        "bl {entry}",
        entry = sym entry,
        prep_stack = sym prep_stack,
    );
}

/// This function exists because we can't rely on panics working,
/// as the Aether runtime library is not initialized yet.
///
/// Exiting the program is therefore the safest option in this early stage.
unsafe fn early_exit() -> ! {
    unsafe {
        aether_sys::sys_exit();
    }
    // If sys_exit fails, we can't continue, so we at the very least loop forever.
    loop {}
}

// The size of the new stack
const STACK_SIZE: usize = 0x4000;

/// The Aether init program ABI provides us with a single page of stack.
/// Because of this, we have to allocate a new stack to be actually used,
/// but of the size we need.
///
/// This function is called as the first thing by the Aether init program,
/// before `entry` is called, actually in fact, before any code at all.
/// It returns a new stack's top address, which is then switched to.
unsafe extern "C" fn prep_stack() -> *mut u8 {
    unsafe {
        let sp: *mut u8;
        #[cfg(target_arch = "x86_64")]
        asm!(
            "mov {}, rsp",
            out(reg) sp,
            options(nomem, nostack),
        );
        #[cfg(target_arch = "aarch64")]
        asm!(
            "mov {}, sp",
            out(reg) sp,
            options(nomem, nostack),
        );
        let prev_stack_base = sp.map_addr(|sp| sp & (!0xfff)); // Align to 4KB

        // Ensure an empty guard page between the new stack and the old one
        let new_stack_top = prev_stack_base.byte_sub(4096);

        // Allocate the new stack
        let new_stack_base = new_stack_top.byte_sub(STACK_SIZE);
        let new_stack_base = match aether_sys::sys_proc_mmap(
            0,
            new_stack_base,
            STACK_SIZE as *mut u8,
            aether_sys::SYS_PROT_READ | aether_sys::SYS_PROT_WRITE,
            0,
        ) {
            Ok(stack_base) => stack_base,
            Err(_) => {
                // If we can't allocate the new stack, we can't continue
                early_exit()
            }
        };

        NEW_STACK_BASE = new_stack_base;
        new_stack_base.byte_add(STACK_SIZE)
    }
}

unsafe extern "C" {
    fn _start(startup_info: *const StartupInfo) -> !;
}

static mut NEW_STACK_BASE: *mut u8 = null_mut();
static BOOT_INFO: AtomicPtr<boot_info> = AtomicPtr::new(null_mut());

const STARTUP_INFO_ADDR: usize = 0x8000_0000; // @TODO: choose a better address

unsafe extern "C" fn entry(boot_info: *mut boot_info, prev_stack_base: *mut u8) -> ! {
    unsafe {
        // The address of the previous stack base
        // This is provided so that we can unmap the old stack to save memory
        // @TODO: Unmap the old stack
        let _ = prev_stack_base;

        BOOT_INFO.store(boot_info, Ordering::Release);

        let Ok(startup_info_ptr) = aether_sys::sys_proc_mmap(
            0,
            STARTUP_INFO_ADDR as *mut u8,
            size_of::<StartupInfo>() as *mut u8,
            aether_sys::SYS_PROT_READ | aether_sys::SYS_PROT_WRITE,
            0,
        ) else {
            early_exit()
        };
        let startup_info_ptr = startup_info_ptr.cast();
        let startup_info = StartupInfo {
            magic: StartupInfo::MAGIC,
            version: 1,
            stack_base: NEW_STACK_BASE,
        };
        core::ptr::write(startup_info_ptr, startup_info);

        _start(startup_info_ptr)
    }
}

pub fn get_boot_info_ptr() -> *mut boot_info {
    BOOT_INFO.load(Ordering::Acquire)
}

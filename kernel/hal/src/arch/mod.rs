#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "aarch64")]
use self::aarch64 as implementation;

#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
use self::x86_64 as implementation;

pub mod cpu {
    pub use super::implementation::cpu::hcf;
}

pub mod early_console {
    pub use super::implementation::early_console::{disable, init, write};
}

pub mod emergency_console {
    pub use super::implementation::emergency_console::{init, write};
}

pub mod mmu {
    pub use super::implementation::mmu::{
        create_context, get_kernel_context, init, map_page, translate, unmap_page,
    };
}

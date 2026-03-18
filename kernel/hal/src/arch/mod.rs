#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "aarch64")]
use self::aarch64 as implementation;

#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
use self::x86_64 as implementation;

pub mod mmu {
    pub use super::implementation::mmu::{create_context, init, map_page, translate, unmap_page};
}

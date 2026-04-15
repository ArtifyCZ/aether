#![no_std]
#![no_main]

use core::ptr::NonNull;

use crate::early_allocator::EarlyAllocator;
use crate::proxy_allocator::ProxyAllocator;
use kernel_core::boot::BootInfo;
use limine::BaseRevision;
use limine::request::{
    ExecutableAddressRequest, ExecutableCmdlineRequest, FramebufferRequest, HhdmRequest,
    MemoryMapRequest, ModuleRequest, RequestsEndMarker, RequestsStartMarker, RsdpRequest,
    StackSizeRequest,
};

mod early_allocator;
mod proxy_allocator;
mod start;

#[used]
#[unsafe(link_section = ".limine_requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static STACK_SIZE_REQUEST: StackSizeRequest = StackSizeRequest::new().with_size(0x8_0000); // 128 kiB

#[used]
#[unsafe(link_section = ".limine_requests")]
static MEMMAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static KERNEL_REQUEST: ExecutableAddressRequest = ExecutableAddressRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static MODULE_REQUEST: ModuleRequest = ModuleRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static EXEC_CMDLINE_REQUEST: ExecutableCmdlineRequest = ExecutableCmdlineRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests_start")]
static _REQUESTS_START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".limine_requests_end")]
static _REQUESTS_END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

#[global_allocator]
static PROXY_ALLOCATOR: ProxyAllocator = unsafe { ProxyAllocator::init() };

static EARLY_ALLOCATOR: EarlyAllocator = unsafe { EarlyAllocator::init() };

struct LimineBootInfo;

impl BootInfo for LimineBootInfo {
    fn get_modules(&self) -> impl Iterator<Item = kernel_core::boot::BootModule<'static>> {
        let module_response = unsafe { MODULE_REQUEST.get_response().unwrap() };
        let modules = module_response.modules();
        modules.iter().map(|module| kernel_core::boot::BootModule {
            name: module.string(),
            data: unsafe {
                core::slice::from_raw_parts(module.addr() as *const u8, module.size() as usize)
            },
        })
    }

    fn get_framebuffer(&self) -> Option<kernel_core::boot::BootFramebuffer> {
        let framebuffer_response = FRAMEBUFFER_REQUEST.get_response()?;
        let framebuffer = framebuffer_response.framebuffers().next()?;
        Some(kernel_core::boot::BootFramebuffer {
            address: NonNull::new(framebuffer.addr())?,
            width: framebuffer.width() as usize,
            height: framebuffer.height() as usize,
            pitch: framebuffer.pitch() as usize,
            bpp: framebuffer.bpp() as usize,
        })
    }
}

unsafe fn main() -> ! {
    assert!(BASE_REVISION.is_supported());

    unsafe {
        PROXY_ALLOCATOR.switch_to_early_allocator(&raw const EARLY_ALLOCATOR);
    }

    let cmdline = EXEC_CMDLINE_REQUEST.get_response().unwrap().cmdline();
    let cmdline = cmdline.to_string_lossy();

    kernel_core::main(
        &cmdline,
        HHDM_REQUEST.get_response().unwrap().offset(),
        (MEMMAP_REQUEST.get_response().unwrap() as *const _
            as *mut limine::response::MemoryMapResponse)
            .cast(),
        RSDP_REQUEST.get_response().unwrap().address() as u64,
        |paged_allocator| unsafe {
            PROXY_ALLOCATOR.switch_to_paged_allocator(paged_allocator);
        },
        LimineBootInfo,
    )
}

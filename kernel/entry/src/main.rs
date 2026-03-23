#![no_std]
#![no_main]

use limine::BaseRevision;
use limine::request::{
    ExecutableAddressRequest, FramebufferRequest, HhdmRequest, MemoryMapRequest, ModuleRequest,
    RequestsEndMarker, RequestsStartMarker, RsdpRequest, StackSizeRequest,
};

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
#[unsafe(link_section = ".limine_requests_start")]
static _REQUESTS_START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".limine_requests_end")]
static _REQUESTS_END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

unsafe fn main() -> ! {
    assert!(BASE_REVISION.is_supported());

    let framebuffer_response: *mut kernel_bindings_gen::limine_framebuffer_response =
        (FRAMEBUFFER_REQUEST.get_response().unwrap() as *const _
            as *mut limine::response::FramebufferResponse)
            .cast();
    let framebuffer = unsafe { framebuffer_response.read().framebuffers.read() };

    unsafe {
        kernel_core::main(
            HHDM_REQUEST.get_response().unwrap().offset(),
            (MEMMAP_REQUEST.get_response().unwrap() as *const _
                as *mut limine::response::MemoryMapResponse)
                .cast(),
            framebuffer,
            (MODULE_REQUEST.get_response().unwrap() as *const _
                as *mut limine::response::ModuleResponse)
                .cast(),
            RSDP_REQUEST.get_response().unwrap().address() as u64,
        );
    }

    loop {}
}

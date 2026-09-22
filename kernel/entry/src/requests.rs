use atom_primitives::HhdmOffset;
use limine::framebuffer::Framebuffer;
use limine::memmap::Entry;
use limine::request::{
    ExecutableAddressRequest, FramebufferRequest, HhdmRequest, MemmapRequest, StackSizeRequest,
};
use limine::{BaseRevision, RequestsEndMarker, RequestsStartMarker};

#[used]
#[unsafe(link_section = ".limine_requests_start")]
static _REQUESTS_START: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".limine_requests_end")]
static _REQUESTS_END: RequestsEndMarker = RequestsEndMarker::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static _BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static _STACK_SIZE_REQUEST: StackSizeRequest = StackSizeRequest::new(0x10000);

#[used]
#[unsafe(link_section = ".limine_requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

pub fn assert_compatible_base_revision() {
    assert!(
        _BASE_REVISION.is_supported()
            || _BASE_REVISION
                .actual_revision()
                .is_some_and(|revision| revision >= 4),
        "Limine base revision 4 or newer is required"
    );
}

pub fn framebuffer() -> Option<&'static Framebuffer> {
    let framebuffers = FRAMEBUFFER_REQUEST.response()?.framebuffers();
    if framebuffers.is_empty() {
        None
    } else {
        Some(framebuffers[0])
    }
}

#[used]
#[unsafe(link_section = ".limine_requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

pub fn hhdm_offset() -> HhdmOffset {
    // Safety: HhdmOffset::new expects the HHDM offset to be correct,
    // and we are reading it from the Limine response.
    unsafe { HhdmOffset::new(HHDM_REQUEST.response().unwrap().offset as usize) }
}

#[used]
#[unsafe(link_section = ".limine_requests")]
static MEMORY_MAP_REQUEST: MemmapRequest = MemmapRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static EXECUTABLE_ADDRESS_REQUEST: ExecutableAddressRequest = ExecutableAddressRequest::new();

pub fn memory_regions() -> &'static [&'static Entry] {
    MEMORY_MAP_REQUEST.response().unwrap().entries()
}

pub fn executable_address() -> (usize, usize) {
    let response = EXECUTABLE_ADDRESS_REQUEST.response().unwrap();
    (
        response.physical_base as usize,
        response.virtual_base as usize,
    )
}

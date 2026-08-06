use atom_primitives::HhdmOffset;
use limine::framebuffer::Framebuffer;
use limine::memmap::Entry;
use limine::request::{FramebufferRequest, HhdmRequest, MemmapRequest, StackSizeRequest};
use limine::{BaseRevision, RequestsEndMarker, RequestsStartMarker};

#[unsafe(link_section = ".limine_requests_start")]
static _REQUESTS_START: RequestsStartMarker = RequestsStartMarker::new();

#[unsafe(link_section = ".limine_requests_end")]
static _REQUESTS_END: RequestsEndMarker = RequestsEndMarker::new();

#[unsafe(link_section = ".limine_requests")]
static _BASE_REVISION: BaseRevision = BaseRevision::new();

#[unsafe(link_section = ".limine_requests")]
static _STACK_SIZE_REQUEST: StackSizeRequest = StackSizeRequest::new(0x10000);

#[unsafe(link_section = ".limine_requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

pub fn framebuffer() -> Option<&'static Framebuffer> {
    let framebuffers = FRAMEBUFFER_REQUEST.response()?.framebuffers();
    if framebuffers.is_empty() {
        None
    } else {
        Some(framebuffers[0])
    }
}

#[unsafe(link_section = ".limine_requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

pub fn hhdm_offset() -> HhdmOffset {
    // Safety: HhdmOffset::new expects the HHDM offset to be correct,
    // and we are reading it from the Limine response.
    unsafe { HhdmOffset::new(HHDM_REQUEST.response().unwrap().offset as usize) }
}

#[unsafe(link_section = ".limine_requests")]
static MEMORY_MAP_REQUEST: MemmapRequest = MemmapRequest::new();

pub fn memory_regions() -> &'static [&'static Entry] {
    MEMORY_MAP_REQUEST.response().unwrap().entries()
}

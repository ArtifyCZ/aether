use limine::framebuffer::Framebuffer;
use limine::request::{FramebufferRequest, HhdmRequest, StackSizeRequest};
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

#[unsafe(link_section = ".limine_requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

pub fn framebuffer() -> Option<&'static Framebuffer> {
    let framebuffers = FRAMEBUFFER_REQUEST.response()?.framebuffers();
    if framebuffers.is_empty() {
        None
    } else {
        Some(framebuffers[0])
    }
}

pub fn hhdm_offset() -> usize {
    HHDM_REQUEST.response().unwrap().offset as usize
}

pub trait ArchInterrupts {
    unsafe fn enable();
    unsafe fn disable();
    unsafe fn are_enabled() -> bool;
}

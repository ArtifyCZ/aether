#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    unsafe { loop {} }
}

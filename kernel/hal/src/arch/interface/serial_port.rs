pub trait ArchSerialPort {
    type InitConfig;

    /// Initializes serial port
    ///
    /// # Safety
    ///
    /// This should be the only way to create an instance.
    /// There must be at most one instance at a time.
    unsafe fn init(config: Self::InitConfig) -> Self;
    /// Checks whether the transmit buffer is empty
    ///
    /// # Safety
    ///
    /// Shouldn't be called more than once at a time.
    unsafe fn is_transmit_empty(&mut self) -> bool;
    /// Sends a byte to the serial
    ///
    /// # Safety
    ///
    /// Shouldn't be called more than once at a time.
    unsafe fn send_byte(&mut self, byte: u8);
}

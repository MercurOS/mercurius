/// Hardware independent UART interface.
pub trait Uart {
    /// Initialize the UART device.
    fn init(&mut self, stop_bits: StopBits);

    /// Send data over UART.
    ///
    /// Calling this function should block until the full string has successfully been
    /// sent.
    fn write(&mut self, string: &str);
}

#[derive(PartialEq)]
pub enum StopBits {
    OneStopBit,
    TwoStopBits,
}

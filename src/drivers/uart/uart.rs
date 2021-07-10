/// Hardware independent UART interface.
pub trait Uart {
    /// Initialize the UART device.
    fn init(&mut self, stop_bits: StopBits);

    /// Send a byte over UART.
    ///
    /// Calling this function should block until the byte has successfully been
    /// sent.
    fn send(&mut self, data: u8);

    /// Send a string over UART.
    fn write(&mut self, string: &str) {
        for byte in string.bytes() {
            self.send(byte);
        }
    }
}

#[derive(PartialEq)]
pub enum StopBits {
    OneStopBit,
    TwoStopBits,
}

impl core::fmt::Write for &mut dyn Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Uart::write(*self, s);
        Ok(())
    }
}

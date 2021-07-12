use lazy_static::lazy_static;
use spin::Mutex;

use crate::drivers::uart::{self, Uart};

type UartDevice = dyn Uart + Send + 'static;

lazy_static! {
    pub static ref WRITER: Mutex<&'static mut UartDevice> = Mutex::new({
        #[cfg(feature = "fu740")]
        let uart = unsafe {
            uart::fu740_c000::UartFu740::new(0).unwrap()
        };

        #[cfg(feature = "qemu")]
        let uart = unsafe {
            uart::ns16550a::UartNs16550a::new(0x1000_0000 as *const core::ffi::c_void)
        };

        uart.init(uart::StopBits::OneStopBit);

        uart
    });
}

impl core::fmt::Write for &mut UartDevice {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Uart::write(*self, s);
        Ok(())
    }
}

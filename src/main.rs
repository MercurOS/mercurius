#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(feature = "fu740")]
    {
        use mercuros_mercurius::drivers::uart::fu740_c000::{UART, StopBits};

        // assume fu740_c000 and identity-mapped memory
        let uart = unsafe { UART::new(0).unwrap() };

        uart.set_tx_watermark(None);
        uart.set_rx_watermark(None);

        uart.tx_enable(StopBits::OneStopBit);

        uart.write("Hello World!\r\n");
    }

    #[cfg(feature = "qemu")]
    {
        use mercuros_mercurius::drivers::uart::ns16550a::UART;

        let uart_base = 0x1000_0000 as *const core::ffi::c_void;
        let uart = unsafe { UART::new(uart_base) };

        uart.set_word_length(8);
        uart.fifo_enable();

        uart.write("Hello World!\r\n");
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

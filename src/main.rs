#![no_std]
#![no_main]

use core::panic::PanicInfo;
use mercuros_mercurius::drivers::uart::{StopBits, Uart};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let uart = {
        #[cfg(feature = "fu740")]
        {
            use mercuros_mercurius::drivers::uart::fu740_c000::UartFu740;

            // assume fu740_c000 and identity-mapped memory
            unsafe { UartFu740::new(0).unwrap() }
        }

        #[cfg(feature = "qemu")]
        {
            use mercuros_mercurius::drivers::uart::ns16550a::UartNs16550a;

            // assume QEMU and identity-mapped memory
            let uart_base = 0x1000_0000 as *const core::ffi::c_void;
            unsafe { UartNs16550a::new(uart_base) }
        }
    };

    uart.init(StopBits::OneStopBit);

    uart.write("Hello World!\r\n");

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

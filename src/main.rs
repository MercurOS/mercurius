#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

use mercuros_mercurius::{
    serial,
    drivers::uart::Uart,
    fdt::{Fdt, FdtError},
};

#[no_mangle]
pub extern "C" fn _start(dtb: *const core::ffi::c_void) -> ! {
    serial::WRITER.lock().write_str("Hello World!\r\n").unwrap();

    match unsafe { Fdt::from_ptr(dtb) } {
        Ok(fdt) => {
            serial::WRITER.lock().write_str("\r\nFDT:\r\n").unwrap();
            for node in fdt.nodes() {
                if let Some(name) = node {
                    for byte in name {
                        serial::WRITER.lock().send(*byte);
                    }
                    serial::WRITER.lock().write("\r\n");
                }
            }
        },
        Err(FdtError::IncompatibleVersion) => {
            serial::WRITER.lock().write_str("Bad FDT version!\r\n").unwrap();
        },
        Err(_) => {
            serial::WRITER.lock().write_str("Bad FDT!\r\n").unwrap();
        },
    };

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial::WRITER.lock().write_fmt(format_args!("[PANIC] {}\r\n", info)).unwrap();
    loop {}
}

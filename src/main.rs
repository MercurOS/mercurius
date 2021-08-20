#![no_std]
#![no_main]

use core::convert::TryInto;
use core::fmt::Write;
use core::panic::PanicInfo;

use mercuros_uefi::{
    MemoryMap,
    api::boot_services::memory::{
        EFI_LOADER_DATA,
        EFI_BOOT_SERVICES_CODE,
        EFI_BOOT_SERVICES_DATA,
        EFI_CONVENTIONAL_MEMORY,
    },
};
use mercuros_mercurius::{
    serial,
    drivers::uart::Uart,
    fdt::{Fdt, FdtError},
    memory::frame::Buddy,
};

const PAGE_SIZE: u64 = 4096;

#[no_mangle]
pub extern "C" fn _start(dtb: *const core::ffi::c_void, mmap: *const MemoryMap) -> ! {
    serial::WRITER.lock().write_str("Hello World!\r\n").unwrap();

    match unsafe { Fdt::from_ptr(dtb) } {
        Ok(fdt) => {
            serial::WRITER.lock().write_str("\r\nFDT:\r\n").unwrap();
            for node in fdt.nodes() {
                if let Some(name) = node {
                    serial::WRITER.lock().write("/");
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

    // Find the first free memory region and set up a page frame allocation
    // table there. Mark allocation status of any memory regions falling within
    // the table coverage.
    let mut buddy: Option<&'static mut Buddy> = None;
    for descriptor in unsafe { &*mmap } {
        match buddy {
            None => {
                if descriptor.r#type == EFI_CONVENTIONAL_MEMORY {
                    // SAFETY: according to the memory map, this region is unoccupied and
                    // therefore safe to write to.
                    unsafe {
                        let raw_buddy = Buddy::new(
                            descriptor.physical_start,
                            descriptor.physical_start
                        );

                        if descriptor.number_of_pages > 1 {
                            raw_buddy.free(
                                descriptor.physical_start + PAGE_SIZE,
                                descriptor.number_of_pages as usize - 1
                            );
                        }

                        buddy = Some(raw_buddy);
                    }
                }
            },
            Some(ref mut buddy) => {
                // FIXME: EFI_LOADER_DATA regions might contain the kernel
                if descriptor.r#type == EFI_LOADER_DATA ||
                    descriptor.r#type == EFI_BOOT_SERVICES_CODE ||
                    descriptor.r#type == EFI_BOOT_SERVICES_DATA ||
                    descriptor.r#type == EFI_CONVENTIONAL_MEMORY
                {
                    buddy.free(
                        descriptor.physical_start,
                        descriptor.number_of_pages.try_into().unwrap()
                    );
                }
            },
        };
    }

    serial::WRITER.lock().write_str("\r\nAvailable physical memory:\r\n").unwrap();
    if let Some(ref buddy) = buddy {
        serial::WRITER.lock().write_fmt(format_args!("{:?}", buddy)).unwrap();
    }

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial::WRITER.lock().write_fmt(format_args!("[PANIC] {}\r\n", info)).unwrap();
    loop {}
}

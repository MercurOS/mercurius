//! UART driver for NS16550a (QEMU).

use crate::memory::Register;
use crate::io::Io;

// Memory Map
// 0x00 - RBR (RO) / THR (WO)
// 0x01 - IER
// 0x02 - IIR (RO) / FCR (WO)
// 0x03 - LCR
// 0x04 - MCR
// 0x05 - LSR
// 0x06 - MSR
// 0x07 - SCR
#[repr(packed)]
pub struct UART {
    // data buffers
    rbr_thr: Register<u8>,

    // interrupt enable
    ier: Register<u8>,

    // interrupt ident. / FIFO control
    iir_fcr: Register<u8>,

    // line control
    lcr: Register<u8>,

    // modem control
    mcr: Register<u8>,

    // line status
    lsr: Register<u8>,

    // modem status
    msr: Register<u8>,

    // scratch register
    scr: Register<u8>,
}

bitflags! {
    struct FcrFlags: u8 {
        const FIFO_ENABLE = 0x01;
    }
}

bitflags! {
    struct LcrFlags: u8 {
        const WORD_LENGTH_SELECT0 = 0x01;
        const WORD_LENGTH_SELECT1 = 0x02;
    }
}

impl UART {
    // The unsafeness here depends on platform and virtual memory layout
    pub unsafe fn new(base_address: *const core::ffi::c_void) -> &'static mut UART {
        &mut *(base_address as *mut Self)
    }

    pub fn send(&mut self, data: u8) {
        self.rbr_thr.write(data);
    }

    pub fn set_word_length(&mut self, _length: usize) {
        // TODO: adjustable word length
        let lcr = (LcrFlags::WORD_LENGTH_SELECT0 | LcrFlags::WORD_LENGTH_SELECT1).bits;
        self.lcr.write(lcr);
    }

    pub fn fifo_enable(&mut self) {
        let fcr = FcrFlags::FIFO_ENABLE.bits;
        self.iir_fcr.write(fcr);
    }

    pub fn write(&mut self, s: &str) {
        for byte in s.bytes() {
            self.send(byte);
        }
    }
}
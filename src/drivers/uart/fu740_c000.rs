//! UART driver for HiFive Freedom Unmatched (FU740-C000).

use crate::memory::Register;
use crate::io::{Io, ReadOnly};

// Memory Map
// 0x00 - txdata
// 0x04 - rxdata
// 0x08 - txctrl
// 0x0c - rxctrl
// 0x10 - ie
// 0x14 - ip
// 0x18 - div
#[repr(packed)]
pub struct UART {
    // 0x0000_000f - data (0 on read)
    // 0x8000_0000 - full (ro)
    txdata: Register<u32>,

    // 0x0000_000f - data
    // 0x8000_0000 - empty
    rxdata: ReadOnly<Register<u32>>,

    txctrl: Register<u32>,
    rxctrl: Register<u32>,

    // interrupt enable
    ie: Register<u32>,

    // pending interrupts
    ip: ReadOnly<Register<u32>>,

    // baud rate divisor
    div: Register<u32>,
}

bitflags! {
    // Data flags (for both txdata and rxdata)
    struct DataFlags: u32 {
        const DATA = 0x0000_00FF;
        const FIFO_FULL_OR_EMPTY = 0x8000_0000;
    }
}

bitflags! {
    // TX control flags
    struct TxCtrlFlags: u32 {
        const ENABLED = 0x0000_0001;
        const TWO_STOP_BITS = 0x0000_0002;
        const WATERMARK_LEVEL = 0x0000_0700;
    }
}

bitflags! {
    // RX control flags
    struct RxCtrlFlags: u32 {
        const ENABLED = 0x0000_0001;
        const WATERMARK_LEVEL = 0x0000_0700;
    }
}

bitflags! {
    // Interrupt flags (for both ie and ip)
    struct InterruptFlags: u32 {
        const TX_WATERMARK = 0x0000_0001;
        const RX_WATERMARK = 0x0000_0002;
    }
}

impl UART {
    // The unsafeness here depends on platform and virtual memory layout
    pub unsafe fn new(instance: usize) -> Option<&'static mut UART> {
        // 2 UART instances
        // 0 - 0x1001_0000 - 0x1001_0FFF
        // 1 - 0x1001_1000 - 0x1001_1FFF
        let base: usize = match instance {
            0 => 0x1001_0000,
            1 => 0x1001_1000,
            _ => return None,
        };

        Some(&mut *(base as *mut Self))
    }

    pub fn send(&mut self, data: u8) {
        // Atomic write & OR allows sending with confirmation by
        // simultaneously attempting a send and reading the buffer full flag.
        // Since the data bits always read as zero, the read value is either
        // zero indicating success or non-zero indicating full FIFO.
        let mut retry = 1u32;
        while retry != 0 {
            unsafe {
                asm!(
                    "amoor.w {0}, {1}, ({2})",
                    out(reg) retry,
                    in(reg) data as u32,
                    in(reg) self.txdata.raw_ptr(),
                );
            }
        }
    }

    pub fn receive(&mut self) -> Option<u8> {
        let raw_data = DataFlags::from_bits_truncate(self.txdata.read());
        if raw_data.contains(DataFlags::FIFO_FULL_OR_EMPTY) {
            None
        } else {
            Some(raw_data.bits as u8)
        }
    }

    pub fn write(&mut self, s: &str) {
        for byte in s.bytes() {
            self.send(byte);
        }
    }

    pub fn tx_enable(&mut self, stop_bits: StopBits) {
        let mut flags = TxCtrlFlags::from_bits_truncate(self.txctrl.read());
        flags.insert(TxCtrlFlags::ENABLED);
        flags.set(
            TxCtrlFlags::TWO_STOP_BITS,
            stop_bits == StopBits::TwoStopBits
        );
        self.txctrl.write(flags.bits);
    }

    pub fn rx_enable(&mut self) {
        let mut flags = RxCtrlFlags::from_bits_truncate(self.rxctrl.read());
        flags.insert(RxCtrlFlags::ENABLED);
        self.txctrl.write(flags.bits);
    }

    pub fn set_tx_watermark(
        &mut self,
        watermark: Option<u32>
    ) {
        if let Some(watermark) = watermark {
            let watermark = (watermark << 8) & TxCtrlFlags::WATERMARK_LEVEL.bits;

            let mut flags = TxCtrlFlags::from_bits_truncate(self.txctrl.read());
            flags.remove(TxCtrlFlags::WATERMARK_LEVEL);
            flags.bits |= watermark;
            self.txctrl.write(flags.bits);
        }

        let mut flags = InterruptFlags::from_bits_truncate(self.ie.read());
        flags.set(InterruptFlags::TX_WATERMARK, watermark.is_some());
        self.ie.write(flags.bits);
    }

    pub fn set_rx_watermark(
        &mut self,
        watermark: Option<u32>
    ) {
        if let Some(watermark) = watermark {
            let watermark = (watermark << 8) & RxCtrlFlags::WATERMARK_LEVEL.bits;

            let mut flags = RxCtrlFlags::from_bits_truncate(self.rxctrl.read());
            flags.remove(RxCtrlFlags::WATERMARK_LEVEL);
            flags.bits |= watermark;
            self.rxctrl.write(flags.bits);
        }

        let mut flags = InterruptFlags::from_bits_truncate(self.ie.read());
        flags.set(InterruptFlags::RX_WATERMARK, watermark.is_some());
        self.ie.write(flags.bits);
    }
}

#[derive(PartialEq)]
pub enum StopBits {
    OneStopBit,
    TwoStopBits,
}

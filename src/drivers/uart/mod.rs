mod uart;

pub use uart::{StopBits, Uart};

#[cfg(feature = "fu740")]
pub mod fu740_c000;

#[cfg(feature = "qemu")]
pub mod ns16550a;

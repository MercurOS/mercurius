use core::ops::{BitAnd, BitOr, Not};

use crate::io::Io;

/// Memory mapped control register.
#[repr(packed)]
pub struct Register<T> {
    value: core::mem::MaybeUninit<T>,
}

impl<T> Register<T> {
    pub unsafe fn raw_ptr(&mut self) -> *mut T {
        self.value.as_mut_ptr()
    }
}

impl<T> Io for Register<T>
where
    T: Copy + PartialEq + BitAnd<Output = T> + BitOr<Output = T> + Not<Output = T>
{
    type Value = T;

    fn read(&self) -> T {
        unsafe {
            core::ptr::read_volatile(self.value.as_ptr())
        }
    }

    fn write(&mut self, value: T) {
        unsafe {
            core::ptr::write_volatile(self.value.as_mut_ptr(), value);
        }
    }
}

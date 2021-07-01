use core::cmp::PartialEq;
use core::ops::{BitAnd, BitOr, Not};

pub trait Io {
    type Value:
        Copy +
        PartialEq +
        BitAnd<Output = Self::Value> +
        BitOr<Output = Self::Value> +
        Not<Output = Self::Value>;

    fn read(&self) -> Self::Value;
    fn write(&mut self, value: Self::Value);
}

pub struct ReadOnly<I> {
    inner: I
}

impl<I> ReadOnly<I> {
    pub const fn new(inner: I) -> Self {
        Self { inner }
    }
}

impl<I: Io> ReadOnly<I> {
    #[inline(always)]
    pub fn read(&self) -> I::Value {
        self.inner.read()
    }
}

pub struct WriteOnly<I> {
    inner: I
}

impl<I> WriteOnly<I> {
    pub const fn new(inner: I) -> Self {
        Self { inner }
    }
}

impl<I: Io> WriteOnly<I> {
    #[inline(always)]
    pub fn write(&self, ) -> I::Value {
        self.inner.read()
    }
}

#![allow(dead_code)]

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub struct __user<T>(T)
where
    T: Clone + Copy;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub struct __kernel<T>(T)
where
    T: Clone + Copy;

impl<T> From<T> for __user<T>
where
    T: Clone + Copy,
{
    fn from(value: T) -> Self {
        __user(value)
    }
}

impl<T> From<T> for __kernel<T>
where
    T: Clone + Copy,
{
    fn from(value: T) -> Self {
        __kernel(value)
    }
}

impl<T> __user<T>
where
    T: Clone + Copy,
{
    pub fn new(value: T) -> Self {
        Self(value)
    }
    pub fn inner(&self) -> T {
        self.0
    }
}

impl<T> __kernel<T>
where
    T: Clone + Copy,
{
    pub fn new(value: T) -> Self {
        Self(value)
    }
    pub fn inner(&self) -> T {
        self.0
    }
}
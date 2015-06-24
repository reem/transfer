use std::marker::PhantomData;
use std::{mem, slice, ops};

use iobuf::AROIobuf;
use prelude::*;

pub struct TypedAROIobuf<T> {
    buf: AROIobuf,
    _phantom: PhantomData<Vec<T>>
}

impl<T> TypedAROIobuf<T> {
    pub unsafe fn new(buf: AROIobuf) -> TypedAROIobuf<T> {
        TypedAROIobuf {
            buf: buf,
            _phantom: PhantomData
        }
    }
}

impl<T> AsRef<[T]> for TypedAROIobuf<T> {
    fn as_ref(&self) -> &[T] {
        unsafe {
            let buf = self.buf.as_window_slice();
            slice::from_raw_parts(
                buf.as_ptr() as *const T,
                buf.len() / mem::size_of::<T>())
        }
    }
}

impl<T> ops::Deref for TypedAROIobuf<T> {
    type Target = [T];
    fn deref(&self) -> &[T] { self.as_ref() }
}


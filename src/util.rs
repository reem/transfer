use std::marker::PhantomData;
use std::{mem, slice};

pub struct TypedAROIobuf<T> {
    buf: AROIobuf,
    _phantom: PhantomData<Vec<T>>
}

impl<T> TypedARIObuf<T> {
    pub unsafe fn new(buf: AROIobuf) -> TypedAROIobuf<T> {
        TypedARIobuf {
            buf: buf,
            _phantom: PhantomData
        }
    }
}

impl<T> AsRef<[T]> for TypedARIObuf<T> {
    fn as_ref(&self) -> &[T] {
        unsafe {
            let buf = self.buf.as_window_slice();
            slice::from_raw_buf(
                buf.as_ptr() as *const T,
                buf.len() / mem::size_of::<T>())
        }
    }
}

impl<T> Deref for TypedARIobuf<T> {
    type Target = [T];
    fn deref(&self) -> &[T] { self.as_ref() }
}


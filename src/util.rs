use std::marker::PhantomData;
use std::{mem, slice, ops, fmt};

use appendbuf::Slice;

pub struct TypedSlice<T> {
    buf: Slice,
    _phantom: PhantomData<Vec<T>>
}

impl<T: Copy> Clone for TypedSlice<T> {
    fn clone(&self) -> TypedSlice<T> {
        TypedSlice {
            buf: self.buf.clone(),
            _phantom: PhantomData
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for TypedSlice<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T> TypedSlice<T> {
    pub unsafe fn new(buf: Slice) -> TypedSlice<T> {
        TypedSlice {
            buf: buf,
            _phantom: PhantomData
        }
    }

    pub fn into_slice(self) -> Slice { self.buf }
}

impl<T> AsRef<[T]> for TypedSlice<T> {
    fn as_ref(&self) -> &[T] {
        unsafe {
            let buf = &*self.buf;
            slice::from_raw_parts(
                buf.as_ptr() as *const T,
                buf.len() / mem::size_of::<T>())
        }
    }
}

impl<T> ops::Deref for TypedSlice<T> {
    type Target = [T];
    fn deref(&self) -> &[T] { self.as_ref() }
}


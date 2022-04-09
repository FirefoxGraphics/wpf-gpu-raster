use std::{marker::PhantomData, ops::Deref};

pub struct Ref<'a, T> {
    ptr: *const T,
    _phantom: PhantomData<&'a T>
}

impl<'a, T> Copy for Ref<'a, T> { }

impl<'a, T> Clone for Ref<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Ref<'a, T> {
    pub fn new(p: &'a T) -> Self {
        Ref { ptr: p as *const T, _phantom: PhantomData}
    }
    pub unsafe fn null() -> Self {
        Ref { ptr: std::ptr::null(), _phantom: PhantomData}
    }
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }
}

impl<'a, T> Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}
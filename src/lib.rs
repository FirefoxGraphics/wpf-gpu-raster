use std::ffi::c_void;
extern "C" {
    fn pathbuilder_new() -> *mut c_void;
    fn pathbuilder_delete(ptr: *mut c_void);
}

pub struct PathBuilder {
    ptr: *mut c_void
}

impl PathBuilder {
    pub fn new() -> Self {
        Self { ptr: unsafe { pathbuilder_new() } }
    }
}

impl Drop for PathBuilder {
    fn drop(&mut self) {
        unsafe { pathbuilder_delete(self.ptr); }
    }
}

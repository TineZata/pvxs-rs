#[repr(C)]
#[derive(Debug)]
pub struct std__Ref_count_base {
    pub _Use_count: usize, // Number of shared_ptr instances managing the object
    pub _Weak_count: usize, // Number of weak_ptr instances associated with the object
}

#[repr(C)]
#[derive(Debug)]
pub struct std__Ptr_base {
    pub _Ptr: *mut std::ffi::c_void,   // Pointer to the managed object (type-erased)
    pub _Rep: *mut std__Ref_count_base, // Pointer to the control block
}

#[repr(C)]
#[derive(Debug)]
pub struct std_shared_ptr {
    pub _base: std__Ptr_base,
}

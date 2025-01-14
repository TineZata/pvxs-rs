#[repr(C)]
#[derive(Debug)]
pub struct StdRefCountBase {
    pub _use_count: usize, // Number of shared_ptr instances managing the object
    pub _weak_count: usize, // Number of weak_ptr instances associated with the object
}

#[repr(C)]
#[derive(Debug)]
pub struct StdPtrBase {
    pub _ptr: *mut std::ffi::c_void,   // Pointer to the managed object (type-erased)
    pub _rep: *mut StdRefCountBase, // Pointer to the control block
}

#[repr(C)]
#[derive(Debug)]
pub struct StdSharedPtr {
    pub _base: StdPtrBase,
}


pub type StdAtomicCounterT = ::std::os::raw::c_ulong;



#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct StdRemoveExtent {
    pub _address: u8,
}
pub type StdRemoveExtentT = StdRemoveExtent;

#[repr(C)]
pub struct StdRefCountBaseVtable(::std::os::raw::c_void);

#[repr(C)]
#[derive(Debug)]
pub struct StdRefCountBase {
    pub vtable_: *const StdRefCountBaseVtable,
    pub _uses: StdAtomicCounterT,
    pub _weaks: StdAtomicCounterT,
}

#[repr(C)]
#[derive(Debug)]
pub struct StdWeakPtr {
    pub _base: StdPtrBase,
}

#[repr(C)]
#[derive(Debug)]
pub struct StdPtrBase {
    pub _ptr: *mut StdPtrBaseElementType,
    pub _rep: *mut StdRefCountBase,
}
pub type StdPtrBaseElementType = StdRemoveExtentT;

#[repr(C)]
#[derive(Debug)]
pub struct StdSharedPtr {
    pub _base: StdPtrBase,
}
pub type StdSharedPtrMybase = StdPtrBase;
pub type StdSharedPtrWeakType = StdWeakPtr;

use std::sync::Arc;
use crate::{bin, std_types::StdSharedPtr};

/// Untyped
pub const PVXS_ARRAY_TYPE_NULL: PvxsArrayType = 255;
/// bool
pub const PVXS_ARRAY_TYPE_BOOL: PvxsArrayType = 8;
/// int8_t
pub const PVXS_ARRAY_TYPE_INT8: PvxsArrayType = 40;
/// int16_t
pub const PVXS_ARRAY_TYPE_INT16: PvxsArrayType = 41;
/// int32_t
pub const PVXS_ARRAY_TYPE_INT32: PvxsArrayType = 42;
/// int64_t
pub const PVXS_ARRAY_TYPE_INT64: PvxsArrayType = 43;
/// uint8_t
pub const PVXS_ARRAY_TYPE_UINT8: PvxsArrayType = 44;
/// uint16_t
pub const PVXS_ARRAY_TYPE_UINT16: PvxsArrayType = 45;
/// uint32_t
pub const PVXS_ARRAY_TYPE_UINT32: PvxsArrayType = 46;
/// uint64_t
pub const PVXS_ARRAY_TYPE_UINT64: PvxsArrayType = 47;
/// float
pub const PVXS_ARRAY_TYPE_FLOAT32: PvxsArrayType = 74;
/// double
pub const PVXS_ARRAY_TYPE_FLOAT64: PvxsArrayType = 75;
/// std::string
pub const PVXS_ARRAY_TYPE_STRING: PvxsArrayType = 104;
/// Value
pub const PVXS_ARRAY_TYPE_VALUE: PvxsArrayType = 136;
/// Identify real array type in void specializations of shared_array.\n! @see shared_array::original_type()"]
pub type PvxsArrayType = u8;

/// std::vector-like contiguous array of items passed by reference.
/// 
/// shared_array comes in const and non-const, as well as void and non-void variants.
/// 
/// A non-const array is allocated and filled, then last non-const reference is exchanged for new const reference.
/// This const reference can then be safely shared between various threads.
/// ```cpp
/// shared_array<uint32_t> arr({1, 2, 3});
/// assert(arr.size()==3);
/// shared_ptr<const uint32_t> constarr(arr.freeze());
/// assert(arr.size()==0);
/// assert(constarr.size()==3);
/// ```
/// 
/// The void / non-void variants allow arrays to be moved without explicit typing.
/// However, the void variant preserves the original ArrayType.
/// 
/// ```cpp
/// shared_array<uint32_t> arr({1, 2, 3});
/// assert(arr.size()==3);
/// shared_array<void> voidarr(arr.castTo<void>());
/// assert(arr.size()==0);
/// assert(voidarr.size()==3); // void size() in elements
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct PvxsSharedArray {
    pub _base: PvxsDetailSaBase,
}
pub type PvxsSharedArrayBaseT = PvxsDetailSaBase;

#[repr(C)]
#[derive(Debug)]
pub struct PvxsDetailSaBase {
    pub _data: StdSharedPtr,
    pub _count: usize,
}

/// Return storage size (aka. sizeof() ) for array element type
/// @throws std::logic_error for invalid types.
pub unsafe fn pvxs_element_size(pvxs_library: Arc<bin::LoadLib>, type_: PvxsArrayType) -> usize {
    let func: libloading::Symbol<unsafe extern "C" fn(PvxsArrayType) -> usize> = 
    pvxs_library.lib
    .get(if cfg!(target_os = "windows") {
        b"?elementSize@pvxs@@YA_KW4ArrayType@1@@Z"
    } else if cfg!(target_os = "linux") {
        b"_ZN4pvxs11elementSizeEh"
    } else {
        panic!("Unsupported platform");
    })
    .expect("Failed to find symbol for pvxs::elementSize");
    func(type_)
}

/// Return a void array usable for the given storage type
pub unsafe fn pvxs_alloc_array(pvxs_library: Arc<bin::LoadLib>, type_: PvxsArrayType, count: usize) -> PvxsSharedArray {
    let func: libloading::Symbol<unsafe extern "C" fn(PvxsArrayType, usize) -> PvxsSharedArray> = 
    pvxs_library.lib
    .get(if cfg!(target_os = "windows") {
        b"?allocArray@pvxs@@YA?AV?$shared_array@XX@1@W4ArrayType@1@_K@Z"
    } else if cfg!(target_os = "linux") {
        b"_ZN4pvxs10allocArrayEh"
    } else {
        panic!("Unsupported platform");
    })
    .expect("Failed to find symbol for pvxs::allocArray");
    func(type_, count)
}
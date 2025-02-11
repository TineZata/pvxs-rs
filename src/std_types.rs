use std::marker::{PhantomData, PhantomPinned};
pub type StdAtomicCounterT = ::std::os::raw::c_ulong;

/// If Bindgen could only determine the size and alignment of a
/// type, it is represented like this.
#[derive(PartialEq, Copy, Clone, Debug, Hash)]
#[repr(C)]
pub struct _OpaqueArray<T: Copy, const N: usize>(pub [T; N]);
impl<T: Copy + Default, const N: usize> Default for _OpaqueArray<T, N> {
    fn default() -> Self {
        Self([<T as Default>::default(); N])
    }
}

impl<const N: usize> _OpaqueArray<u64, N> {
    /// Extracts all valid strings from the array.
    pub fn extract_strings(&self) -> Vec<String> {
        self.0
            .iter()
            .filter_map(|&ptr| {
                let ptr = ptr as *const std::ffi::c_char;
                if ptr.is_null() {
                    return None;
                }

                unsafe {
                    std::ffi::CStr::from_ptr(ptr)
                        .to_str()
                        .ok()
                        .map(|s| s.to_owned()) // Convert &str to String
                }
            })
            .collect()
    }

    /// Creates an `_OpaqueArray` from a list of Rust strings.
    /// Returns the array and a vector of `CString` to keep the strings in memory.
    pub fn from_rust_strings(strings: &[&str]) -> Self {
        let c_strings: Vec<std::ffi::CString> = strings
            .iter()
            .map(|&s| std::ffi::CString::new(s).expect("CString conversion failed"))
            .collect();

        let mut ptrs = [0u64; N];
        for (i, cstr) in c_strings.iter().enumerate().take(N) {
            ptrs[i] = cstr.as_ptr() as u64;
        }

        Self(ptrs)
    }
}

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
#[derive(Debug, Copy, Clone)]
pub struct StdPtrBase {
    pub _ptr: *mut StdPtrBaseElementType,
    pub _rep: *mut StdRefCountBase,
}
pub type StdPtrBaseElementType = StdRemoveExtentT;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct StdSharedPtr {
    pub _base: StdPtrBase,
}
pub type StdSharedPtrMybase = StdPtrBase;
pub type StdSharedPtrWeakType = StdWeakPtr;

#[repr(C, align(8))]
#[derive(Debug, Clone)]
pub struct StdBasicString {
    /// Pointer to the start of string data
    pub begin: *const u8,  
    /// Size of the string
    /// 
    /// Note:  
    ///     Don't rely on this value to be the length of the string.
    ///     Always use `to_rust_string` to convert to a Rust `String`.
    pub size: usize,
    /// Capacity of the allocated buffer
    /// 
    /// Note:
    ///     Don't rely on this value to be the capacity of the string.
    ///     Always use `to_rust_string` to convert to a Rust `String`.
    pub capacity: usize,
}

impl StdBasicString {
    /// Converts the `StdString` to a Rust `String`.
    pub unsafe fn to_rust_string_lossy(&self) -> String {
        // Ensure the pointer is not null and the size is valid
        if self.begin.is_null() || self.size == 0 {
            return String::new();
        }

        if self.size > 1_000  {
            // Prevent indexing a huge string... calculate the length manually
            let mut len = 0;
            while *self.begin.add(len) != 0 && len < self.size {
                len += 1;
            }
            // Create a slice from the raw pointer and length
            let slice = std::slice::from_raw_parts(self.begin, len);
            // Convert the slice to a Rust String
            return String::from_utf8_lossy(slice).into_owned()
        }

        // Create a slice from the raw pointer and length
        let slice = std::slice::from_raw_parts(self.begin, self.size);

        // Convert the slice to a String
        String::from_utf8_lossy(slice).into_owned()
    }

    /// Creates an `StdString` from a Rust `String`.
    pub fn from_rust_string(s: String) -> Self {
        let size = s.len();
        let capacity = s.capacity();
        let begin = s.as_ptr();

        // Prevent Rust from freeing the string while `StdString` exists
        std::mem::forget(s);

        Self {
            begin,
            size,
            capacity,
        }
    }

    /// Reclaims ownership of the original Rust `String`.
    ///
    /// This is unsafe because it assumes the `StdString` was created
    /// from a Rust `String` using `from_rust_string`.
    pub unsafe fn into_rust_string(self) -> String {
        // Reclaim ownership of the original Rust String
        String::from_raw_parts(self.begin as *mut u8, self.size, self.capacity) 
    }
}

pub type StdString = _OpaqueArray<u64, 4usize>;
pub type StdWstring = _OpaqueArray<u64, 4usize>;
pub type StdU16string = _OpaqueArray<u64, 4usize>;
pub type StdU32string = _OpaqueArray<u64, 4usize>;

#[repr(C)]
pub struct StdStringVal {
    pub _bx: StdStringValBxty,
    pub _size: StdStringValSizeType,
    pub _res: StdStringValSizeType,
}

pub type StdStringValValueType = __OpaqueArray<u8, 0usize>;
pub type StdStringValSizeType = __OpaqueArray<u8, 0usize>;
pub type StdStringValDifferenceType = __OpaqueArray<u8, 0usize>;
pub type StdStringValPointer = __OpaqueArray<u8, 0usize>;
pub type StdStringValConstPointer = __OpaqueArray<u8, 0usize>;
pub type StdStringValReference = *mut StdStringValValueType;
pub type StdStringValConstReference = *const StdStringValValueType;
#[repr(C)]
pub struct StdStringValBxty {
    pub _buf: __UnionField<*mut StdStringValValueType>,
    pub _ptr: __UnionField<StdStringValPointer>,
    pub _alias: __UnionField<*mut ::std::os::raw::c_char>,
    pub bindgen_union_field: u64,
}

#[repr(C)]
pub union ShortStringOptimisation {
    /// Capacity of the allocated buffer
    /// 
    /// Note:
    ///     Don't rely on this value to be the capacity of the string.
    ///     Always use `to_rust_string` to convert to a Rust `String`.
    pub capacity: usize,
    /// Buffer for short strings
    pub short_buffer: [u8; 16usize],
}

impl Clone for ShortStringOptimisation {
    fn clone(&self) -> Self {
        unsafe { std::ptr::read(self) }
    }
}

impl core::fmt::Debug for ShortStringOptimisation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            write!(f, "ShortStringOptimisation {{ capacity: {}, short_buffer: {:?} }}", self.capacity, &self.short_buffer)
        }
    }
}

#[repr(C, align(8))]
#[derive(Debug, Clone)]
pub struct StdString32 {
    pub begin: *const u8,
    pub size: usize,
    pub cap_or_sso: ShortStringOptimisation,
}

impl StdString32 {
    pub fn new() -> Self {
        Self {
            begin: std::ptr::null(),
            size: 0,
            cap_or_sso: ShortStringOptimisation { capacity: 0 },
        }
    }
    pub unsafe fn to_rust_string(&self) -> String {
        // Ensure the pointer is not null and the size is valid
        if self.begin.is_null() || self.size == 0 {
            return String::new();
        }

        if self.size > 1_000  {
            // Prevent indexing a huge string... calculate the length manually
            let mut len = 0;
            while *self.begin.add(len) != 0 && len < self.size {
                len += 1;
            }
            // Create a slice from the raw pointer and length
            let slice = std::slice::from_raw_parts(self.begin, len);
            // Convert the slice to a Rust String
            return String::from_utf8_lossy(slice).into_owned()
        }

        // Create a slice from the raw pointer and length
        let slice = std::slice::from_raw_parts(self.begin, self.size);

        // Convert the slice to a String
        String::from_utf8_lossy(slice).into_owned()
    }

    pub fn from_rust_string(s: String) -> Self {
        let size = s.len();
        let capacity = s.capacity();
        let begin = s.as_ptr();

        // Prevent Rust from freeing the string while `StdString` exists
        std::mem::forget(s);

        Self {
            begin,
            size,
            cap_or_sso: ShortStringOptimisation { capacity },
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug, Hash)]
#[repr(C)]
pub struct __OpaqueArray<T: Copy, const N: usize>(pub [T; N]);
impl<T: Copy + Default, const N: usize> Default for __OpaqueArray<T, N> {
    fn default() -> Self {
        Self([<T as Default>::default(); N])
    }
}

#[repr(C)]
pub struct __UnionField<T>(::std::marker::PhantomData<T>);
impl<T> __UnionField<T> {
    #[inline]
    pub const fn new() -> Self {
        __UnionField(::std::marker::PhantomData)
    }
    #[inline]
    pub unsafe fn as_ref(&self) -> &T {
        ::std::mem::transmute(self)
    }
    #[inline]
    pub unsafe fn as_mut(&mut self) -> &mut T {
        ::std::mem::transmute(self)
    }
}
impl<T> ::std::default::Default for __UnionField<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
impl<T> ::std::clone::Clone for __UnionField<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> ::std::marker::Copy for __UnionField<T> {}
impl<T> ::std::fmt::Debug for __UnionField<T> {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        fmt.write_str("__UnionField")
    }
}
impl<T> ::std::hash::Hash for __UnionField<T> {
    fn hash<H: ::std::hash::Hasher>(&self, _state: &mut H) {}
}
impl<T> ::std::cmp::PartialEq for __UnionField<T> {
    fn eq(&self, _other: &__UnionField<T>) -> bool {
        true
    }
}
impl<T> ::std::cmp::Eq for __UnionField<T> {}

#[repr(C)]
pub struct StdVectorVal {
    pub _first: StdVectorValPointer,
    pub _last: StdVectorValPointer,
    pub _end: StdVectorValPointer,
}
pub type StdVectorValValueType = __OpaqueArray<u8, 0usize>;
pub type StdVectorValSizeType = __OpaqueArray<u8, 0usize>;
pub type StdVectorValDifferenceType = __OpaqueArray<u8, 0usize>;
pub type StdVectorValPointer = __OpaqueArray<u8, 0usize>;
pub type StdVectorValConstPointer = __OpaqueArray<u8, 0usize>;
pub type StdVectorValReference = *mut StdVectorValValueType;
pub type StdVectorValConstReference = *const StdVectorValValueType;
#[repr(C, align(8))]
pub struct StdBasicVector<T> {
    // A thing, because repr(C) structs are not allowed to consist exclusively
    // of PhantomData fields.
    pub _void:  [u64; 3usize],
    // The conceptual vector elements to ensure that autotraits are propagated
    // correctly, e.g. CxxVector is UnwindSafe iff T is.
    _elements: PhantomData<[T]>,
    // Prevent unpin operation from Pin<&mut CxxVector<T>> to &mut CxxVector<T>.
    _pinned: PhantomData<PhantomPinned>,
}
pub type StdVector<T> = StdBasicVector<T>;

#[repr(C)]
#[derive(Debug)]
pub struct StdMap {
    pub _base: StdTree,
}

#[repr(C)]
#[derive(Debug)]
pub struct StdTree {
    pub _pair: u8,
}

#[repr(C)]
#[derive(Debug)]
pub struct StdRuntimeError {
    pub _base: StdException,
}

pub type StdRuntimeErrorBase = StdException;

#[repr(C)]
pub struct StdExceptionVtable(::std::os::raw::c_void);
#[repr(C)]
#[derive(Debug)]
pub struct StdException {
    pub vtable_: *const StdExceptionVtable,
    pub _data: StdExceptionData,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of std_exception"][::std::mem::size_of::<StdException>() - 24usize];
    ["Alignment of std_exception"][::std::mem::align_of::<StdException>() - 8usize];
    ["Offset of field: std_exception::_Data"][::std::mem::offset_of!(StdException, _data) - 8usize];
};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct StdExceptionData {
    pub _what: *const ::std::os::raw::c_char,
    pub _do_free: bool,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of __std_exception_data"][::std::mem::size_of::<StdExceptionData>() - 16usize];
    ["Alignment of __std_exception_data"][::std::mem::align_of::<StdExceptionData>() - 8usize];
    ["Offset of field: __std_exception_data::_What"]
        [::std::mem::offset_of!(StdExceptionData, _what) - 0usize];
    ["Offset of field: __std_exception_data::_DoFree"]
        [::std::mem::offset_of!(StdExceptionData, _do_free) - 8usize];
};

#[repr(C)]
#[derive(Debug)]
pub struct StdLogicError {
    pub _base: StdException,
}
pub type StdLogicErrorBase = StdException;
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of std_logic_error"][::std::mem::size_of::<StdLogicError>() - 24usize];
    ["Alignment of std_logic_error"][::std::mem::align_of::<StdLogicError>() - 8usize];
};

#[repr(C, align(8))]
#[derive(Debug, Copy, Clone)]
pub struct StdFunction64 {
    pub _address: [u8; 64usize],
}

impl StdFunction64 {
    pub fn new() -> Self {
        Self {
            _address: [0; 64usize],
        }
    }
}

pub type StdFunction = _OpaqueArray<u64, 8usize>;

impl StdFunction {
    pub fn new() -> Self {
        Self([0; 8usize])
    }
}



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

#[repr(C)]
#[derive(Debug)]
pub struct StdBasicString {
    pub _pair: u8,
}
pub type StdString = [u64; 4usize];
pub type StdWstring = StdBasicString;
pub type StdU16string = StdBasicString;
pub type StdU32string = StdBasicString;

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

use std::sync::atomic::AtomicUsize;
use std::collections::BTreeMap;

pub type StdBoolConstant = u8;
pub type StdEnableIfT = u8;
pub type StdBasicStringAlty = StdAllocatorTraits;
pub type StdBasicStringAltyTraits = StdAllocatorTraits;
pub type StdBasicStringScaryVal = StdStringVal;
pub type StdBasicStringTraitsType<_Traits> = _Traits;
pub type StdBasicStringAllocatorType<_Alloc> = _Alloc;
pub type StdBasicStringValueType<_Elem> = _Elem;
pub type StdBasicStringSizeType = StdBasicStringAltyTraits;
pub type StdBasicStringDifferenceType = StdBasicStringAltyTraits;
pub type StdBasicStringPointer = StdBasicStringAltyTraits;
pub type StdBasicStringConstPointer = StdBasicStringAltyTraits;
pub type StdBasicStringReference<_Elem> = *mut StdBasicStringValueType<_Elem>;
pub type StdBasicStringConstReference<_Elem> = *const StdBasicStringValueType<_Elem>;
pub type StdBasicStringIterator = __OpaqueArray<u8, 0usize>;
pub type StdBasicStringConstIterator = __OpaqueArray<u8, 0usize>;
pub type StdBasicStringReverseIterator = StdReverseIterator<StdBasicStringIterator>;
pub type StdBasicStringConstReverseIterator = StdReverseIterator<StdBasicStringConstIterator>;
pub type StdBasicStringIsElemCptr = StdBoolConstant;
pub type StdBasicStringIsStringViewIsh = StdEnableIfT;
pub const STD_BASIC_STRING_ALLOCATION_POLICY_EXACTLY: StdBasicStringAllocationPolicy = 0;
pub type StdBasicStringAllocationPolicy = ::std::os::raw::c_int;
pub const STD_BASIC_STRING_CONSTRUCT_STRATEGY_FROM_CHAR: StdBasicStringConstructStrategy = 0;
pub const STD_BASIC_STRING_CONSTRUCT_STRATEGY_FROM_PTR: StdBasicStringConstructStrategy = 0;
pub const STD_BASIC_STRING_CONSTRUCT_STRATEGY_FROM_STRING: StdBasicStringConstructStrategy = 0;
pub type StdBasicStringConstructStrategy = ::std::os::raw::c_uchar;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct StdString {
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

impl StdString {
    /// Converts the `StdString` to a Rust `String`.
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

#[repr(C)]
#[derive(Debug, Clone)]
pub struct StdWstring {
    pub begin: *const u16, // Pointer to the character data
    pub size: usize,      // Size of the string
    pub capacity: usize,  // Capacity of the allocated buffer
}
#[repr(C)]
#[derive(Debug, Clone)]
pub struct StdU16string {
    pub begin: *const u16, // Pointer to the character data
    pub size: usize,      // Size of the string
    pub capacity: usize,  // Capacity of the allocated buffer
}
#[repr(C)]
#[derive(Debug, Clone)]
pub struct  StdU32string {
    pub begin: *const u32, // Pointer to the character data
    pub size: usize,      // Size of the string
    pub capacity: usize,  // Capacity of the allocated buffer
}

/// If Bindgen could only determine the size and alignment of a
/// type, it is represented like this.
#[derive(PartialEq, Copy, Clone, Debug, Hash)]
#[repr(C)]
pub struct __OpaqueArray<T: Copy, const N: usize>(pub [T; N]);
impl<T: Copy + Default, const N: usize> Default for __OpaqueArray<T, N> {
    fn default() -> Self {
        Self([<T as Default>::default(); N])
    }
}

pub type StdStringValueArray = __OpaqueArray<u8, 0usize>;
pub type StdStringValuePointer = __OpaqueArray<u8, 0usize>;
pub type StdStringSizeType = __OpaqueArray<u8, 0usize>;

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

#[repr(C)]
pub struct StdStringValBxty {
    pub _buf: __UnionField<*mut StdStringValueArray>,
    pub _ptr: __UnionField<StdStringValuePointer>,
    pub _alias: __UnionField<*mut ::std::os::raw::c_char>,
    pub union_field: u64,
}

#[repr(C)]
pub struct StdStringVal {
    pub _bx: StdStringValBxty,
    pub _mysize: StdStringSizeType,
    pub _myres: StdStringSizeType,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct StdAllocatorTraits {
    pub _address: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct StdReverseIterator<_BidIt> {
    pub _phantom_0: ::std::marker::PhantomData<::std::cell::UnsafeCell<_BidIt>>,
    pub current: _BidIt,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct StdFunction {
    pub _address: u8,
}

#[repr(C)]
///Holds atomic counters for reference and weak counts, similar to C++'s `std::shared_ptr`."]
#[derive(Debug)]
pub struct StdRefCountBase {
    pub _use_count: AtomicUsize, // Number of shared_ptr instances managing the object
    pub _weak_count: AtomicUsize, // Number of weak_ptr instances associated with the object
}

#[repr(C)]
/// Explicitly contains two pointers: one to the managed object (_ptr) and another to the control block (_rep).
/// This ensures the StdSharedPtr size matches the C++ `std::shared_ptr` (16 bytes on 64-bit systems)
#[derive(Debug, Clone)]
pub struct StdPtrBaseVoid {
    ///Pointer to the managed object of type T"]
    pub _ptr: *mut std::ffi::c_void,
    ///Pointer to the control block"]
    pub _rep: *mut StdRefCountBase,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct StdSharedPtrBase<T> {
    pub _ptr: T,
    pub _rep: *mut StdRefCountBase,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct StdSharedPtr<T> {
    pub _base: StdSharedPtrBase<T>,
}

///!< no associate storage"]
pub const STORE_TYPE_NULL: StoreType = 0;
///!< bool"]
pub const STORE_TYPE_BOOL: StoreType = 1;
///!< uint64_t"]
pub const STORE_TYPE_UINTEGER: StoreType = 2;
///!< int64_t"]
pub const STORE_TYPE_INTEGER: StoreType = 3;
///!< double"]
pub const STORE_TYPE_REAL: StoreType = 4;
///!< std::string"]
pub const STORE_TYPE_STRING: StoreType = 5;
///!< Value"]
pub const STORE_TYPE_COMPOUND: StoreType = 6;
///!< shared_array<const void>"]
pub const STORE_TYPE_ARRAY: StoreType = 7;
///! selector for union FieldStorage::store"]
pub type StoreType = u8;

///!< Untyped"]
pub const ARRAY_TYPE_NULL: ArrayType = 255;
///!< bool"]
pub const ARRAY_TYPE_BOOL: ArrayType = 8;
///!< int8_t"]
pub const ARRAY_TYPE_INT8: ArrayType = 40;
///!< int16_t"]
pub const ARRAY_TYPE_INT16: ArrayType = 41;
///!< int32_t"]
pub const ARRAY_TYPE_INT32: ArrayType = 42;
///!< int64_t"]
pub const ARRAY_TYPE_INT64: ArrayType = 43;
///!< uint8_t"]
pub const ARRAY_TYPE_UINT8: ArrayType = 44;
///!< uint16_t"]
pub const ARRAY_TYPE_UINT16: ArrayType = 45;
///!< uint32_t"]
pub const ARRAY_TYPE_UINT32: ArrayType = 46;
///!< uint64_t"]
pub const ARRAY_TYPE_UINT64: ArrayType = 47;
///!< float"]
pub const ARRAY_TYPE_FLOAT32: ArrayType = 74;
///!< double"]
pub const ARRAY_TYPE_FLOAT64: ArrayType = 75;
///!< std::string"]
pub const ARRAY_TYPE_STRING: ArrayType = 104;
/// < Value
pub const ARRAY_TYPE_VALUE: ArrayType = 136;
/// Identify real array type in void specializations of shared_array.\n! @see shared_array::original_type()"
pub type ArrayType = u8;

/// Possible Field types.
/// eg. String is scalar string, StringA is array of strings.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TypeCode {
    /// ! the actual type code.  eg. for switch()
    pub code: CodeType,
}
pub const CODE_T_BOOL: CodeType = 0;
pub const CODE_T_BOOL_A: CodeType = 8;
pub const CODE_T_INT8: CodeType = 32;
pub const CODE_T_INT16: CodeType = 33;
pub const CODE_T_INT32: CodeType = 34;
pub const CODE_T_INT64: CodeType = 35;
pub const CODE_T_UINT8: CodeType = 36;
pub const CODE_T_UINT16: CodeType = 37;
pub const CODE_T_UINT32: CodeType = 38;
pub const CODE_T_UINT64: CodeType = 39;
pub const CODE_T_INT8_A: CodeType = 40;
pub const CODE_T_INT16_A: CodeType = 41;
pub const CODE_T_INT32_A: CodeType = 42;
pub const CODE_T_INT64_A: CodeType = 43;
pub const CODE_T_UINT8_A: CodeType = 44;
pub const CODE_T_UINT16_A: CodeType = 45;
pub const CODE_T_UINT32_A: CodeType = 46;
pub const CODE_T_UINT64_A: CodeType = 47;
pub const CODE_T_FLOAT32: CodeType = 66;
pub const CODE_T_FLOAT64: CodeType = 67;
pub const CODE_T_FLOAT32_A: CodeType = 74;
pub const CODE_T_FLOAT64_A: CodeType = 75;
pub const CODE_T_STRING: CodeType = 96;
pub const CODE_T_STRING_A: CodeType = 104;
pub const CODE_T_STRUCT: CodeType = 128;
pub const CODE_T_UNION: CodeType = 129;
pub const CODE_T_ANY: CodeType = 130;
pub const CODE_T_STRUCT_A: CodeType = 136;
pub const CODE_T_UNION_A: CodeType = 137;
pub const CODE_T_ANY_A: CodeType = 138;
pub const CODE_T_NULL: CodeType = 255;
///! actual complete (scalar) type code."]
pub type CodeType = u8;
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_TypeCode"][::std::mem::size_of::<TypeCode>() - 1usize];
    ["Alignment of pvxs_TypeCode"][::std::mem::align_of::<TypeCode>() - 1usize];
    ["Offset of field: pvxs_TypeCode::code"][::std::mem::offset_of!(TypeCode, code) - 0usize];
};
unsafe extern "C" {
    #[link_name = "\u{1}?valid@TypeCode@pvxs@@QEBA_NXZ"]
    pub fn pvxs_TypeCode_valid(this: *const TypeCode) -> bool;
}
unsafe extern "C" {
    #[link_name = "\u{1}?storedAs@TypeCode@pvxs@@QEBA?AW4StoreType@2@XZ"]
    pub fn pvxs_TypeCode_storedAs(this: *const TypeCode) -> StoreType;
}
unsafe extern "C" {
    #[link_name = "\u{1}?arrayType@TypeCode@pvxs@@QEBA?AW4ArrayType@2@XZ"]
    pub fn pvxs_TypeCode_arrayType(this: *const TypeCode) -> ArrayType;
}
unsafe extern "C" {
    ///! name string.  eg. \"bool\" or \"uint8_t\""]
    #[link_name = "\u{1}?name@TypeCode@pvxs@@QEBAPEBDXZ"]
    pub fn pvxs_TypeCode_name(this: *const TypeCode) -> *const ::std::os::raw::c_char;
}
impl TypeCode {
    #[inline]
    pub unsafe fn valid(&self) -> bool {
        pvxs_TypeCode_valid(self)
    }
    #[inline]
    pub unsafe fn stored_as(&self) -> StoreType {
        pvxs_TypeCode_storedAs(self)
    }
    #[inline]
    pub unsafe fn array_type(&self) -> ArrayType {
        pvxs_TypeCode_arrayType(self)
    }
    #[inline]
    pub unsafe fn name(&self) -> *const ::std::os::raw::c_char {
        pvxs_TypeCode_name(self)
    }
}

/// Definition of a member of a Struct/Union for use with TypeDef
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Member {
    pub code: TypeCode,
    pub name: StdString,
    pub id: StdString,
    pub children: Vec<Member>,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MemberHelper {
    _unused: [u8; 0],
}

/// Define a new type, either from scratch, or based on an existing Value
/// 
/// ```cpp
/// namespace M = pvxs::members;
/// auto def1 = TypeDef(TypeCode::Int32); // a single scalar field
/// auto def2 = TypeDef(TypeCode::Struct, {
///     M::Int32("value"),
///     M::Struct("alarm", "alarm_t", {
///         M::Int32("severity"),
///     }),
///     def1.as("special"), // compose definitions
///     });
///     auto val = def2.create(); // instantiate a Value
/// });
/// ```
/// 
#[repr(C)]
#[derive(Debug)]
pub struct TypeDef {
    pub top: StdSharedPtr<*mut Member>,
    pub desc: StdSharedPtr<*mut FieldDesc>,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TypeDefNode {
    _unused: [u8; 0],
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_TypeDef"][::std::mem::size_of::<TypeDef>() - 32usize];
    ["Alignment of pvxs_TypeDef"][::std::mem::align_of::<TypeDef>() - 8usize];
    ["Offset of field: pvxs_TypeDef::top"][::std::mem::offset_of!(TypeDef, top) - 0usize];
    ["Offset of field: pvxs_TypeDef::desc"][::std::mem::offset_of!(TypeDef, desc) - 16usize];
};

/// Describes a single field, leaf or otherwise, in a nested structure.
///
///    FieldDesc are always stored depth first as a contiguous array,
///    with offset to descendant fields given as positive integers relative
///    to the current field. (not possible to jump _back_)
///
///    We deal with indices in this FieldDesc array, found in `FieldDesc::mlookup`
///    and `FieldDesc::miter` relative to the current position in the FieldDesc array.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct FieldDesc {
    ///Type ID string (Struct/Union)"]
    pub id: StdString,

    /// Lookup of all descendant fields of this Structure or Union.
    /// `fld.sub.leaf -> rel index`\r
    /// - For Struct, relative to this (always >=1)\r
    /// - For Union, offset in members array (one entry will always be zero)"]
    pub mlookup: BTreeMap<StdString, usize>,

    /// Child iteration. `child# -> (sub, rel index in enclosing vector<FieldDesc>)`
    pub miter: Vec<(StdString, usize)>,

    /// Number of FieldDesc nodes between this node and its parent Struct (or 0 if no parent).
    /// This value also appears in the parent's miter and mlookup mappings.
    /// Only usable when a StructTop is accessible and `this != StructTop::desc`.
    pub parent_index: usize,

    /// For Union, UnionA, StructA:
    /// - For Union, the choices concatenated together (members.size() != #choices)\r
    /// - For UnionA/StructA containing a single Union/Struct"]
    pub members: Vec<FieldDesc>,

    ///The type code of this FieldDesc."]
    pub code: TypeCode,
}

/// Generic data container
///         
/// References a single data field, which may be free-standing (eg. `int x = 5;`)
/// or a member of an enclosing Struct, or an element in an array of Struct.
///         
/// - Use valid() (or operator bool() ) to determine if pointed to a valid field.
/// - Use operator[] to traverse within a Kind::Compound field.
///         
/// ```cpp
/// Value val = nt::NTScalar{TypeCode::Int32}.create();
/// val["value"] = 42;
/// Value alias = val;
/// assert(alias["value"].as<int32_t>()==42); // 'alias' is a second reference to the same Struct
/// ```
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Value {
    store: StoreType,
    desc: FieldDesc,
}

/// Equivalent of `CommonBase::Req` in C++
#[repr(C)]
#[derive(Debug)]
pub struct Req {
    pv_request: Value,
    fields: Member,
    options: BTreeMap<StdString, Value>,
}

#[repr(C)]
#[derive(Debug)]
pub struct CommonBase {
    pub ctx: StdSharedPtr<*mut std::ffi::c_void>,
    pub _name: StdString,
    pub _server: StdString,
    pub req: StdSharedPtr<*mut Req>,
    pub _prio: std::ffi::c_uint,
    pub _autoexec: bool,
    pub _sync_cancel: bool,
}

#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_client_detail_CommonBase"]
    [::std::mem::size_of::<CommonBase>() - 88usize];
    ["Alignment of pvxs_client_detail_CommonBase"]
    [::std::mem::align_of::<CommonBase>() - 8usize];
    ["Offset of field: pvxs_client_detail_CommonBase::ctx"]
    [::std::mem::offset_of!(CommonBase, ctx) - 0usize];
    ["Offset of field: pvxs_client_detail_CommonBase::_name"]
    [::std::mem::offset_of!(CommonBase, _name) - 16usize];
    ["Offset of field: pvxs_client_detail_CommonBase::_server"]
    [::std::mem::offset_of!(CommonBase, _server) - 40usize];
    ["Offset of field: pvxs_client_detail_CommonBase::req"]
    [::std::mem::offset_of!(CommonBase, req) - 64usize];
    ["Offset of field: pvxs_client_detail_CommonBase::_prio"]
    [::std::mem::offset_of!(CommonBase, _prio) - 80usize];
    ["Offset of field: pvxs_client_detail_CommonBase::_autoexec"]
    [::std::mem::offset_of!(CommonBase, _autoexec) - 84usize];
    ["Offset of field: pvxs_client_detail_CommonBase::_syncCancel"]
    [::std::mem::offset_of!(CommonBase, _sync_cancel) - 85usize];
};

///! Options common to all operations
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CommonBuilder<Base> {
    pub _phantom_0: ::std::marker::PhantomData<::std::cell::UnsafeCell<Base>>,
    pub _base: Base,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CommonBaseReq {
    pub pv_request: Value,
    pub fields: Member,
    pub options: BTreeMap<StdString, Value>,
}

/// Prepare a remote GET or GET_FIELD (info) operation.
/// 
/// See Context::get()
#[repr(C)]
#[derive(Debug)]
pub struct GetBuilder {
    pub _base: CommonBuilder<CommonBase>,
    pub _on_init: StdFunction,
    pub _result: StdFunction,
    pub _get: bool,
}

#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_client_GetBuilder"][::std::mem::size_of::<GetBuilder>() - 96usize];
    ["Alignment of pvxs_client_GetBuilder"]
        [::std::mem::align_of::<GetBuilder>() - 8usize];
    ["Offset of field: pvxs_client_GetBuilder::_onInit"]
        [::std::mem::offset_of!(GetBuilder, _on_init) - 88usize];
    ["Offset of field: pvxs_client_GetBuilder::_result"]
        [::std::mem::offset_of!(GetBuilder, _result) - 89usize];
    ["Offset of field: pvxs_client_GetBuilder::_get"]
        [::std::mem::offset_of!(GetBuilder, _get) - 90usize];
};


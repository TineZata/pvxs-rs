use std:: sync::Arc;
use libloading::Symbol;

use crate::{bin::LoadLib, std_types::{StdSharedPtr, StdString}, storetype::StoreType, typecode::TypeCode};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FieldDesc {
    _unused: [u8; 0],
}

/// Generic data container
/// 
///  References a single data field, which may be free-standing (eg. \"int x = 5;\")
/// or a member of an enclosing Struct, or an element in an array of Struct.
/// 
/// - Use valid() (or operator bool() ) to determine if pointed to a valid field.
/// - Use operator[] to traverse within a Kind::Compound field.
/// 
/// ```cpp 
/// Value val = nt::NTScalar{TypeCode::Int32}.create();
/// val[\"value\"] = 42;
/// Value alias = val;
/// assert(alias[\"value\"].as<int32_t>()==42); // 'alias' is a second reference to the same Struct
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct Value {
    pub store: StdSharedPtr,
    pub desc: *const FieldDesc,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ValueHelper {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ValueIAll {
    pub _address: u8,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_Value__IAll"][::std::mem::size_of::<ValueIAll>() - 1usize];
    ["Alignment of pvxs_Value__IAll"][::std::mem::align_of::<ValueIAll>() - 1usize];
};
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ValueIChildren {
    pub _address: u8,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_Value__IChildren"][::std::mem::size_of::<ValueIChildren>() - 1usize];
    ["Alignment of pvxs_Value__IChildren"]
        [::std::mem::align_of::<ValueIChildren>() - 1usize];
};
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ValueIMarked {
    pub nextcheck: usize,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_Value__IMarked"][::std::mem::size_of::<ValueIMarked>() - 8usize];
    ["Alignment of pvxs_Value__IMarked"][::std::mem::align_of::<ValueIMarked>() - 8usize];
    ["Offset of field: pvxs_Value__IMarked::nextcheck"]
        [::std::mem::offset_of!(ValueIMarked, nextcheck) - 0usize];
};
pub type ValueIAllType = ValueIterable;
pub type ValueIChildrenType = ValueIterable;
pub type ValueIMarkedType = ValueIterable;
#[doc = "! Provides options to control printing of a Value via std::ostream."]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ValueFmt {
    pub top: *const Value,
    pub _limit: usize,
    pub _format: ValueFmtFormatT,
    pub _show_value: bool,
}
pub const VALUE_FMT_FORMAT_T_TREE: ValueFmtFormatT = 0;
pub const VALUE_FMT_FORMAT_T_DELTA: ValueFmtFormatT = 1;
pub type ValueFmtFormatT = ::std::os::raw::c_int;
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_Value_Fmt"][::std::mem::size_of::<ValueFmt>() - 24usize];
    ["Alignment of pvxs_Value_Fmt"][::std::mem::align_of::<ValueFmt>() - 8usize];
    ["Offset of field: pvxs_Value_Fmt::top"][::std::mem::offset_of!(ValueFmt, top) - 0usize];
    ["Offset of field: pvxs_Value_Fmt::_limit"]
        [::std::mem::offset_of!(ValueFmt, _limit) - 8usize];
    ["Offset of field: pvxs_Value_Fmt::_format"]
        [::std::mem::offset_of!(ValueFmt, _format) - 16usize];
    ["Offset of field: pvxs_Value_Fmt::_showValue"]
        [::std::mem::offset_of!(ValueFmt, _show_value) - 20usize];
};
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_Value"][::std::mem::size_of::<Value>() - 24usize];
    ["Alignment of pvxs_Value"][::std::mem::align_of::<Value>() - 8usize];
    ["Offset of field: pvxs_Value::store"][::std::mem::offset_of!(Value, store) - 0usize];
    ["Offset of field: pvxs_Value::desc"][::std::mem::offset_of!(Value, desc) - 16usize];
};

/// allocate new storage, with default values
pub unsafe fn pvxs_value_clone_empty(this: *const Value, pvxs_library: Arc<LoadLib>) -> Value 
{
    // Load the symbol for `cloneEmpty`
    let func: Symbol<unsafe extern "C" fn(*const Value) -> Value> = 
        pvxs_library.lib
        .get(if cfg!(target_os = "windows") {
            b"??cloneEmpty@Value@pvxs@@QEBA?AV12@XZ"
        } else if cfg!(target_os = "linux") {
            b"_ZN4pvxs5Value9cloneEmptyEv"
        } else {
            b""
        })
        .expect("Function `cloneEmpty` not found");
    func(this)
}

/// Allocate new storage and copy in our values
pub unsafe fn pvxs_value_clone(this: *const Value, pvxs_library: Arc<LoadLib>) -> Value {
    // Load the symbol for `clone`
    let func: Symbol<unsafe extern "C" fn(*const Value) -> Value> = 
        pvxs_library.lib
        .get(if cfg!(target_os = "windows") {
            b"?clone@Value@pvxs@@QEBA?AV12@XZ"
        } else if cfg!(target_os = "linux") {
            b"_ZN4pvxs5Value5cloneEv"
        } else {
            b""
        })
        .expect("Function `clone` not found");
    func(this)
}

/// Copy value(s) from other.
/// Acts like from(o) for kind==Kind::Compound .
/// Acts like from(o.as<T>()) for kind!=Kind::Compound
pub unsafe fn pvxs_value_assign(this: *mut Value, o: *const Value, pvxs_library: Arc<LoadLib>) -> *mut Value {
    // Load the symbol for `assign`
    let func: Symbol<unsafe extern "C" fn(*mut Value, *const Value) -> *mut Value> = 
        pvxs_library.lib
        .get(if cfg!(target_os = "windows") {
            b"??assign@Value@pvxs@@QEAAAEAV12@AEBV12@@Z"
        } else if cfg!(target_os = "linux") {
            b"_ZN4pvxs5Value6assignERKS0_"
        } else {
            b""
        })
        .expect("Function `assign` not found");
    func(this, o)
}

/// Use to allocate members for an array of Struct and array of Union
pub unsafe fn pvxs_value_alloc_member(this: *mut Value, pvxs_library: Arc<LoadLib>) -> Value {
    // Load the symbol for `allocMember`
    let func: Symbol<unsafe extern "C" fn(*mut Value) -> Value> = 
        pvxs_library.lib
        .get(if cfg!(target_os = "windows") {
            b"?allocMember@Value@pvxs@@QEAA?AV12@XZ"
        } else if cfg!(target_os = "linux") {
            b"_ZN4pvxs5Value11allocMemberEv"
        } else {
            b""
        })
        .expect("Function `allocMember` not found");
    func(this)
}

/// Restore to newly allocated state.
/// 
/// Free any allocation for array or string values, zero numeric values.
/// unmark() all fields.
/// 
/// @since 1.1.0
pub unsafe fn pvxs_value_clear(this: *mut Value, pvxs_library: Arc<LoadLib>) {
    // Load the symbol for `clear`
    let func: Symbol<unsafe extern "C" fn(*mut Value) -> ()> = 
        pvxs_library.lib
        .get(if cfg!(target_os = "windows") {
            b"?clear@Value@pvxs@@QEAAXXZ"
        } else if cfg!(target_os = "linux") {
            b"_ZN4pvxs5Value5clearEv"
        } else {
            b""
        })
        .expect("Function `clear` not found");
    func(this)
}

/// Test if this field is marked as valid/changed
pub unsafe fn pvxs_value_is_marked(this: *const Value, parents: bool, children: bool, pvxs_library: Arc<LoadLib>) -> bool {
    // Load the symbol for `isMarked`
    let func: Symbol<unsafe extern "C" fn(*const Value, bool, bool) -> bool> = 
        pvxs_library.lib
        .get(if cfg!(target_os = "windows") {
            b"?isMarked@Value@pvxs@@QEBA_N_N0@Z"
        } else if cfg!(target_os = "linux") {
            b"_ZNK4pvxs5Value8isMarkedEbb"
        } else {
            b""
        })
        .expect("Function `isMarked` not found");
    func(this, parents, children)
}

/// return *this if isMarked()==true, or a !valid() ref. if false.
pub unsafe fn pvxs_value_if_marked(this: *const Value, parents: bool, children: bool, pvxs_library: Arc<LoadLib>) -> Value {
    // Load the symbol for `ifMarked`
    let func: Symbol<unsafe extern "C" fn(*const Value, bool, bool) -> Value> = 
        pvxs_library.lib
        .get(if cfg!(target_os = "windows") {
            b"?ifMarked@Value@pvxs@@QEBA?AV12@_N0@Z"
        } else if cfg!(target_os = "linux") {
            b"_ZNK4pvxs5Value8ifMarkedEbb"
        } else {
            b""
        })
        .expect("Function `ifMarked` not found");
    func(this, parents, children)
}

/// Mark this field as valid/changed
pub unsafe fn pvxs_value_mark(this: *mut Value, v: bool, pvxs_library: Arc<LoadLib>) {
    // Load the symbol for `mark`
    let func: Symbol<unsafe extern "C" fn(*mut Value, bool) -> ()> = 
        pvxs_library.lib
        .get(if cfg!(target_os = "windows") {
            b"?mark@Value@pvxs@@QEAAX_N@Z"
        } else if cfg!(target_os = "linux") {
            b"_ZN4pvxs5Value4markEb"
        } else {
            b""
        })
        .expect("Function `mark` not found");
    func(this, v)
}

unsafe extern "C" {
    #[doc = "! Remove mark from this field, and optionally parent and/or child fields.\n! \\since 1.1.3 Correctly unmark parent fields"]
    #[link_name = "\u{1}?unmark@Value@pvxs@@QEAAX_N0@Z"]
    pub fn pvxs_Value_unmark(this: *mut Value, parents: bool, children: bool);
}
unsafe extern "C" {
    #[doc = "! Type of the referenced field (or Null)"]
    #[link_name = "\u{1}?type@Value@pvxs@@QEBA?AUTypeCode@2@XZ"]
    pub fn pvxs_Value_type(this: *const Value) -> TypeCode;
}
unsafe extern "C" {
    #[doc = "! Type of value stored in referenced field"]
    #[link_name = "\u{1}?storageType@Value@pvxs@@QEBA?AW4StoreType@2@XZ"]
    pub fn pvxs_Value_storageType(this: *const Value) -> StoreType;
}
unsafe extern "C" {
    #[doc = "! Type ID string (Struct or Union only)"]
    #[link_name = "\u{1}?id@Value@pvxs@@QEBAAEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@XZ"]
    pub fn pvxs_Value_id(this: *const Value) -> *const StdString;
}
unsafe extern "C" {
    #[doc = "! Test prefix of Type ID string (Struct or Union only)"]
    #[link_name = "\u{1}?idStartsWith@Value@pvxs@@QEBA_NAEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@@Z"]
    pub fn pvxs_Value_idStartsWith(this: *const Value, prefix: *const StdString) -> bool;
}
unsafe extern "C" {
    #[doc = " Return our name for a descendant field.\n @code\n   Value v = ...;\n   assert(v.nameOf(v[\"some.field\"])==\"some.field\");\n @endcode\n @throws NoField unless both this and descendant are valid()\n @throws std::logic_error if descendant is not actually a descendant"]
    #[link_name = "\u{1}?nameOf@Value@pvxs@@QEBAAEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@AEBV12@@Z"]
    pub fn pvxs_Value_nameOf(
        this: *const Value,
        descendant: *const Value,
    ) -> *const StdString;
}
unsafe extern "C" {
    #[link_name = "\u{1}?copyOut@Value@pvxs@@QEBAXPEAXW4StoreType@2@@Z"]
    pub fn pvxs_Value_copyOut(
        this: *const Value,
        ptr: *mut ::std::os::raw::c_void,
        type_: StoreType,
    );
}
unsafe extern "C" {
    #[link_name = "\u{1}?tryCopyOut@Value@pvxs@@QEBA_NPEAXW4StoreType@2@@Z"]
    pub fn pvxs_Value_tryCopyOut(
        this: *const Value,
        ptr: *mut ::std::os::raw::c_void,
        type_: StoreType,
    ) -> bool;
}
unsafe extern "C" {
    #[link_name = "\u{1}?copyIn@Value@pvxs@@QEAAXPEBXW4StoreType@2@@Z"]
    pub fn pvxs_Value_copyIn(
        this: *mut Value,
        ptr: *const ::std::os::raw::c_void,
        type_: StoreType,
    );
}
unsafe extern "C" {
    #[link_name = "\u{1}?tryCopyIn@Value@pvxs@@QEAA_NPEBXW4StoreType@2@@Z"]
    pub fn pvxs_Value_tryCopyIn(
        this: *mut Value,
        ptr: *const ::std::os::raw::c_void,
        type_: StoreType,
    ) -> bool;
}
unsafe extern "C" {
    #[doc = " Attempt to access a descendant field, or throw exception.\n\n Acts like operator[] on success, but throws a (hopefully descriptive)\n exception instead of returning an invalid Value.\n\n @throws LookupError If the lookup can not be satisfied\n @throws NoField If this Value is empty\n @since 1.1.2 An empty Value correctly throws NoField instead of returning an empty Value"]
    #[link_name = "\u{1}?lookup@Value@pvxs@@QEAA?AV12@AEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@@Z"]
    pub fn pvxs_Value_lookup(this: *mut Value, name: *const StdString) -> Value;
}
unsafe extern "C" {
    #[link_name = "\u{1}?lookup@Value@pvxs@@QEBA?BV12@AEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@@Z"]
    pub fn pvxs_Value_lookup1(this: *const Value, name: *const StdString) -> Value;
}
unsafe extern "C" {
    #[doc = "! Number of child fields.\n! only Struct, StructA, Union, UnionA return non-zero\n! \\since 1.1.3 correctly return non-zero for StructA and UnionA"]
    #[link_name = "\u{1}?nmembers@Value@pvxs@@QEBA_KXZ"]
    pub fn pvxs_Value_nmembers(this: *const Value) -> usize;
}
unsafe extern "C" {
    #[link_name = "\u{1}??1Value@pvxs@@QEAA@XZ"]
    pub fn pvxs_Value_Value_destructor(this: *mut Value);
}
impl Value {
    #[inline]
    pub unsafe fn clone_empty(&self, pvxs_library: Arc<LoadLib>) -> Value {
        pvxs_value_clone_empty(self, pvxs_library)
    }
    #[inline]
    pub unsafe fn clone(&self, pvxs_library: Arc<LoadLib>) -> Value {
        pvxs_value_clone(self, pvxs_library)
    }
    #[inline]
    pub unsafe fn assign(&mut self, o: *const Value, pvxs_library: Arc<LoadLib>) -> *mut Value {
        pvxs_value_assign(self, o, pvxs_library)
    }
    #[inline]
    pub unsafe fn alloc_member(&mut self, pvxs_library: Arc<LoadLib>) -> Value {
        pvxs_value_alloc_member(self, pvxs_library)
    }
    #[inline]
    pub unsafe fn clear(&mut self, pvxs_library: Arc<LoadLib>) {
        pvxs_value_clear(self, pvxs_library)
    }
    #[inline]
    pub unsafe fn is_marked(&self, parents: bool, children: bool, pvxs_library: Arc<LoadLib>) -> bool {
        pvxs_value_is_marked(self, parents, children, pvxs_library)
    }
    #[inline]
    pub unsafe fn if_marked(&self, parents: bool, children: bool, pvxs_library: Arc<LoadLib>) -> Value {
        pvxs_value_if_marked(self, parents, children, pvxs_library)
    }
    #[inline]
    pub unsafe fn mark(&mut self, v: bool, pvxs_library: Arc<LoadLib>) {
        pvxs_value_mark(self, v, pvxs_library)
    }
    #[inline]
    pub unsafe fn unmark(&mut self, parents: bool, children: bool, pvxs_library: Arc<LoadLib>) {
        pvxs_Value_unmark(self, parents, children, pvxs_library)
    }
    #[inline]
    pub unsafe fn type_(&self, pvxs_library: Arc<LoadLib>) -> TypeCode {
        pvxs_Value_type(self, pvxs_library)
    }
    #[inline]
    pub unsafe fn storage_type(&self, pvxs_library: Arc<LoadLib>) -> StoreType {
        pvxs_Value_storageType(self, pvxs_library)
    }
    #[inline]
    pub unsafe fn id(&self, pvxs_library: Arc<LoadLib>) -> *const StdString {
        pvxs_Value_id(self, pvxs_library)
    }
    #[inline]
    pub unsafe fn id_starts_with(&self, prefix: *const StdString, pvxs_library: Arc<LoadLib>) -> bool {
        pvxs_Value_idStartsWith(self, prefix, pvxs_library)
    }
    #[inline]
    pub unsafe fn name_of(&self, descendant: *const Value, pvxs_library: Arc<LoadLib>) -> *const StdString {
        pvxs_Value_nameOf(self, descendant, pvxs_library)
    }
    #[inline]
    pub unsafe fn copy_out(&self, ptr: *mut ::std::os::raw::c_void, type_: StoreType, pvxs_library: Arc<LoadLib>) {
        pvxs_Value_copyOut(self, ptr, type_, pvxs_library)
    }
    #[inline]
    pub unsafe fn try_copy_out(
        &self,
        ptr: *mut ::std::os::raw::c_void,
        type_: StoreType,
        pvxs_library: Arc<LoadLib>,
    ) -> bool {
        pvxs_Value_tryCopyOut(self, ptr, type_, pvxs_library)
    }
    #[inline]
    pub unsafe fn copy_in(&mut self, ptr: *const ::std::os::raw::c_void, type_: StoreType, pvxs_library: Arc<LoadLib>) {
        pvxs_Value_copyIn(self, ptr, type_, pvxs_library)
    }
    #[inline]
    pub unsafe fn try_copy_in(
        &mut self,
        ptr: *const ::std::os::raw::c_void,
        type_: StoreType,
        pvxs_library: Arc<LoadLib>,
    ) -> bool {
        pvxs_Value_tryCopyIn(self, ptr, type_, pvxs_library)
    }
    #[inline]
    pub unsafe fn lookup(&mut self, name: *const StdString, pvxs_library: Arc<LoadLib>) -> Value {
        pvxs_Value_lookup(self, name, pvxs_library)
    }
    #[inline]
    pub unsafe fn lookup1(&self, name: *const StdString, pvxs_library: Arc<LoadLib>) -> Value {
        pvxs_Value_lookup1(self, name, pvxs_library)
    }
    #[inline]
    pub unsafe fn nmembers(&self, pvxs_library: Arc<LoadLib>) -> usize {
        pvxs_Value_nmembers(self, pvxs_library)
    }
    #[inline]
    pub unsafe fn destruct(&mut self, pvxs_library: Arc<LoadLib>) {
        pvxs_Value_Value_destructor(self, pvxs_library)
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ValueIterator<T> {
    pub _phantom_0: ::std::marker::PhantomData<::std::cell::UnsafeCell<T>>,
    pub _base: T,
    pub val: Value,
    pub pos: usize,
}
#[repr(C)]
#[derive(Debug)]
pub struct ValueIterable {
    pub val: Value,
}
pub type ValueIterableIterator<T> = ValueIterator<T>;

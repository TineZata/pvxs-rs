use crate::{array::ArrayType, storetype::StoreType};

/// Possible Field types.
/// 
/// eg. String is scalar string, StringA is array of strings
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TypeCode {
    #[doc = "! the actual type code.  eg. for switch()"]
    pub code: TypeCodeType,
}
pub const TYPE_CODE_CODE_T_BOOL: TypeCodeType = 0;
pub const TYPE_CODE_CODE_T_BOOL_A: TypeCodeType = 8;
pub const TYPE_CODE_CODE_T_INT8: TypeCodeType = 32;
pub const TYPE_CODE_CODE_T_INT16: TypeCodeType = 33;
pub const TYPE_CODE_CODE_T_INT32: TypeCodeType = 34;
pub const TYPE_CODE_CODE_T_INT64: TypeCodeType = 35;
pub const TYPE_CODE_CODE_T_UINT8: TypeCodeType = 36;
pub const TYPE_CODE_CODE_T_UINT16: TypeCodeType = 37;
pub const TYPE_CODE_CODE_T_UINT32: TypeCodeType = 38;
pub const TYPE_CODE_CODE_T_UINT64: TypeCodeType = 39;
pub const TYPE_CODE_CODE_T_INT8_A: TypeCodeType = 40;
pub const TYPE_CODE_CODE_T_INT16_A: TypeCodeType = 41;
pub const TYPE_CODE_CODE_T_INT32_A: TypeCodeType = 42;
pub const TYPE_CODE_CODE_T_INT64_A: TypeCodeType = 43;
pub const TYPE_CODE_CODE_T_UINT8_A: TypeCodeType = 44;
pub const TYPE_CODE_CODE_T_UINT16_A: TypeCodeType = 45;
pub const TYPE_CODE_CODE_T_UINT32_A: TypeCodeType = 46;
pub const TYPE_CODE_CODE_T_UINT64_A: TypeCodeType = 47;
pub const TYPE_CODE_CODE_T_FLOAT32: TypeCodeType = 66;
pub const TYPE_CODE_CODE_T_FLOAT64: TypeCodeType = 67;
pub const TYPE_CODE_CODE_T_FLOAT32_A: TypeCodeType = 74;
pub const TYPE_CODE_CODE_T_FLOAT64_A: TypeCodeType = 75;
pub const TYPE_CODE_CODE_T_STRING: TypeCodeType = 96;
pub const TYPE_CODE_CODE_T_STRING_A: TypeCodeType = 104;
pub const TYPE_CODE_CODE_T_STRUCT: TypeCodeType = 128;
pub const TYPE_CODE_CODE_T_UNION: TypeCodeType = 129;
pub const TYPE_CODE_CODE_T_ANY: TypeCodeType = 130;
pub const TYPE_CODE_CODE_T_STRUCT_A: TypeCodeType = 136;
pub const TYPE_CODE_CODE_T_UNION_A: TypeCodeType = 137;
pub const TYPE_CODE_CODE_T_ANY_A: TypeCodeType = 138;
pub const TYPE_CODE_CODE_T_NULL: TypeCodeType = 255;
#[doc = "! actual complete (scalar) type code."]
pub type TypeCodeType = u8;
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
    #[doc = "! name string.  eg. \"bool\" or \"uint8_t\""]
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
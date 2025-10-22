/// epicsTypes
/// Architecture Independent Data Types
/// 
/// These are sufficient for all our current archs
/// 
pub const EPICS_BOOLEAN_EPICS_FALSE: EpicsBoolean = 0;
pub const EPICS_BOOLEAN_EPICS_TRUE: EpicsBoolean = 1;
pub type EpicsBoolean = ::std::os::raw::c_int;
pub type EpicsInt8 = ::std::os::raw::c_schar;
pub type EpicsUint8 = ::std::os::raw::c_uchar;
pub type EpicsInt16 = ::std::os::raw::c_short;
pub type EpicsUInt16 = ::std::os::raw::c_ushort;
pub type EpicsInt32 = ::std::os::raw::c_int;
pub type EpicsUInt32 = ::std::os::raw::c_uint;
pub type EpicsInt64 = ::std::os::raw::c_longlong;
pub type EpicsUInt64 = ::std::os::raw::c_ulonglong;
pub type EpicsEnum16 = EpicsUInt16;
pub type EpicsFloat32 = f32;
pub type EpicsFloat64 = f64;
pub type EpicsStatus = EpicsInt32;

pub union EpicsAny {
    pub int8: EpicsInt8,
    pub u_int8: EpicsUint8,
    pub int16: EpicsInt16,
    pub u_int16: EpicsUInt16,
    pub enum16: EpicsEnum16,
    pub int32: EpicsInt32,
    pub u_int32: EpicsUInt32,
    pub int64: EpicsInt64,
    pub u_int64: EpicsUInt64,
    pub float32: EpicsFloat32,
    pub float64: EpicsFloat64,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of epics_any"][::std::mem::size_of::<EpicsAny>() - 8usize];
    ["Alignment of epics_any"][::std::mem::align_of::<EpicsAny>() - 8usize];
    ["Offset of field: epics_any::int8"][::std::mem::offset_of!(EpicsAny, int8) - 0usize];
    ["Offset of field: epics_any::uInt8"][::std::mem::offset_of!(EpicsAny, u_int8) - 0usize];
    ["Offset of field: epics_any::int16"][::std::mem::offset_of!(EpicsAny, int16) - 0usize];
    ["Offset of field: epics_any::uInt16"][::std::mem::offset_of!(EpicsAny, u_int16) - 0usize];
    ["Offset of field: epics_any::enum16"][::std::mem::offset_of!(EpicsAny, enum16) - 0usize];
    ["Offset of field: epics_any::int32"][::std::mem::offset_of!(EpicsAny, int32) - 0usize];
    ["Offset of field: epics_any::uInt32"][::std::mem::offset_of!(EpicsAny, u_int32) - 0usize];
    ["Offset of field: epics_any::int64"][::std::mem::offset_of!(EpicsAny, int64) - 0usize];
    ["Offset of field: epics_any::uInt64"][::std::mem::offset_of!(EpicsAny, u_int64) - 0usize];
    ["Offset of field: epics_any::float32"][::std::mem::offset_of!(EpicsAny, float32) - 0usize];
    ["Offset of field: epics_any::float64"][::std::mem::offset_of!(EpicsAny, float64) - 0usize];
};
/// Union of all types
/// 
/// Strings included here as pointers only so that we support
/// large string types.
/// 
/// Arrays included here as pointers because large arrays will
/// not fit in this union
pub type EpicsAnyT = EpicsAny;
pub const EPICS_TYPE_EPICS_INT8_T: EpicsType = 0;
pub const EPICS_TYPE_EPICS_UINT8_T: EpicsType = 1;
pub const EPICS_TYPE_EPICS_INT16_T: EpicsType = 2;
pub const EPICS_TYPE_EPICS_UINT16_T: EpicsType = 3;
pub const EPICS_TYPE_EPICS_ENUM16_T: EpicsType = 4;
pub const EPICS_TYPE_EPICS_INT32_T: EpicsType = 5;
pub const EPICS_TYPE_EPICS_UINT32_T: EpicsType = 6;
pub const EPICS_TYPE_EPICS_FLOAT32_T: EpicsType = 7;
pub const EPICS_TYPE_EPICS_FLOAT64_T: EpicsType = 8;
pub const EPICS_TYPE_EPICS_STRING_T: EpicsType = 9;
pub const EPICS_TYPE_EPICS_OLD_STRING_T: EpicsType = 10;
#[doc = " \\brief Corresponding Type Codes\n (this enum must start at zero)\n\n \\note Update \\a epicsTypeToDBR_XXXX[] and \\a DBR_XXXXToEpicsType\n  in db_access.h if you edit this enum"]
pub type EpicsType = ::std::os::raw::c_int;
unsafe extern "C" {
    #[link_name = "\u{1}?epicsTypeNames@@3PAPEBDA"]
    pub static mut epicsTypeNames: [*const ::std::os::raw::c_char; 11usize];
}
unsafe extern "C" {
    #[link_name = "\u{1}?epicsTypeCodeNames@@3PAPEBDA"]
    pub static mut epicsTypeCodeNames: [*const ::std::os::raw::c_char; 11usize];
}
unsafe extern "C" {
    #[link_name = "\u{1}?epicsTypeSizes@@3QBIB"]
    pub static epicsTypeSizes: [::std::os::raw::c_uint; 11usize];
}
pub const EPICS_TYPE_CLASS_EPICS_INT_C: EpicsTypeClass = 0;
pub const EPICS_TYPE_CLASS_EPICS_UINT_C: EpicsTypeClass = 1;
pub const EPICS_TYPE_CLASS_EPICS_ENUM_C: EpicsTypeClass = 2;
pub const EPICS_TYPE_CLASS_EPICS_FLOAT_C: EpicsTypeClass = 3;
pub const EPICS_TYPE_CLASS_EPICS_STRING_C: EpicsTypeClass = 4;
pub const EPICS_TYPE_CLASS_EPICS_OLD_STRING_C: EpicsTypeClass = 5;
pub type EpicsTypeClass = ::std::os::raw::c_int;
unsafe extern "C" {
    #[link_name = "\u{1}?epicsTypeClasses@@3QBW4epicsTypeClass@@B"]
    pub static epicsTypeClasses: [EpicsTypeClass; 11usize];
}
unsafe extern "C" {
    #[link_name = "\u{1}?epicsTypeAnyFieldName@@3PAPEBDA"]
    pub static mut epicsTypeAnyFieldName: [*const ::std::os::raw::c_char; 11usize];
}
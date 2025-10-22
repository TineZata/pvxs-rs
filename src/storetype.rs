/// no associate storage
pub const STORE_TYPE_NULL: StoreType = 0;
/// bool
pub const STORE_TYPE_BOOL: StoreType = 1;
/// uint64_t
pub const STORE_TYPE_UINTEGER: StoreType = 2;
/// int64_t
pub const STORE_TYPE_INTEGER: StoreType = 3;
/// double
pub const STORE_TYPE_REAL: StoreType = 4;
/// std::string
pub const STORE_TYPE_STRING: StoreType = 5;
/// Value
pub const STORE_TYPE_COMPOUND: StoreType = 6;
/// shared_array<const void>
pub const STORE_TYPE_ARRAY: StoreType = 7;
///! selector for union FieldStorage::store"]
pub type StoreType = u8;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct UnselectT {
    pub _address: u8,
}

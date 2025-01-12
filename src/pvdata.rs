use std::collections::HashMap;
use libloading::{Library, Symbol};
use crate::storetype::StoreType;
use std::ffi::CStr;

/// Flexible recursive data structure to represent various types of PV data
#[derive(Debug, Clone)]
pub enum PVData {
    Double(f64),
    Int(i64),
    String(String),
    Structure(HashMap<String, PVData>),
    Invalid, // To represent missing or invalid fields
}

impl PVData {
    /// Helper to extract a value as a string for display
    pub fn to_string(&self) -> String {
        match self {
            PVData::Double(v) => format!("Double({})", v),
            PVData::Int(v) => format!("Int({})", v),
            PVData::String(v) => format!("String({})", v),
            PVData::Structure(v) => format!("Structure({:?})", v),
            PVData::Invalid => "Invalid".to_string(),
        }
    }

    /// Parse a `pvxs::Value` into `PVData`
    unsafe fn parse_value(raw_ptr: *mut std::ffi::c_void, lib: &Library) -> PVData {
        // Load the `valid` method
        let is_valid: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void) -> bool> = lib
            .get(b"?valid@Value@pvxs@@QBE_NXZ")
            .expect("Failed to load `valid`");

        // Check if the value is valid
        if !is_valid(raw_ptr) {
            return PVData::Invalid;
        }

        // Load storage type
        let get_storage_type: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void) -> i32> = lib
            .get(b"?storageType@Value@pvxs@@QBE?AW4StoreType@2@XZ")
            .expect("Failed to load `storageType`");

        
        // Determine the storage type of the value
        let store_type = StoreType::from(get_storage_type(raw_ptr));

        // Load the `tryCopyOut` method
        // Original C++ signature: bool Value::tryCopyOut(void *ptr, StoreType type) const
        let try_copy_out: Symbol<unsafe extern "C" fn(
            *mut std::ffi::c_void,
            *mut std::ffi::c_void,
            i32,
        ) -> bool> = lib
            .get(b"?tryCopyOut@Value@pvxs@@QBE_NPAXW4StoreType@2@@Z")
            .expect("Failed to load `tryCopyOut`");

        // Parse based on the storage type
        match store_type {
            StoreType::Real => {
                let mut value: f64 = 0.0;
                let success = try_copy_out(
                    raw_ptr,
                    &mut value as *mut _ as *mut std::ffi::c_void,
                    store_type as i32,
                );
                if success {
                    PVData::Double(value)
                } else {
                    PVData::Invalid
                }
            }
            StoreType::Integer => {
                let mut value: i64 = 0;
                let success = try_copy_out(
                    raw_ptr,
                    &mut value as *mut _ as *mut std::ffi::c_void,
                    store_type as i32,
                );
                if success {
                    PVData::Int(value)
                } else {
                    PVData::Invalid
                }
            }
            StoreType::String => {
                let mut buffer: [u8; 256] = [0; 256]; // Example buffer size for strings
                let success = try_copy_out(
                    raw_ptr,
                    buffer.as_mut_ptr() as *mut std::ffi::c_void,
                    store_type as i32,
                );
                if success {
                    let c_str = CStr::from_ptr(buffer.as_ptr() as *const i8);
                    PVData::String(c_str.to_string_lossy().to_string())
                } else {
                    PVData::Invalid
                }
            }
            _ => PVData::Invalid,
        }
    }

    /// TryCopyOut for a double
    unsafe fn parse_double(raw_ptr: *mut std::ffi::c_void, lib: &Library) -> PVData {
        let get_double: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void) -> f64> = lib
            .get(b"?tryCopyOut@Value@pvxs@@QBE_NPAXW4StoreType@2@@Z")
            .expect("Failed to load `tryCopyOut` for double");
        
        PVData::Double(get_double(raw_ptr))
    }

}

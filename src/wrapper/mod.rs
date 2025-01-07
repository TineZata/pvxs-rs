use std::ffi::CStr;

/// Wrapper for dynamically loaded library
pub fn get_version_str() -> String {
    let lib = match crate::bindings::PvxsLibrary::new() {
        Ok(lib) => lib,
        Err(_) => return "Err: PvxsLibrary failed to load".to_string(),
    };
    unsafe {
        let c_str = lib.version_str();
        if c_str.is_null() {
            "Unknown PVXS version".to_string()
        } else {
            CStr::from_ptr(c_str).to_string_lossy().into_owned()
        }
    }
}

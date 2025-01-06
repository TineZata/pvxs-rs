use std::ffi::CStr;

/// Retrieve the PVXS version string from the underlying library.
///
/// # Example
/// ```rust
/// let version = pvxs_rs::get_pvxs_version();
/// println!("PVXS Version: {}", version);
/// ```
/*pub fn get_pvxs_version() -> String {
    unsafe {
        let c_str: *const u8 = super::bindings::pvxs_version_str();
        if c_str.is_null() {
            return "Unknown PVXS version".to_string();
        }
        CStr::from_ptr(c_str as *const i8)
            .to_string_lossy()
            .into_owned()
    }
}*/
pub fn get_pvxs_version() -> String {
    unsafe {
        let c_str = super::bindings::pvxs_version_str();
        if c_str.is_null() {
            return "Unknown PVXS version".to_string();
        }
        CStr::from_ptr(c_str).to_string_lossy().into_owned()
    }
}

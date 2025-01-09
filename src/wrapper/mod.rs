use std::ffi::CStr;
use crate::version::Version;
use crate::context::Context;

/// Wrapper for dynamically loaded library
pub fn get_version_str() -> String {
    unsafe {
        let c_str = Version::version_str();
        if c_str.is_null() {
            "Unknown PVXS version".to_string()
        } else {
            CStr::from_ptr(c_str).to_string_lossy().into_owned()
        }
    }
}

pub fn client_context_from_env() -> *mut crate::Context {
    unsafe { Context::context_from_env() }
}


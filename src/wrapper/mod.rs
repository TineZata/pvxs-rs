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

pub fn client_config_new() -> *mut crate::config::Config {
    let lib = match crate::bindings::PvxsLibrary::new() {
        Ok(lib) => lib,
        Err(_) => return std::ptr::null_mut(),
    }
}

/*
pub fn client_config_from_env() -> *mut std::ffi::c_void {
    let lib = match crate::bindings::PvxsLibrary::new() {
        Ok(lib) => lib,
        Err(_) => return std::ptr::null_mut(),
    };
    unsafe { 
        let config = lib.client_config_from_env();
        if (config as *const std::ffi::c_void).is_null() {
            std::ptr::null_mut()
        } else {
            config
        }
    }
}*/

/*pub fn client_context_from_env() -> *mut std::ffi::c_void {
    let lib = match crate::bindings::PvxsLibrary::new() {
        Ok(lib) => lib,
        Err(_) => return std::ptr::null_mut(),
    };
    unsafe { 
        let context = lib.client_context_from_env();
        if (context as *const std::ffi::c_void).is_null() {
            std::ptr::null_mut()
        } else {
            context
        }
    }
}*/


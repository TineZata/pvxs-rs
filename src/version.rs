use std::{ffi::CStr, sync::Arc};
use crate::pvxs_library::PvxsLibrary;

#[repr(C)]
pub struct Version {
}

impl Version {
    /// Resolve the version string from the PVXS library
    pub unsafe fn version_str(pvxs_library: Arc<PvxsLibrary>) -> String{
        // Load the symbol for `version_str`
        let func: libloading::Symbol<unsafe extern "C" fn() -> *const std::os::raw::c_char> = 
            pvxs_library.lib
            .get(if cfg!(target_os = "windows") {
                b"?version_str@pvxs@@YAPEBDXZ"
            } else if cfg!(target_os = "linux") {
                b"_ZN4pvxs11version_strEv"
            } else {
                panic!("Unsupported platform");
            })
            .expect("Failed to find symbol for version_str");
        
        let str_ptr = func();
        if str_ptr.is_null() {
            return "Unknown PVXS version".to_string();
        }
        CStr::from_ptr(str_ptr).to_string_lossy().into_owned()
    }
}

#[repr(C)]
pub struct Version {

}

impl Version {
    /// Resolve the version string from the PVXS library
    pub unsafe fn version_str() -> *const std::os::raw::c_char {
        let pvxs_library = match crate::bindings::PvxsLibrary::new() {
            Ok(lib) => lib,
            Err(_) => return std::ptr::null_mut(),
        };
        // Load the symbol for `version_str`
        let func: libloading::Symbol<unsafe extern "C" fn() -> *const std::os::raw::c_char> = 
            pvxs_library.lib
            .get(if cfg!(target_os = "windows") {
                b"?version_str@pvxs@@YAPBDXZ"
            } else if cfg!(target_os = "linux") {
                b"_ZN4pvxs11version_strEv"
            } else {
                panic!("Unsupported platform");
            })
            .expect("Failed to find symbol for version_str");
        func()
    }
}

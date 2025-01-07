use libloading::{Library, Symbol};
use std::os::raw::c_char;

pub struct PvxsLibrary {
    lib: Library,
}

impl PvxsLibrary {
    /// Safely load the PVXS shared library.
    pub fn new() -> Result<Self, String> {
        let lib_name = if cfg!(target_os = "windows") {
            "pvxs.dll"
        } else if cfg!(target_os = "linux") {
            "libpvxs.so"
        } else {
            return Err("Unsupported platform".to_string());
        };

        // Attempt to load the library
        unsafe {
            println!("Loading library: {}", lib_name);
            match Library::new(lib_name) {
                Ok(lib) => Ok(Self { lib }),
                Err(err) => Err(format!("Failed to load library '{}': {}", lib_name, err)),
            }
        }
    }

    /// Resolve the mangled function dynamically.
    pub unsafe fn version_str(&self) -> *const c_char {
        let func: Symbol<unsafe extern "C" fn() -> *const c_char> = self
            .lib
            .get(if cfg!(target_os = "windows") {
                b"?version_str@pvxs@@YAPBDXZ"
            } else if cfg!(target_os = "linux") {
                b"_ZN4pvxs11version_strEv"
            } else {
                panic!("Unsupported platform");
            })
            .expect("Failed to find the mangled symbol");

        func()
    }
}


use libloading::{Library, Symbol};
use std::os::raw::c_char;
use std::os::raw::c_void;

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

    /// Resolve the version_str using mangled name.
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
            .expect("Failed to find the mangled symbol for version_str");

        func()
    }

    pub unsafe fn client_config_new(self) -> *mut crate::wrapper::config::Config {
        let func: Symbol<unsafe extern "C" fn() -> *mut Config> = self
            .lib
            .get(if cfg!(target_os = "windows") {
                b"??4Config@client@pvxs@@QAEAAU012@$$QAU012@@Z"
            } else if cfg!(target_os = "linux") {
                b"_undefined_linux_mangled_config_newEv"
            } else {
                panic!("Unsupported platform");
            })
            .expect("Failed to find the mangled symbol for config_new");

        func()
    } 

    /*pub unsafe fn client_config_from_env(&self) -> *mut std::ffi::c_void {
        let func: Symbol<unsafe extern "C" fn() -> *mut std::ffi::c_void> = self
            .lib
            .get(if cfg!(target_os = "windows") {
                b"?fromEnv@Config@client@pvxs@@SA?AU123@XZ"
            } else if cfg!(target_os = "linux") {
                b"_undefined_linux_mangled_config_from_envEv"
            } else {
                panic!("Unsupported platform");
            })
            .expect("Failed to find the mangled symbol for config_from_env");

        func()
    }

    pub unsafe fn client_context_from_env(&self) -> *mut std::ffi::c_void {
        let func: Symbol<unsafe extern "C" fn() -> *mut std::ffi::c_void> = self
            .lib
            .get(if cfg!(target_os = "windows") {
                b"?fromEnv@Context@client@pvxs@@SA?AV123@XZ"
            } else if cfg!(target_os = "linux") {
                b"_undefined_linux_mangled_context_from_envEv"
            } else {
                panic!("Unsupported platform");
            })
            .expect("Failed to find the mangled symbol for context_from_env");

        func()
    }*/
}


use libloading::Library;

pub struct PvxsLibrary {
    pub lib: Library,
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
            match Library::new(lib_name) {
                Ok(lib) => Ok(Self { lib }),
                Err(err) => Err(format!("Failed to load binary '{}': {}", lib_name, err)),
            }
        }
    }
}
   

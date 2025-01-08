use libloading::{Library, Symbol};

pub struct Context {
    // Pointer to the underlying C++ Context object
    context_ptr: *mut std::ffi::c_void,
    // Reference to the dynamically loaded PVXS library
    lib: Library,
}

impl Context {
    /// Load a `Context` using the static `fromEnv()` method
    pub unsafe fn context_from_env(lib: &Library) -> Result<Self, String> {
        // Load the symbol for `fromEnv`
        let func: Symbol<unsafe extern "C" fn() -> *mut std::ffi::c_void> = lib
            .get(if cfg!(target_os = "windows") {
                b"?fromEnv@Context@client@pvxs@@SA?AV123@XZ"
            } else if cfg!(target_os = "linux") {
                b"_ZN4pvxs6client7Context7fromEnvEv"
            } else {
                panic!("Unsupported platform");
            })
            .expect("Failed to find symbol for Context::fromEnv");

        // Call the `fromEnv` function to get a pointer to the C++ Context
        let context_ptr = func();
        if context_ptr.is_null() {
            return Err("Failed to create Context using fromEnv".to_string());
        }

        // Return the Context instance
        Ok(Self {
            context_ptr,
            lib: lib.clone(),
        })
    }

    /// Create a `GetBuilder` for retrieving type information
    pub unsafe fn info(&self, pv_name: &str) -> Result<GetBuilder, String> {
        // Dynamically load the `info` symbol
        let func: libloading::Symbol<
            unsafe extern "C" fn(
                *mut std::ffi::c_void,
                *const std::os::raw::c_char,
            ) -> *mut std::ffi::c_void,
        > = self
            .lib
            .get(if cfg!(target_os = "windows") {
                b"?info@Context@client@pvxs@@QAE?AVGetBuilder@23@PBD@Z"
            } else if cfg!(target_os = "linux") {
                b"_ZN5pvxs6client7Context4infoEPKc"
            } else {
                panic!("Unsupported platform");
            })
            .expect("Failed to find the symbol for Context::info");

        // Prepare the PV name
        let c_pv_name = std::ffi::CString::new(pv_name).unwrap();

        // Call the `info` method on the Context object
        let builder_ptr = func(self.context_ptr, c_pv_name.as_ptr());
        if builder_ptr.is_null() {
            return Err("Failed to create GetBuilder for info operation".to_string());
        }

        // Create and return a GetBuilder instance
        Ok(GetBuilder::new(builder_ptr, self.lib.clone()))
    }
}

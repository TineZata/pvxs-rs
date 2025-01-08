pub struct GetBuilder {
    // Pointer to the C++ GetBuilder object
    builder_ptr: *mut std::ffi::c_void,
    // Reference to the dynamically loaded PVXS library
    lib: libloading::Library,
}

impl GetBuilder {
    /// Constructs a `GetBuilder` with a pointer to the C++ object
    /// This should only be called internally by methods like `Context::info`.
    pub(crate) fn new(builder_ptr: *mut std::ffi::c_void, lib: libloading::Library) -> Self {
        Self {
            builder_ptr,
            lib,
        }
    }

    /// Executes the `info` or `get` operation
    pub unsafe fn exec(&self) -> Result<PVData, String> {
        // Dynamically load the `exec` symbol
        let func: libloading::Symbol<
            unsafe extern "C" fn(*mut std::ffi::c_void) -> *mut std::ffi::c_void,
        > = self
            .lib
            .get(if cfg!(target_os = "windows") {
                b"?exec@GetBuilder@client@pvxs@@QAE?AVOperation@23@XZ"
            } else if cfg!(target_os = "linux") {
                b"_ZN5pvxs6client10GetBuilder4execEv"
            } else {
                panic!("Unsupported platform");
            })
            .expect("Failed to find the symbol for GetBuilder::exec");

        // Call the `exec` method on the C++ object
        let result_ptr = func(self.builder_ptr);
        if result_ptr.is_null() {
            return Err("Failed to execute the operation".to_string());
        }

        // Convert the result to a Rust structure (placeholder)
        //Ok(PVData::Structure(std::collections::HashMap::new())) // Replace with actual parsing
    }
}

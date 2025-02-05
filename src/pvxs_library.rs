use libloading::{Library, Symbol};

/// Struct resposible for loading the PVXS dll and lib files.
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
                Err(err) => Err(format!("bindings:: Failed to load binary '{}': {}", lib_name, err)),
            }
        }
    }

     /// Load a symbol from the library.
    pub unsafe fn load_symbol<T>(&self, mangled_name: &str) -> Result<Symbol<T>, String> {
    self.lib
        .get::<T>(mangled_name.as_bytes())
        .map_err(|e| format!("Failed to load symbol '{}': {}", mangled_name, e))
    }
}

impl Drop for PvxsLibrary {
    fn drop(&mut self) {
        // Dummy drop implementation
    }
}

/*
pub struct PvxsLibraryManager {
    library: Arc<Mutex<Option<PvxsLibrary>>>,
}

impl PvxsLibraryManager {
    pub fn new() -> Self {
        Self {
            library: Arc::new(Mutex::new(None)),
        }
    }

    pub fn instance() -> Arc<Mutex<Option<PvxsLibrary>>> {
        static mut INSTANCE: Option<Arc<Mutex<Option<PvxsLibrary>>>> = None;
        static ONCE: std::sync::Once = std::sync::Once::new();

        unsafe {
            ONCE.call_once(|| {
                INSTANCE = Some(Arc::new(Mutex::new(None)));
            });
            INSTANCE.clone().unwrap()
        }
    }

    pub fn load_library(&self) -> Result<(), String> {
        let mut lib_lock = self.library.lock().unwrap();
        if lib_lock.is_none() {
            *lib_lock = Some(PvxsLibrary::new()?);
        }
        Ok(())
    }

    pub fn unload_library(&self) {
        let mut lib_lock = self.library.lock().unwrap();
        *lib_lock = None; // Drop the library instance
    }

    pub fn load_symbol<T>(&self, mangled_name: &str) -> Result<Symbol<T>, String> {
        let lib_lock = self.library.lock().unwrap();
        if let Some(library) = lib_lock.as_ref() {
            unsafe { library.load_symbol::<T>(mangled_name) }
        } else {
            Err("Library not loaded".to_string())
        }
    }   
}
    */

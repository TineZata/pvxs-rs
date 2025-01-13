use libloading::Symbol;
use crate::bindings::PvxsLibrary;
use crate::getbuilder::GetBuilder;

#[derive(Debug)]
#[repr(C)]
pub struct Context {
    // Opaque pointer which prevents direct access to the C++ object
    context_ptr: *mut std::ffi::c_void, 
}

impl Context {
    /// Load a `Context` using the static `fromEnv()` method
    pub unsafe fn context_from_env() -> *mut crate::Context {
        let pvxs_library = match PvxsLibrary::new() {
            Ok(lib) => lib,
            Err(_) => return std::ptr::null_mut(),
        };
        // Load the symbol for `fromEnv`
        let func: Symbol<unsafe extern "C" fn() -> *mut std::ffi::c_void> = 
            pvxs_library.lib
            .get(if cfg!(target_os = "windows") {
                b"?fromEnv@Context@client@pvxs@@SA?AV123@XZ"
            } else if cfg!(target_os = "linux") {
                b"_ZN4pvxs6client7Context7fromEnvEv"
            } else {
                panic!("Unsupported platform");
            })
            .expect("Failed to find symbol for Context::fromEnv");
        let result = func();
        dbg!(result);      
        result as *mut crate::Context
    }

    /// Create a `GetBuilder` for retrieving type information
    /// ?info@Context@client@pvxs@@QAE?AVGetBuilder@23@ABV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@@Z (public: class pvxs::client::GetBuilder __thiscall pvxs::client::Context::info(class std::basic_string<char,struct std::char_traits<char>,class std::allocator<char> > const &))
    /// GetBuilder Context::info(const std::string& name) { return GetBuilder{pvt, name, false}; }
    pub unsafe fn info(&self, pv_name: &str) -> Result<GetBuilder, String> {
        let pvxs_library = match PvxsLibrary::new() {
            Ok(lib) => lib,
            Err(_) => return Err("GetBuilder failed to load the PVXS library".to_string()),
        };
        // Dynamically load the `info` symbol
        let func: libloading::Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *const std::os::raw::c_char,) -> *mut std::ffi::c_void,> = 
            pvxs_library.lib
            .get(if cfg!(target_os = "windows") {
                b"?info@Context@client@pvxs@@QEAA?AVGetBuilder@23@AEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@@Z"
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

        dbg!(builder_ptr);
        // Create and return a GetBuilder instance
        Ok(GetBuilder::new(builder_ptr))
    }
}

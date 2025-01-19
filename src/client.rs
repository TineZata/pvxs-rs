use std::ffi::c_void;
use std::sync::Arc;
use libloading::Symbol;
use crate::pvxs_library::PvxsLibrary;

#[doc = " An independent PVA protocol client instance\n\n  Typically created with Config::build()\n\n  @code\n  Context ctxt(Config::from_env().build());\n  @endcode"]
#[repr(C)]
pub struct Context {
}

impl Context {
    /// Load a `Context` using the static `fromEnv()` method
    pub unsafe fn from_env(pvxs_library: Arc<PvxsLibrary>) -> *mut c_void {
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
        func()
    }
}

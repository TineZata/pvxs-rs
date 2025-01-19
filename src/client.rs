use std::ffi::c_void;
use std::sync::Arc;
use libloading::Symbol;
use crate::pvxs_library::PvxsLibrary;

#[doc = " An independent PVA protocol client instance\n\n  Typically created with Config::build()\n\n  @code\n  Context ctxt(Config::from_env().build());\n  @endcode"]
#[repr(C)]
pub struct Context {
}


impl Context {
    #[doc = " Create new client context based on configuration from $EPICS_PVA* environment variables.\n\n Shorthand for @code Config::fromEnv().build() @endcode.\n @since 0.2.1"]
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

    /*
    unsafe extern "C" {
    #[doc = "! effective config of running client"]
    #[link_name = "\u{1}?config@Context@client@pvxs@@QEBAAEBUConfig@23@XZ"]
    pub fn pvxs_client_Context_config(
        this: *const pvxs_client_Context,
    ) -> *const pvxs_client_Config;
}
     */
    #[doc = "! effective config of running client"]
    pub unsafe fn config(this: *const Context, pvxs_library: Arc<PvxsLibrary>) -> *const c_void {
        // Load the symbol for `config`
        let func: Symbol<unsafe extern "C" fn(*const Context) -> *const c_void> = 
            pvxs_library.lib
            .get(if cfg!(target_os = "windows") {
                b"?config@Context@client@pvxs@@QEBAAEBUConfig@23@XZ"
            } else if cfg!(target_os = "linux") {
                b"_ZN4pvxs6client7Context6configEv"
            } else {
                panic!("Unsupported platform");
            })
            .expect("Failed to find symbol for Context::config");
        func(this)
    }
}

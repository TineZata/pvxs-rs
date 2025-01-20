use std::ffi::c_void;
use std::sync::Arc;
use libloading::Symbol;
use crate::client_config::ClientConfig;
use crate::pvxs_library::PvxsLibrary;
use crate::std_shared_ptr::StdSharedPtr;

#[doc = " An independent PVA protocol client instance\n\n  Typically created with Config::build()\n\n  @code\n  Context ctxt(Config::from_env().build());\n  @endcode"]
#[repr(C)]
#[derive(Debug)]
pub struct Context {
    pub pvt: StdSharedPtr,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_client_Context"][::std::mem::size_of::<Context>() - 16usize];
    ["Alignment of pvxs_client_Context"][::std::mem::align_of::<Context>() - 8usize];
    ["Offset of field: pvxs_client_Context::pvt"][::std::mem::offset_of!(Context, pvt) - 0usize];
};

impl Context {
    #[doc = " Create new client context based on configuration from $EPICS_PVA* environment variables.\n\n Shorthand for @code Config::fromEnv().build() @endcode.\n @since 0.2.1"]
    pub unsafe fn from_env(pvxs_library: Arc<PvxsLibrary>) -> Context {
        // Load the symbol for `fromEnv`
        let func: Symbol<unsafe extern "C" fn() -> Context> = 
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

    #[doc = "! Effective config of running client"]
    pub unsafe fn config(this: *const Context, pvxs_library: Arc<PvxsLibrary>) -> *const ClientConfig {
        // Load the symbol for `config`
        let func: Symbol<unsafe extern "C" fn(*const Context) -> *const ClientConfig> = 
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

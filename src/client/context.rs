use libloading::Symbol;
use std::sync::Arc;
use crate::bin::LoadLib;
use crate::std_types::StdSharedPtr;
use super::config::Config;

/// An independent PVA protocol client instance
/// 
/// Typically created with Config::build()
/// 
/// ```cpp
/// Context ctxt(Config::from_env().build())
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct Context {
    pub pvt: StdSharedPtr,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ContextPvt {
    _unused: [u8; 0],
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_client_Context"][::std::mem::size_of::<Context>() - 16usize];
    ["Alignment of pvxs_client_Context"][::std::mem::align_of::<Context>() - 8usize];
    ["Offset of field: pvxs_client_Context::pvt"]
        [::std::mem::offset_of!(Context, pvt) - 0usize];
};

/// Create new client context based on configuration from $EPICS_PVA* environment variables.
/// 
/// Shorthand for `Config::fromEnv().build()`
/// @since 0.2.1
pub unsafe fn pvxs_client_context_from_env(pvxs_library: Arc<LoadLib>) -> Context {
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

/// effective config of running client
pub unsafe fn pvxs_client_context_config(this: *const Context, pvxs_library: Arc<LoadLib>) -> *const Config {
    // Load the symbol for `config`
    let func: Symbol<unsafe extern "C" fn(*const Context) -> *const Config> = 
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

/// Force close the client.
/// ~Context() will close() automatically.  So an explicit call is optional.
/// 
/// Aborts/interrupts all in progress network operations.
/// Blocks until any in-progress callbacks have completed.
/// 
/// @since 1.1.0
pub unsafe fn pvxs_client_context_close(this: *mut Context, pvxs_library: Arc<LoadLib>) {
    // Load the symbol for `close`
    let func: Symbol<unsafe extern "C" fn(*mut Context)> = 
        pvxs_library.lib
        .get(if cfg!(target_os = "windows") {
            b"?close@Context@client@pvxs@@QEAAXXZ"
        } else if cfg!(target_os = "linux") {
            b"_ZN4pvxs6client7Context5closeEv"
        } else {
            panic!("Unsupported platform");
        })
        .expect("Failed to find symbol for Context::close");

    func(this)
}

/// Request prompt search of any disconnected channels.
/// 
/// This method is recommended for use when executing a batch of operations.
/// 
/// ```cpp
/// Context ctxt = ...;
/// std::vector<std::string> pvnames = ...;
/// std::vector<Operation> ops(pvnames.size());
/// 
/// // Initiate all operations
/// for(size_t i=0; i<pvname.size(); i++)
/// ops[i] = ctxt.get(pvnames[i]).exec();
/// 
/// ctxt.hurryUp(); // indicate end of batch
/// for(size_t i=0; i<pvname.size(); i++)
/// ... = ops[i].wait(); // wait for results
/// ``` 
/// 
/// Optional.  Equivalent to detection of a new server.
/// This method has no effect if called more often than once per 30 seconds.
/// 
pub unsafe fn pvxs_client_context_hurry_up(this: *mut Context, pvxs_library: Arc<LoadLib>) {
    // Load the symbol for `hurryUp`
    let func: Symbol<unsafe extern "C" fn(*mut Context)> = 
        pvxs_library.lib
        .get(if cfg!(target_os = "windows") {
            b"?hurryUp@Context@client@pvxs@@QEAAXXZ"
        } else if cfg!(target_os = "linux") {
            b"_ZN4pvxs6client7Context7hurryUpEv"
        } else {
            panic!("Unsupported platform");
        })
        .expect("Failed to find symbol for Context::hurryUp");

    func(this)
}

/// Create/allocate a new client with the provided config.
/// Config::build() is a convenient shorthand.
pub unsafe fn pvxs_client_context_context(this: *mut Context, arg1: *const Config, pvxs_library: Arc<LoadLib>) {
    // Load the symbol for `Context`
    let func: Symbol<unsafe extern "C" fn(*mut Context, *const Config)> = 
        pvxs_library.lib
        .get(if cfg!(target_os = "windows") {
            b"??0Context@client@pvxs@@QEAA@AEBUConfig@12@@Z"
        } else if cfg!(target_os = "linux") {
            b"_ZN4pvxs6client7ContextC1ERKNS0_6ConfigE"
        } else {
            panic!("Unsupported platform");
        })
        .expect("Failed to find symbol for Context::Context");

    func(this, arg1)
}

pub unsafe fn pvxs_client_context_context_destructor(this: *mut Context, pvxs_library: Arc<LoadLib>) {
    // Load the symbol for `~Context`
    let func: Symbol<unsafe extern "C" fn(*mut Context)> = 
        pvxs_library.lib
        .get(if cfg!(target_os = "windows") {
            b"??1Context@client@pvxs@@QEAA@XZ"
        } else if cfg!(target_os = "linux") {
            b"_ZN4pvxs6client7ContextD1Ev"
        } else {
            panic!("Unsupported platform");
        })
        .expect("Failed to find symbol for Context::~Context");

    func(this)
}

impl Context {
    #[inline]
    pub unsafe fn from_env(pvxs_library: Arc<LoadLib>) -> Context {
        pvxs_client_context_from_env(pvxs_library)
    }
    #[inline]
    pub unsafe fn config(&self, pvxs_library: Arc<LoadLib>) -> *const Config {
        pvxs_client_context_config(self, pvxs_library)
    }
    #[inline]
    pub unsafe fn close(&mut self, pvxs_library: Arc<LoadLib>) {
        pvxs_client_context_close(self, pvxs_library)
    }
    #[inline]
    pub unsafe fn hurry_up(&mut self, pvxs_library: Arc<LoadLib>) {
        pvxs_client_context_hurry_up(self, pvxs_library)
    }
    #[inline]
    pub unsafe fn new(arg1: *const Config, pvxs_library: Arc<LoadLib>) -> Self {
        let mut __bindgen_tmp = ::std::mem::MaybeUninit::uninit();
        pvxs_client_context_context(__bindgen_tmp.as_mut_ptr(), arg1, pvxs_library);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn destruct(&mut self, pvxs_library: Arc<LoadLib>) {
        pvxs_client_context_context_destructor(self, pvxs_library)
    }
}
use std::sync::Arc;
use libloading::Symbol;
use crate::pvxs_library::PvxsLibrary;
use crate::std_types::{GetBuilder, StdSharedPtr, StdString};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Config {
    /// List of unicast, multicast, and broadcast addresses to which search requests will be sent.
    /// 
    /// Entries may take the forms:
    /// - `<ipaddr>[:<port#>]`
    /// - `<ipmultiaddr>[:<port>][,<ttl>][@<ifaceaddr>]`
    /// 
    pub address_list: StdString,
    /// List of local interface addresses on which beacons may be received.
    /// 
    /// Also constrains autoAddrList to only consider broadcast addresses of listed interfaces.
    /// Empty implies wildcard 0.0.0.0
    pub interfaces: StdString,
    /// List of TCP name servers.
    /// 
    /// Client context will maintain connections, and send search requests, to these servers.
    /// @since 0.2.0
    pub name_servers: StdString,
    /// UDP port to bind.  
    /// 
    /// Default is 5076.  
    /// May be zero, cf. Server::config() to find allocated port.
    pub udp_port: ::std::os::raw::c_ushort,
    /// Default TCP port for name servers
    /// 
    /// @since 0.2.0"
    pub tcp_port: ::std::os::raw::c_ushort,
    /// Whether to extend the addressList with local interface broadcast addresses.  (recommended)
    pub auto_addr_list: bool,
    /// Inactivity timeout interval for TCP connections.  (seconds)
    /// 
    /// Default 40.0
    /// 
    /// @since 0.2.0
    pub tcp_timeout: ::std::os::raw::c_double,
    /// Private field: Big endian flag
    _be: bool,
    /// Private field: UDP flag
    _udp: bool,
}

#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_client_Config"][::std::mem::size_of::<Config>() - 96usize];
    ["Alignment of pvxs_client_Config"][::std::mem::align_of::<Config>() - 8usize];
    ["Offset of field: pvxs_client_Config::addressList"]
        [::std::mem::offset_of!(Config, address_list) - 0usize];
    ["Offset of field: pvxs_client_Config::interfaces"]
        [::std::mem::offset_of!(Config, interfaces) - 24usize];
    ["Offset of field: pvxs_client_Config::nameServers"]
        [::std::mem::offset_of!(Config, name_servers) - 48usize];
    ["Offset of field: pvxs_client_Config::udp_port"]
        [::std::mem::offset_of!(Config, udp_port) - 72usize];
    ["Offset of field: pvxs_client_Config::tcp_port"]
        [::std::mem::offset_of!(Config, tcp_port) - 74usize];
    ["Offset of field: pvxs_client_Config::autoAddrList"]
        [::std::mem::offset_of!(Config, auto_addr_list) - 76usize];
    ["Offset of field: pvxs_client_Config::tcpTimeout"]
        [::std::mem::offset_of!(Config, tcp_timeout) - 80usize];
    ["Offset of field: pvxs_client_Config::BE"]
        [::std::mem::offset_of!(Config, _be) - 88usize];
    ["Offset of field: pvxs_client_Config::UDP"]
        [::std::mem::offset_of!(Config, _udp) - 89usize];
};

#[doc = " An independent PVA protocol client instance\n\n  Typically created with Config::build()\n\n  @code\n  Context ctxt(Config::from_env().build());\n  @endcode"]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Context {
    pub _private: StdSharedPtr<*mut std::ffi::c_void>,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_client_Context"][::std::mem::size_of::<Context>() - 16usize];
    ["Alignment of pvxs_client_Context"][::std::mem::align_of::<Context>() - 8usize];
    ["Offset of field: pvxs_client_Context::pvt"][::std::mem::offset_of!(Context, _private) - 0usize];
};

impl Context {
    /// Create new client context based on configuration from $EPICS_PVA* environment variables.
    /// 
    /// Shorthand for `cpp Config::fromEnv().build()`\n 
    /// @since 0.2.1
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

    /// Effective config of running client.
    pub unsafe fn config(this: *const Context, pvxs_library: Arc<PvxsLibrary>) -> *const Config {
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

    /// Returns a new GetBuilder instance from the context and name.
    pub unsafe fn info(this: *const Context, pvxs_library: Arc<PvxsLibrary>, name: &StdString) -> GetBuilder {
        // Load the symbol for `info`
        let func: Symbol<unsafe extern "C" fn(*const Context, &StdString) -> GetBuilder> = 
            pvxs_library.lib
            .get(if cfg!(target_os = "windows") {
                b"?info@Context@client@pvxs@@QEAAXAEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@@Z"
            } else if cfg!(target_os = "linux") {
                b"_ZN4pvxs6client7Context4infoERKNSt6__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"
            } else {
                panic!("Unsupported platform");
            })
            .expect("Failed to find symbol for Context::info");
        func(this, name)
    }
    
}

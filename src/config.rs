use libloading::Symbol;
use crate::bindings::PvxsLibrary;


///
/// PVA Network Configuration
/// Variable                        Client  Server
/// EPICS_PVA_ADDR_LIST              x       x
/// EPICS_PVAS_BEACON_ADDR_LIST              x
/// EPICS_PVA_AUTO_ADDR_LIST         x       x
/// EPICS_PVAS_AUTO_BEACON_ADDR_LIST         x
/// EPICS_PVAS_INTF_ADDR_LIST                x
/// EPICS_PVA_SERVER_PORT            x       x
/// EPICS_PVAS_SERVER_PORT                   x
/// EPICS_PVA_BROADCAST_PORT         x       x
/// EPICS_PVAS_BROADCAST_PORT                x
/// EPICS_PVAS_IGNORE_ADDR_LIST              x
/// EPICS_PVA_CONN_TMO               x       x
/// EPICS_PVA_NAME_SERVERS           x
#[derive(Debug)]
#[repr(C)]
pub struct Config {
    // Pointers to std::vector<std::string> equivalents
    address_list: *const std::ffi::c_void,
    interfaces: *const std::ffi::c_void,
    name_servers: *const std::ffi::c_void,

    // Primitive fields
    udp_port: u16,
    tcp_port: u16,
    auto_addr_list: bool,
    tcp_timeout: f64,
    
    // Private fields
    be: bool,  // Big-endian flag
    udp: bool, // UDP sharing flag

    // Padding for alignment (if needed)
    _padding: [u8; 5], // To align to 8 bytes if necessary (depends on architecture)
}

impl Config {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self {
            address_list: std::ptr::null(),
            interfaces: std::ptr::null(),
            name_servers: std::ptr::null(),
            udp_port: 5076,
            tcp_port: 5075,
            auto_addr_list: false,
            tcp_timeout: 40.0,
            be: false,
            udp: false,
            _padding: [0; 5],
        }
    }

    /// Create a new client context using the current configuration.
    /// 
    /// Original signature: ?build@Config@client@pvxs@@QEBA?AVContext@23@XZ (public: class pvxs::client::Context __thiscall pvxs::client::Config::build(void)const )
    /// 
    pub unsafe fn client_config_build(&self) -> *mut crate::Context {
        dbg!("Attempting to load library...");
        let pvxs_library = match PvxsLibrary::new() {
            Ok(lib) => lib,
            Err(_) => return std::ptr::null_mut(),
        };

        dbg!("Attempting to load symbol...");
        // Load the symbol for `build`
        //let func: Symbol<unsafe extern "C" fn(*const std::ffi::c_void) -> *mut std::ffi::c_void> = 
        let func: Symbol<unsafe extern "C" fn() -> *mut std::ffi::c_void> = 
            pvxs_library.lib
            .get(if cfg!(target_os = "windows") {
                b"?build@Config@client@pvxs@@QEBA?AVContext@23@XZ"
            } else if cfg!(target_os = "linux") {
                b"_ZNK5pvxs6client6Config5buildEv"
            } else {
                panic!("Unsupported platform");
            })
            .expect("Failed to find symbol for Config::build");
        
        //let result = func(self as *const _ as *const std::ffi::c_void);
        dbg!("Symbol loaded, calling function...");
        let result = func();
        dbg!("Func result:", result);      
        result as *mut crate::Context
    }
}

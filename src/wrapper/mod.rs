use std::sync::Arc;
use crate::client::config::Config;
use crate::client::context::Context;
//use crate::std_types::{GetBuilder, StdSSOString};
use crate::version::Version;
use crate::bin::LoadLib;
//use crate::client::{Context, Config};

/// Returns the version of the PVXS library as a string.
/// 
/// ## Example:
/// ```rust
/// let version: String = pvxs::get_version_str();
/// println!("PVXS Version: {}", version);
/// ```
pub fn get_version_str() -> String {
    let pvxs_library = Arc::new(LoadLib::new().expect("Failed to load the lib or dll"));
    let version = unsafe { Version::version_str(Arc::clone(&pvxs_library)) };
    if version.is_empty(){
         "Version string should not be empty".to_string()
    }else {
        version
    }
}

/// Returns the version of the PVXS library as a u32.
/// 
/// ## Example:
/// 
/// ```rust
/// let version: u32 = pvxs::get_version_int();
/// println!("PVXS Version: {}", version);
/// ```
pub fn get_version_int() -> u32 {
    let pvxs_library = Arc::new(LoadLib::new().expect("Failed to load the lib or dll"));
    unsafe { Version::version_int(Arc::clone(&pvxs_library)) as u32 }
}

/// Returns the ABI version of the PVXS library as a u32.
/// 
/// ## Example:
/// 
/// ```rust
/// let version: u32 = pvxs::get_version_abi_int();
/// println!("PVXS ABI Version: {}", version);
/// ```
pub fn get_version_abi_int() -> u32 {
    let pvxs_library = Arc::new(LoadLib::new().expect("Failed to load the lib or dll"));
    unsafe { Version::version_abi_int(Arc::clone(&pvxs_library)) as u32 }
}

/// Returns a new context created from the environment.
/// 
/// ## Example:
/// ```rust
/// use pvxs::client::Context;
/// 
/// let ctx: Context = pvxs::get_context_from_env();
/// ```
pub fn get_context_from_env() -> Context {
    let pvxs_library = Arc::new(LoadLib::new().expect("Failed to load the PvxsLibrary"));
    unsafe { Context::from_env(Arc::clone(&pvxs_library)) }
}

/// Return default configuration for the context.
/// 
/// ## Example:
/// ```rust
/// use pvxs::client::Config;
/// 
/// let config: Config = pvxs::get_client_config();
/// println!("Address list: {}", unsafe { config.address_list.to_rust_string_lossy() });
/// println!("Interfaces: {}", unsafe { config.interfaces.to_rust_string_lossy() });
/// println!("Name servers: {}", unsafe { config.name_servers.to_rust_string_lossy() });
/// println!("UDP port: {}", config.udp_port);
/// println!("TCP port: {}", config.tcp_port);
/// println!("TCP timeout: {}", config.tcp_timeout);
/// println!("Auto address list: {}", if config.auto_addr_list { "true" } else { "false" });
/// ```
pub fn get_client_config() -> Config {
    let pvxs_library = Arc::new(LoadLib::new().expect("Failed to load the PvxsLibrary"));
    let ctx: Context = unsafe { Context::from_env(Arc::clone(&pvxs_library)) };
    let config: *const Config = unsafe { Context::config(&ctx, Arc::clone(&pvxs_library)) };
    let config: &Config = unsafe { &*config };
    config.clone()
}

/*pub fn get_context_info(name: String) -> GetBuilder {
    let pvxs_library = Arc::new(PvxsLibrary::new().expect("Failed to load the PvxsLibrary"));
    let ctx: Context = unsafe { Context::from_env(Arc::clone(&pvxs_library)) };
    let std_string = &StdSSOString::from_rust_string(name);
    let info: GetBuilder = unsafe {
        Context::info(&ctx, Arc::clone(&pvxs_library), std_string )
    };
    info
}*/

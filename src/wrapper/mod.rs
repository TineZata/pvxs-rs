use std::sync::Arc;
use crate::version::Version;
use crate::pvxs_library::PvxsLibrary;
use crate::client::{Context, Config};

/// Returns the version of the PVXS library as a string.
/// 
/// ## Example:
/// ```rust
/// use pvxs::version::get_version_str;
/// 
/// let version: String = pvxs::get_version_str();
/// println!("PVXS Version: {}", version);
/// ```
/// 
pub fn get_version_str() -> String {
    let pvxs_library = Arc::new(PvxsLibrary::new().expect("Failed to load the PvxsLibrary"));
    let version = unsafe { Version::version_str(pvxs_library) };
    if version.is_empty(){
         "Version string should not be empty".to_string()
    }else {
        version
    }
}

/// Returns a new context created from the environment.
/// 
/// ## Example:
/// ```rust
/// use pvxs::client::Context;
/// 
/// let ctx: Context = pvxs::get_context_from_env();
/// if ctx._private._base._ptr.is_null() {
///     println!("Failed to create context from environment");
///     return;
/// } else {
///     ctx
/// }
/// ```
///
pub fn get_context_from_env() -> Context {
    let pvxs_library = Arc::new(PvxsLibrary::new().expect("Failed to load the PvxsLibrary"));
    unsafe { Context::from_env(Arc::clone(&pvxs_library)) }
}

/// Return default configuration for the context.
/// 
/// ## Example:
/// ```rust
/// use pvxs::client::Config;
/// 
/// let config: Config = pvxs::get_context_config();
/// let addr = unsafe { config.address_list.to_rust_string() };
/// println!("Address list: {}", addr);
/// 
/// let interfaces = unsafe { config.interfaces.to_rust_string() };
/// println!("Interfaces: {}", interfaces);
/// 
/// let name_servers = unsafe { config.name_servers.to_rust_string() };
/// println!("Name servers: {}", name_servers);
/// 
/// println!("UDP port: {}", config.udp_port);
/// println!("TCP port: {}", config.tcp_port);
/// println!("TCP timeout: {}", config.tcp_timeout);
/// println!("Auto address list: {}", if config.auto_addr_list { "true" } else { "false" });
/// 
/// config
/// ```
/// 
pub fn get_context_config() -> Config {
    let pvxs_library = Arc::new(PvxsLibrary::new().expect("Failed to load the PvxsLibrary"));
    let ctx: Context = unsafe { Context::from_env(Arc::clone(&pvxs_library)) };
    let config: *const Config = unsafe { Context::config(&ctx, Arc::clone(&pvxs_library)) };
    let config_obj: &Config = unsafe { &*config };
    config_obj.clone()
}

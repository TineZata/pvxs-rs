use crate::version::Version;
use crate::ClientContext;
//use crate::config::Config;

/// Wrapper for dynamically loaded library
pub fn get_version_str() -> String {
    unsafe { Version::version_str() }
}

/*
pub fn client_config_build() -> *mut ClientContext {
    unsafe { 
        // Return struct literal for Config, which creates a new context
        // using the current configuration
        Config::new().client_config_build() 
    }
}
*/
pub fn client_context_from_env() -> *mut ClientContext {
    unsafe { ClientContext::context_from_env() }
}

/*
pub fn client_context_info(ctx: *mut ClientContext, pv_name: &str) -> Result<crate::GetBuilder, String> {
    unsafe { (*ctx).info(pv_name) }
}
    */


use crate::version::Version;
use crate::context::Context;

/// Wrapper for dynamically loaded library
pub fn get_version_str() -> String {
    unsafe { Version::version_str() }
}

pub fn client_context_from_env() -> *mut crate::Context {
    unsafe { Context::context_from_env() }
}


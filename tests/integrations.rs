use std::sync::Arc;
use pvxs::pvxs_library::PvxsLibrary;
use pvxs::version::Version;
use pvxs::client::{Context, Config};

#[test]
fn test1_pvxs_version() {
    let pvxs_library = Arc::new(PvxsLibrary::new().expect("Failed to load the PvxsLibrary"));
    let version = unsafe { Version::version_str(pvxs_library) };
    assert!(!version.is_empty(), "Version string should not be empty");
    dbg!(version);
}

#[test]
fn test2_pvxs_client_context_from_env() {
    let pvxs_library = Arc::new(PvxsLibrary::new().expect("Failed to load the PvxsLibrary"));
    // Create a context
    let ctx: Context = unsafe { Context::from_env(Arc::clone(&pvxs_library)) };
    // Assert that the shared pointer is valid
    assert!(!ctx._private._base._ptr.is_null(), "Context pointer should be valid");
    //dbg!(ctx);
}

#[test]
fn test3_pvxs_client_context_config() {
    let pvxs_library = Arc::new(PvxsLibrary::new().expect("Failed to load the PvxsLibrary"));
    let ctx: Context = unsafe { Context::from_env(Arc::clone(&pvxs_library)) };
    assert!(!ctx._private._base._ptr.is_null(), "Failed to create context from environment");
    let config: *const Config = unsafe { Context::config(&ctx, Arc::clone(&pvxs_library)) };
    let config_obj: &Config = unsafe { &*config };
    assert_eq!(config_obj.udp_port, 5076, "UDP port should be default 5076");
    assert_eq!(config_obj.tcp_port, 5075, "TCP port should be default 5075");
    assert_eq!(config_obj.tcp_timeout, 40.0, "TCP timeout should be default 40.0s");
    //dbg!(config_obj);
}


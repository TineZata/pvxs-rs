use std::sync::Arc;
//use pvxs::client_config::ClientConfig;
use pvxs::pvxs_library::PvxsLibrary;
//use pvxs::std_shared_ptr::StdSharedPtr;
use pvxs::version::Version;
use pvxs::client::Context;

#[test]
fn test_pvxs_version() {
    let pvxs_library: PvxsLibrary = match PvxsLibrary::new() {
        Ok(lib) => lib,
        Err(_) => panic!("Failed to load the PvxsLibrary"),
    };
    let version = unsafe { Version::version_str(pvxs_library) };
    assert!(!version.is_empty(), "Version string should not be empty");
    dbg!(version);
}

#[test]
fn test_pvxs_client_context_from_env() {
    unsafe {
        let ctx = {
            let pvxs_library = Arc::new(PvxsLibrary::new().expect("Failed to load the PvxsLibrary"));
            let ctx: Context = Context::from_env(Arc::clone(&pvxs_library));
            assert!(!ctx.pvt._base._ptr.is_null(), "Context base pointer should not be null");    
            ctx
        };
        //dbg!(ctx);
    }
}

#[test]
fn test_pvxs_client_context_config() {
    unsafe {
        let pvxs_library = Arc::new(PvxsLibrary::new().expect("Failed to load the PvxsLibrary"));
        let ctx: Context = Context::from_env(Arc::clone(&pvxs_library));
        assert!(!ctx.pvt._base._ptr.is_null(), "Failed to create context from environment");
        let config = Context::config(&ctx, Arc::clone(&pvxs_library));
        let config_obj = &*config;
        assert_eq!(config_obj.udp_port, 5076, "UDP port should be default 5076");
        //dbg!(config_obj);
    }
}


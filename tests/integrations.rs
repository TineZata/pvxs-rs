use std::ffi::c_void;
use std::sync::Arc;

use pvxs::pvxs_library::PvxsLibrary;
use pvxs::version::Version;
use pvxs::client_context::ClientContext;

#[test]
fn test_pvxs_version() {
    let pvxs_library: PvxsLibrary = match PvxsLibrary::new() {
        Ok(lib) => lib,
        Err(_) => panic!("Failed to load the PvxsLibrary"),
    };
    let version = unsafe { Version::version_str(pvxs_library) };
    
    println!("PVXS Version: {}", version);
    assert!(!version.is_empty(), "Version string should not be empty");
}
/*
#[test]
fn test_pvxs_client_config_build() {
    let ctx = pvxs::client_config_build();
    dbg!("Context:", ctx);
    assert!(!ctx.is_null(), "Failed to create context from configuration");
}
    */
#[test]
fn test_pvxs_client_context_from_env() {
    // Arc ensure that the PvxsLibrary is shared between threads and cleaned up when the last reference is dropped
    unsafe {
        let pvxs_library = Arc::new(PvxsLibrary::new().expect("Failed to load the PvxsLibrary"));
        let ctx_raw: *mut c_void =  ClientContext::context_from_env(Arc::clone(&pvxs_library));
        println!("Context: {:?}", ctx_raw);
        assert!(!ctx_raw.is_null(), "Failed to create context from environment");
    };
}




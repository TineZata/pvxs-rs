use std::sync::Arc;
use pvxs::pvxs_library::PvxsLibrary;
use pvxs::std_shared_ptr::StdSharedPtr;
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
        let pvxs_library = Arc::new(PvxsLibrary::new().expect("Failed to load the PvxsLibrary"));
        let shared_ptr: *mut StdSharedPtr = Context::from_env(Arc::clone(&pvxs_library)) as *mut StdSharedPtr;
        assert!(!shared_ptr.is_null(), "Failed to create context from environment");
        dbg!(shared_ptr);
    }
}




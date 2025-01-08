#[test]
fn test_pvxs_version() {
    let version = pvxs::get_version_str();
    println!("PVXS Version: {}", version);
    assert!(!version.is_empty(), "Version string should not be empty");
}

#[test]
fn test_pvxs_client_context_from_env() {
    let ctx: *mut pvxs::Context = pvxs::wrapper::client_context_from_env();
    assert!(!ctx.is_null(), "Failed to create context from environment");
}




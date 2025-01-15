#[test]
fn test_pvxs_version() {
    let version = pvxs::get_version_str();
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
    let ctx: *mut pvxs::ClientContext = pvxs::wrapper::client_context_from_env();
    //println!("Context: {:?}", ctx);
    assert!(!ctx.is_null(), "Failed to create context from environment");
}




use pvxs_rs::get_pvxs_version;

#[test]
fn test_pvxs_version() {
    let version = get_pvxs_version();
    println!("PVXS version: {}", version);
    assert!(!version.is_empty(), "PXVS version string should not be empty");
}

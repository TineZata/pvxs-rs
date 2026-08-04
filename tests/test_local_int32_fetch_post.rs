// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{NTScalarMetadataBuilder, Server};

#[test]
fn test_pv_local_fetch_post() {
    let initial_value = 100;
    let name = "loc:int";
    let server = Server::start_isolated().expect("start isolated server");

    server
        .create_pv_int32(name, initial_value, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let fetched = server.fetch_int32(name).expect("fetch initial");
    assert_eq!(fetched.value, initial_value);

    assert!(server.post_double(name, std::f64::consts::PI).is_err());
    assert!(server.post_string(name, "invalid").is_err());

    let new_value = 200;
    server.post_int32(name, new_value).expect("post int32");
    let fetched = server.fetch_int32(name).expect("fetch posted");
    assert_eq!(fetched.value, new_value);
}

#[test]
fn test_pv_local_fetch_post_with_error_propagation() {
    let initial_value = 1234;
    let name = "loc:int:err";
    let server = Server::start_isolated().expect("start isolated server");

    server
        .create_pv_int32(name, initial_value, NTScalarMetadataBuilder::new())
        .expect("create pv");

    assert!(server.post_string(name, "invalid_value").is_err());

    let fetched = server.fetch_int32(name).expect("fetch after error");
    assert_eq!(fetched.value, initial_value);

    let new_value = 5678;
    server.post_int32(name, new_value).expect("post valid int32");
    let fetched = server.fetch_int32(name).expect("fetch valid int32");
    assert_eq!(fetched.value, new_value);
}
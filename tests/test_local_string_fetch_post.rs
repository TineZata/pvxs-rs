// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{NTScalarMetadataBuilder, Server};

#[test]
fn test_pv_local_string_fetch_post() {
    let initial_value = "Hello, EPICS!";
    let name = "loc:string";
    let server = Server::start_isolated().expect("start isolated server");

    server
        .create_pv_string(name, initial_value, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let fetched = server.fetch_string(name).expect("fetch initial");
    assert_eq!(fetched.value, initial_value);

    let new_value = "Updated string value";
    server.post_string(name, new_value).expect("post string");
    let fetched = server.fetch_string(name).expect("fetch posted");
    assert_eq!(fetched.value, new_value);
}

#[test]
fn test_pv_local_string_fetch_post_with_error_propagation() {
    let initial_value = "Initial string";
    let name = "loc:string:error";
    let server = Server::start_isolated().expect("start isolated server");

    server
        .create_pv_string(name, initial_value, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let fetched = server.fetch_string(name).expect("fetch initial");
    assert_eq!(fetched.value, initial_value);

    let new_value = "New string value";
    server.post_string(name, new_value).expect("post string");
    let fetched = server.fetch_string(name).expect("fetch posted");
    assert_eq!(fetched.value, new_value);
}

#[test]
fn test_pv_local_string_special_characters() {
    let name = "loc:string:special";
    let server = Server::start_isolated().expect("start isolated server");

    server
        .create_pv_string(name, "", NTScalarMetadataBuilder::new())
        .expect("create pv");

    let cases = [
        "",
        "Hello, World! @#$%^&*()",
        "Line 1\\nLine 2\\tTabbed",
        "ASCII only test payload",
    ];

    for s in cases {
        server.post_string(name, s).expect("post string case");
        let fetched = server.fetch_string(name).expect("fetch string case");
        assert_eq!(fetched.value, s);
    }

    let long_string = "A".repeat(1000);
    server
        .post_string(name, &long_string)
        .expect("post long string");
    let fetched = server.fetch_string(name).expect("fetch long string");
    assert_eq!(fetched.value, long_string);
}
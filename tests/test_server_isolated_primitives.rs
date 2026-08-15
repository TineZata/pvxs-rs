// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{NTEnumMetadataBuilder, NTScalarMetadataBuilder, Server};

#[test]
fn isolated_server_supports_primitive_operations() {
    let server = Server::start_isolated().expect("start server");
    assert!(server.tcp_port() > 0);
    assert!(server.udp_port() > 0);

    server
        .create_pv_double(
            "test:primitive:double",
            42.0,
            NTScalarMetadataBuilder::new(),
        )
        .expect("create double");
    server
        .create_pv_int32("test:primitive:int32", 123, NTScalarMetadataBuilder::new())
        .expect("create int32");
    server
        .create_pv_string(
            "test:primitive:string",
            "hello",
            NTScalarMetadataBuilder::new(),
        )
        .expect("create string");
    server
        .create_pv_enum(
            "test:primitive:enum",
            vec!["ONE", "TWO", "THREE"],
            0,
            NTEnumMetadataBuilder::new(),
        )
        .expect("create enum");

    server
        .post_double("test:primitive:double", 84.0)
        .expect("post double");
    server
        .post_int32("test:primitive:int32", 456)
        .expect("post int32");
    server
        .post_string("test:primitive:string", "world")
        .expect("post string");
    server
        .post_enum("test:primitive:enum", 2)
        .expect("post enum");

    assert_eq!(
        server
            .fetch_double("test:primitive:double")
            .expect("fetch double")
            .value,
        84.0
    );
    assert_eq!(
        server
            .fetch_int32("test:primitive:int32")
            .expect("fetch int32")
            .value,
        456
    );
    assert_eq!(
        server
            .fetch_string("test:primitive:string")
            .expect("fetch string")
            .value,
        "world"
    );
    let fetched_enum = server
        .fetch_enum("test:primitive:enum")
        .expect("fetch enum");
    assert_eq!(fetched_enum.value, 2);
    assert_eq!(fetched_enum.value_choices, vec!["ONE", "TWO", "THREE"]);

    server.stop_drop().expect("stop server");
}

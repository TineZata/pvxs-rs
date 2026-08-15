// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{NTScalarMetadataBuilder, Server};

#[test]
fn duplicate_pv_names_are_rejected() {
    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_int32(
            "test:tracking:duplicate",
            42,
            NTScalarMetadataBuilder::new(),
        )
        .expect("create first pv");
    let duplicate = server.create_pv_int32(
        "test:tracking:duplicate",
        99,
        NTScalarMetadataBuilder::new(),
    );
    assert!(duplicate.is_err());
    assert!(duplicate
        .unwrap_err()
        .to_string()
        .contains("already exists"));
    server.stop_drop().expect("stop server");
}

#[test]
fn removed_pv_name_can_be_reused() {
    let server = Server::start_isolated().expect("start server");
    let name = "test:tracking:readd";
    server
        .create_pv_double(name, 1.23, NTScalarMetadataBuilder::new())
        .expect("create first pv");
    server.remove_pv(name).expect("remove pv");
    assert!(server.fetch_double(name).is_err());
    server
        .create_pv_double(name, 4.56, NTScalarMetadataBuilder::new())
        .expect("recreate pv");
    assert_eq!(
        server.fetch_double(name).expect("fetch recreated pv").value,
        4.56
    );
    server.stop_drop().expect("stop server");
}

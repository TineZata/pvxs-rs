// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{NTScalarMetadataBuilder, Server};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

#[test]
fn started_server_listens_and_stop_invalidates_handle() {
    let server = Server::start_isolated().expect("start server");
    let handle = server.handle();
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server.tcp_port());
    assert!(TcpStream::connect_timeout(&address, Duration::from_millis(500)).is_ok());

    server
        .create_pv_double("test:lifecycle:value", 7.5, NTScalarMetadataBuilder::new())
        .expect("create pv");
    assert_eq!(
        handle
            .fetch_double("test:lifecycle:value")
            .expect("fetch before stop")
            .value,
        7.5
    );

    server.stop_drop().expect("stop server");
    assert!(handle.fetch_double("test:lifecycle:value").is_err());
    assert!(TcpStream::connect_timeout(&address, Duration::from_millis(200)).is_err());
}

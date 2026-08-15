// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{NTScalarMetadataBuilder, Server};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

#[test]
fn environment_server_uses_configured_ephemeral_ports() {
    std::env::set_var("EPICS_PVA_SERVER_PORT", "0");
    std::env::set_var("EPICS_PVA_BROADCAST_PORT", "0");
    let server = Server::start_from_env().expect("start environment server");
    server
        .create_pv_double(
            "test:environment:double",
            42.0,
            NTScalarMetadataBuilder::new(),
        )
        .expect("create double pv");

    assert!(server.tcp_port() > 0);
    assert!(server.udp_port() > 0);
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server.tcp_port());
    assert!(TcpStream::connect_timeout(&address, Duration::from_millis(500)).is_ok());
    assert_eq!(
        server
            .fetch_double("test:environment:double")
            .expect("fetch double")
            .value,
        42.0
    );

    server.stop_drop().expect("stop server");
}

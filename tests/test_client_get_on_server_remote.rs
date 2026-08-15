// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{Context, NTScalarMetadataBuilder, Server};

fn context_for(server: &Server) -> Context {
    std::env::set_var("EPICS_PVA_ADDR_LIST", format!("127.0.0.1:{}", server.udp_port()));
    std::env::set_var("EPICS_PVA_AUTO_ADDR_LIST", "NO");
    std::env::set_var("EPICS_PVA_BROADCAST_PORT", server.udp_port().to_string());
    Context::from_env().expect("context from env")
}

#[test]
fn client_get_reads_server_value() {
    let server = Server::start_isolated().expect("start server");
    let name = "test:remote:get:int32";
    server
        .create_pv_int32(name, 100, NTScalarMetadataBuilder::new())
        .expect("create int32 pv");

    let mut context = context_for(&server);
    let value = context.get(name, 2.0).expect("get int32");
    assert_eq!(value.get_field_int32("value").expect("value"), 100);

    server.stop_drop().expect("stop server");
}

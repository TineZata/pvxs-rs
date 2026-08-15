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
fn server_and_client_observe_consistent_updates() {
    let server = Server::start_isolated().expect("start server");
    let name = "test:interaction:double";
    server
        .create_pv_double(name, 123.45, NTScalarMetadataBuilder::new())
        .expect("create double pv");
    let mut context = context_for(&server);

    let initial = context.get(name, 2.0).expect("get initial value");
    assert_eq!(initial.get_field_double("value").expect("value"), 123.45);

    server.post_double(name, 999.99).expect("server post");
    let updated = context.get(name, 2.0).expect("get updated value");
    assert_eq!(updated.get_field_double("value").expect("value"), 999.99);

    context.put_double(name, 42.5, 2.0).expect("client put");
    assert_eq!(server.fetch_double(name).expect("server fetch").value, 42.5);

    server.stop_drop().expect("stop server");
}

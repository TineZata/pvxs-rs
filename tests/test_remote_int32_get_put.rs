// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{Context, NTScalarMetadataBuilder, Server};
use std::sync::{Mutex, MutexGuard, OnceLock};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn lock_env() -> MutexGuard<'static, ()> {
    match env_lock().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn setup_client_for(server: &Server) -> Context {
    std::env::set_var("EPICS_PVA_ADDR_LIST", format!("127.0.0.1:{}", server.udp_port()));
    std::env::set_var("EPICS_PVA_AUTO_ADDR_LIST", "NO");
    std::env::set_var("EPICS_PVA_BROADCAST_PORT", server.udp_port().to_string());
    Context::from_env().expect("context from env")
}

#[test]
fn test_pv_remote_int_get_put() {
    let _guard = lock_env();
    let timeout = 2.0;
    let initial_value = 50;
    let name = "remote:int";

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_int32(name, initial_value, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    let first = ctx.get(name, timeout).expect("initial get");
    assert_eq!(first.get_field_int32("value").expect("value"), initial_value);

    server.stop_drop().expect("stop server");
    assert!(ctx.get(name, 0.2).is_err());

    let server = Server::start_isolated().expect("restart server");
    server
        .create_pv_int32(name, initial_value, NTScalarMetadataBuilder::new())
        .expect("recreate pv");

    let mut ctx = setup_client_for(&server);
    let new_value = 150;
    ctx.put_int32(name, new_value, timeout).expect("put int32");
    let second = ctx.get(name, timeout).expect("second get");
    assert_eq!(second.get_field_int32("value").expect("value"), new_value);

    server.stop_drop().expect("stop server");
}
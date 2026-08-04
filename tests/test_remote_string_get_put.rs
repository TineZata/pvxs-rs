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
fn test_pv_remote_string_get_put() {
    let _guard = lock_env();
    let timeout = 2.0;
    let initial_value = "Remote string PV";
    let name = "remote:string";

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_string(name, initial_value, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    let first = ctx.get(name, timeout).expect("initial get");
    assert_eq!(first.get_field_string("value").expect("value"), initial_value);

    server.stop_drop().expect("stop server");
    assert!(ctx.get(name, 0.2).is_err());

    let server = Server::start_isolated().expect("restart server");
    server
        .create_pv_string(name, initial_value, NTScalarMetadataBuilder::new())
        .expect("recreate pv");

    let mut ctx = setup_client_for(&server);
    let new_value = "Updated remote string";
    ctx.put_string(name, new_value, timeout).expect("put string");
    let second = ctx.get(name, timeout).expect("second get");
    assert_eq!(second.get_field_string("value").expect("value"), new_value);

    server.stop_drop().expect("stop server");
}

#[test]
fn test_pv_remote_string_encoding() {
    let _guard = lock_env();
    let timeout = 2.0;
    let name = "remote:string:encoding";

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_string(name, "", NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    let test_strings = [
        "Simple ASCII",
        "Numbers: 1234567890",
        "Punctuation: !@#$%^&*()",
        "Mixed: Hello world 42",
    ];

    for s in test_strings {
        ctx.put_string(name, s, timeout).expect("put string");
        let value = ctx.get(name, timeout).expect("get string");
        assert_eq!(value.get_field_string("value").expect("value"), s);
    }

    server.stop_drop().expect("stop server");
}
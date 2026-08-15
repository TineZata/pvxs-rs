// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{Context, NTEnumMetadataBuilder, NTScalarMetadataBuilder, Server};
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
fn created_double_pv_accepts_remote_put() {
    let _guard = lock_env();
    let timeout = 2.0;
    let name = "source:double";

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_double(name, 3.25, NTScalarMetadataBuilder::new())
        .expect("create double pv");

    let mut ctx = setup_client_for(&server);
    let first = ctx.get(name, timeout).expect("initial get");
    assert!((first.get_field_double("value").expect("value") - 3.25).abs() < 1e-12);

    ctx.put_double(name, 7.0, timeout).expect("put double");
    let fetched = server.fetch_double(name).expect("fetch after put");
    assert!((fetched.value - 7.0).abs() < 1e-12);

    server.stop_drop().expect("stop server");
}

#[test]
fn created_int32_pv_accepts_remote_put() {
    let _guard = lock_env();
    let timeout = 2.0;
    let name = "source:int32";

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_int32(name, 5, NTScalarMetadataBuilder::new())
        .expect("create int32 pv");

    let mut ctx = setup_client_for(&server);
    ctx.put_int32(name, 42, timeout).expect("put int32");
    let fetched = server.fetch_int32(name).expect("fetch int32");
    assert_eq!(fetched.value, 42);

    server.stop_drop().expect("stop server");
}

#[test]
fn created_enum_pv_keeps_strict_remote_shape() {
    let _guard = lock_env();
    let timeout = 2.0;
    let name = "source:enum";

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_enum(
            name,
            vec!["DISABLED", "ENABLED", "TESTING"],
            1,
            NTEnumMetadataBuilder::new(),
        )
        .expect("create enum pv");

    let mut ctx = setup_client_for(&server);
    let value = ctx.get(name, timeout).expect("get enum");
    assert_eq!(value.get_field_enum("value").expect("value"), 1);
    assert_eq!(value.get_field_enum("value.index").expect("value.index"), 1);
    assert_eq!(
        value
            .get_field_string_array("value.choices")
            .expect("value.choices"),
        vec![
            "DISABLED".to_string(),
            "ENABLED".to_string(),
            "TESTING".to_string(),
        ]
    );
    ctx.put_enum(name, 0, timeout).expect("put enum");
    assert_eq!(server.fetch_enum(name).expect("fetch enum").value, 0);

    server.stop_drop().expect("stop server");
}

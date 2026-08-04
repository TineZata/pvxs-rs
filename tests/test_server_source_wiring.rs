// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{
    Context, NTEnumMetadataBuilder, NTScalarMetadataBuilder, Server, SharedPV, StaticSource,
};
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
fn readonly_source_pv_rejects_remote_put() {
    let _guard = lock_env();
    let timeout = 2.0;
    let name = "source:readonly:double";

    let server = Server::start_isolated().expect("start server");
    let mut pv = SharedPV::create_readonly().expect("create readonly");
    pv.open_double(3.25, NTScalarMetadataBuilder::new())
        .expect("open double");

    let mut source = StaticSource::new();
    source.add(name, pv).expect("add source pv");
    server.add_source(source).expect("attach source");

    let mut ctx = setup_client_for(&server);
    let first = ctx.get(name, timeout).expect("initial get");
    assert!((first.get_field_double("value").expect("value") - 3.25).abs() < 1e-12);

    assert!(ctx.put_double(name, 7.0, timeout).is_err());
    let fetched = server.fetch_double(name).expect("fetch after failed put");
    assert!((fetched.value - 3.25).abs() < 1e-12);

    server.stop_drop().expect("stop server");
}

#[test]
fn mailbox_source_pv_accepts_remote_put() {
    let _guard = lock_env();
    let timeout = 2.0;
    let name = "source:mailbox:int32";

    let server = Server::start_isolated().expect("start server");
    let mut pv = SharedPV::create_mailbox().expect("create mailbox");
    pv.open_int32(5, NTScalarMetadataBuilder::new())
        .expect("open int32");

    server.add_shared_pv(name, pv).expect("attach shared pv");

    let mut ctx = setup_client_for(&server);
    ctx.put_int32(name, 42, timeout).expect("put int32");
    let fetched = server.fetch_int32(name).expect("fetch int32");
    assert_eq!(fetched.value, 42);

    server.stop_drop().expect("stop server");
}

#[test]
fn readonly_enum_source_keeps_strict_remote_shape() {
    let _guard = lock_env();
    let timeout = 2.0;
    let name = "source:readonly:enum";

    let server = Server::start_isolated().expect("start server");
    let mut pv = SharedPV::create_readonly().expect("create readonly enum");
    pv.open_enum(
        vec!["DISABLED", "ENABLED", "TESTING"],
        1,
        NTEnumMetadataBuilder::new(),
    )
    .expect("open enum");

    server.add_shared_pv(name, pv).expect("attach enum pv");

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
    assert!(ctx.put_enum(name, 0, timeout).is_err());

    server.stop_drop().expect("stop server");
}

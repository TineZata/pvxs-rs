// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{Context, NTEnumMetadataBuilder, NTScalarMetadataBuilder, Server};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Duration;

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
fn loopback_get_put_double_roundtrip() {
    let _guard = lock_env();

    let server = Server::start_isolated().expect("server start");
    assert!(server.udp_port() > 0, "udp port should be bound");
    assert!(server.tcp_port() > 0, "tcp port should be bound");

    server
        .create_pv_double("pv:double", 1.0, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);

    ctx.put_double("pv:double", 4.25, 2.0).expect("put double");
    let got = ctx.get("pv:double", 2.0).expect("get double");
    assert!((got.get_field_double("value").expect("value") - 4.25).abs() < 1e-12);

    server.stop_drop().expect("server stop");
}

#[test]
fn loopback_get_put_string_array_roundtrip() {
    let _guard = lock_env();

    let server = Server::start_isolated().expect("server start");
    server
        .create_pv_string_array(
            "pv:sa",
            vec!["a".to_string(), "b".to_string()],
            NTScalarMetadataBuilder::new(),
        )
        .expect("create pv");

    let mut ctx = setup_client_for(&server);

    let new_val = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
    ctx.put_string_array("pv:sa", new_val.clone(), 2.0)
        .expect("put string array");
    let got = ctx.get("pv:sa", 2.0).expect("get string array");
    assert_eq!(got.get_field_string_array("value").expect("value"), new_val);

    server.stop_drop().expect("server stop");
}

#[test]
fn loopback_get_put_enum_roundtrip() {
    let _guard = lock_env();

    let server = Server::start_isolated().expect("server start");
    server
        .create_pv_enum(
            "pv:enum",
            vec!["OFF", "ON", "TRIP"],
            0,
            NTEnumMetadataBuilder::new(),
        )
        .expect("create pv");

    let mut ctx = setup_client_for(&server);

    ctx.put_enum("pv:enum", 2, 2.0).expect("put enum");
    let got = ctx.get("pv:enum", 2.0).expect("get enum");
    assert_eq!(got.get_field_enum("value").expect("value"), 2);

    server.stop_drop().expect("server stop");
}

#[test]
fn monitor_receives_initial_sample_on_start() {
    let _guard = lock_env();

    let server = Server::start_isolated().expect("server start");
    server
        .create_pv_double("pv:mon", 9.5, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    let mut mon = ctx.monitor("pv:mon").expect("monitor create");
    mon.start().expect("monitor start");

    let sample = mon.get_update(3.0).expect("initial monitor sample");
    assert!((sample.get_field_double("value").expect("value") - 9.5).abs() < 1e-12);

    mon.stop().expect("monitor stop");
    server.stop_drop().expect("server stop");
}

#[test]
fn concurrent_monitors_receive_initial_sample() {
    let _guard = lock_env();

    let server = Server::start_isolated().expect("server start");
    server
        .create_pv_double("pv:mon:concurrent", 9.5, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    let mut monitors = Vec::new();
    for _ in 0..16 {
        let mut monitor = ctx
            .monitor("pv:mon:concurrent")
            .expect("monitor create");
        monitor.start().expect("monitor start");
        monitors.push(monitor);
    }

    for monitor in &mut monitors {
        let sample = monitor
            .get_update(3.0)
            .expect("concurrent monitor initial sample");
        assert!((sample.get_field_double("value").expect("value") - 9.5).abs() < 1e-12);
    }

    for monitor in &mut monitors {
        monitor.stop().expect("monitor stop");
    }
    server.stop_drop().expect("server stop");
}

#[test]
fn get_missing_pv_returns_error() {
    let _guard = lock_env();

    let server = Server::start_isolated().expect("server start");
    let mut ctx = setup_client_for(&server);

    let err = ctx
        .get("pv:does-not-exist", 1.5)
        .expect_err("missing PV should error");
    assert!(
        err.to_string().to_ascii_lowercase().contains("not")
            || err.to_string().to_ascii_lowercase().contains("reject")
            || err.to_string().to_ascii_lowercase().contains("udp recv")
            || err.to_string().to_ascii_lowercase().contains("forcibly closed"),
        "unexpected error text: {}",
        err
    );

    server.stop_drop().expect("server stop");
}

#[test]
fn put_wrong_type_returns_error() {
    let _guard = lock_env();

    let server = Server::start_isolated().expect("server start");
    server
        .create_pv_string("pv:str", "hello", NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);

    let err = ctx
        .put_double("pv:str", 1.0, 2.0)
        .expect_err("wrong-type PUT should error");
    assert!(
        err.to_string().to_ascii_lowercase().contains("not")
            || err.to_string().to_ascii_lowercase().contains("failed")
            || err.to_string().to_ascii_lowercase().contains("udp recv")
            || err.to_string().to_ascii_lowercase().contains("forcibly closed"),
        "unexpected error text: {}",
        err
    );

    server.stop_drop().expect("server stop");
}

#[test]
fn server_stop_breaks_future_requests() {
    let _guard = lock_env();

    let server = Server::start_isolated().expect("server start");
    let mut ctx = setup_client_for(&server);

    let udp = server.udp_port();
    server.stop_drop().expect("server stop");

    std::thread::sleep(Duration::from_millis(50));

    std::env::set_var("EPICS_PVA_ADDR_LIST", format!("127.0.0.1:{}", udp));
    let err = ctx
        .get("pv:any", 0.2)
        .expect_err("request after stop should fail");
    assert!(
        err.to_string().to_ascii_lowercase().contains("timeout")
            || err.to_string().to_ascii_lowercase().contains("connect")
            || err.to_string().to_ascii_lowercase().contains("not")
            || err.to_string().to_ascii_lowercase().contains("udp recv")
            || err.to_string().to_ascii_lowercase().contains("forcibly closed"),
        "unexpected error text: {}",
        err
    );
}

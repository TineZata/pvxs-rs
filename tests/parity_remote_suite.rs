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
fn remote_string_encoding_roundtrip() {
    let _guard = lock_env();

    let server = Server::start_isolated().expect("server start");
    server
        .create_pv_string("parity:str", "", NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);

    let values = [
        "Simple ASCII",
        "Numbers: 1234567890",
        "Punctuation: !@#$%^&*()",
        "Unicode: αβγδ ελληνικά",
        "Chinese: 你好世界",
        "Emoji: 🚀🌟💡",
        "Mixed: Hello 世界 🌍!",
    ];

    for expected in values {
        ctx.put_string("parity:str", expected, 2.0)
            .expect("put string");
        let got = ctx.get("parity:str", 2.0).expect("get string");
        assert_eq!(got.get_field_string("value").expect("value"), expected);
    }

    server.stop_drop().expect("server stop");
}

#[test]
fn remote_double_precision_and_special_values() {
    let _guard = lock_env();

    let server = Server::start_isolated().expect("server start");
    server
        .create_pv_double("parity:double", 0.0, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);

    let precise = 1.23456789012345_f64;
    ctx.put_double("parity:double", precise, 2.0)
        .expect("put precise");
    let got = ctx.get("parity:double", 2.0).expect("get precise");
    assert!((got.get_field_double("value").expect("value") - precise).abs() < 1e-14);

    let special = [f64::INFINITY, f64::NEG_INFINITY, f64::MAX, f64::MIN, f64::MIN_POSITIVE];
    for expected in special {
        ctx.put_double("parity:double", expected, 2.0)
            .expect("put special");
        let got = ctx
            .get("parity:double", 2.0)
            .expect("get special")
            .get_field_double("value")
            .expect("value");
        if expected.is_infinite() {
            assert!(got.is_infinite());
            assert_eq!(got.is_sign_negative(), expected.is_sign_negative());
        } else {
            assert_eq!(got, expected);
        }
    }

    // NaN: only verify round-trip can be requested and value decodes.
    ctx.put_double("parity:double", f64::NAN, 2.0).expect("put NaN");
    let got = ctx.get("parity:double", 2.0).expect("get NaN");
    assert!(got.get_field_double("value").expect("value").is_nan());

    server.stop_drop().expect("server stop");
}

#[test]
fn remote_double_array_large_and_special_values() {
    let _guard = lock_env();

    let server = Server::start_isolated().expect("server start");
    server
        .create_pv_double_array("parity:da", vec![0.0], NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);

    let large: Vec<f64> = (0..256).map(|i| i as f64 * 0.25).collect();
    ctx.put_double_array("parity:da", large.clone(), 2.0)
        .expect("put large array");
    let got = ctx.get("parity:da", 2.0).expect("get large array");
    assert_eq!(got.get_field_double_array("value").expect("value"), large);

    let specials = vec![
        0.0,
        -0.0,
        f64::MIN,
        f64::MAX,
        f64::MIN_POSITIVE,
        1e-308,
        1e308,
        std::f64::consts::PI,
        std::f64::consts::E,
    ];
    ctx.put_double_array("parity:da", specials.clone(), 2.0)
        .expect("put specials array");
    let got = ctx.get("parity:da", 2.0).expect("get specials array");
    assert_eq!(
        got.get_field_double_array("value").expect("value").len(),
        specials.len()
    );

    ctx.put_double_array("parity:da", vec![], 2.0)
        .expect("put empty array");
    let got = ctx.get("parity:da", 2.0).expect("get empty array");
    assert!(got
        .get_field_double_array("value")
        .expect("value")
        .is_empty());

    server.stop_drop().expect("server stop");
}

#[test]
fn remote_enum_state_transitions_and_invalid_index() {
    let _guard = lock_env();

    let server = Server::start_isolated().expect("server start");
    server
        .create_pv_enum(
            "parity:enum",
            vec!["INIT", "READY", "ACTIVE", "PAUSED", "STOPPED"],
            0,
            NTEnumMetadataBuilder::new(),
        )
        .expect("create pv");

    let mut ctx = setup_client_for(&server);

    for idx in 0..5_i16 {
        ctx.put_enum("parity:enum", idx, 2.0).expect("put enum");
        let got = ctx.get("parity:enum", 2.0).expect("get enum");
        assert_eq!(got.get_field_enum("value").expect("value"), idx);
    }

    assert!(ctx.put_enum("parity:enum", 99, 2.0).is_err());
    assert!(ctx.put_enum("parity:enum", -1, 2.0).is_err());

    server.stop_drop().expect("server stop");
}

#[test]
fn multiple_clients_see_consistent_updates() {
    let _guard = lock_env();

    let server = Server::start_isolated().expect("server start");
    server
        .create_pv_int32("parity:i32", 7, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx_a = setup_client_for(&server);
    let mut ctx_b = setup_client_for(&server);

    ctx_a.put_int32("parity:i32", 42, 2.0).expect("put from A");
    let got_b = ctx_b.get("parity:i32", 2.0).expect("get from B");
    assert_eq!(got_b.get_field_int32("value").expect("value"), 42);

    ctx_b.put_int32("parity:i32", -5, 2.0).expect("put from B");
    let got_a = ctx_a.get("parity:i32", 2.0).expect("get from A");
    assert_eq!(got_a.get_field_int32("value").expect("value"), -5);

    server.stop_drop().expect("server stop");
}

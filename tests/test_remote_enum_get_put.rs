// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{Context, NTEnumMetadataBuilder, Server};
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
fn test_pv_remote_enum_get_put() {
    let _guard = lock_env();
    let timeout = 2.0;
    let choices = vec!["DISABLED", "ENABLED", "TESTING"];
    let initial_index = 0;
    let name = "remote:enum";

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_enum(name, choices.clone(), initial_index, NTEnumMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    let first = ctx.get(name, timeout).expect("initial get");
    assert_eq!(first.get_field_enum("value").expect("value"), initial_index);
    assert_eq!(
        first.get_field_enum("value.index").expect("value.index"),
        initial_index
    );
    assert_eq!(
        first
            .get_field_string_array("value.choices")
            .expect("value.choices"),
        choices.iter().map(|s| s.to_string()).collect::<Vec<_>>()
    );

    server.stop_drop().expect("stop server");
    assert!(ctx.get(name, 0.2).is_err());

    let server = Server::start_isolated().expect("restart server");
    server
        .create_pv_enum(name, choices.clone(), initial_index, NTEnumMetadataBuilder::new())
        .expect("recreate pv");

    let mut ctx = setup_client_for(&server);
    ctx.put_enum(name, 1, timeout).expect("put enum");
    let second = ctx.get(name, timeout).expect("second get");
    assert_eq!(second.get_field_enum("value").expect("value"), 1);
    assert_eq!(second.get_field_enum("value.index").expect("value.index"), 1);

    server.stop_drop().expect("stop server");
}

#[test]
fn test_pv_remote_enum_state_transitions() {
    let _guard = lock_env();
    let timeout = 2.0;
    let name = "remote:enum:states";
    let choices = vec!["INIT", "READY", "ACTIVE", "PAUSED", "STOPPED"];

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_enum(name, choices.clone(), 0, NTEnumMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);

    for (idx, _state) in choices.iter().enumerate() {
        ctx.put_enum(name, idx as i16, timeout).expect("put enum state");
        let value = ctx.get(name, timeout).expect("get enum state");
        assert_eq!(value.get_field_enum("value").expect("value") as usize, idx);
        assert_eq!(
            value.get_field_enum("value.index").expect("value.index") as usize,
            idx
        );
    }

    server.stop_drop().expect("stop server");
}

#[test]
fn test_pv_remote_enum_invalid_index() {
    let _guard = lock_env();
    let timeout = 2.0;
    let name = "remote:enum:invalid";

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_enum(
            name,
            vec!["OPTION_A", "OPTION_B", "OPTION_C"],
            0,
            NTEnumMetadataBuilder::new(),
        )
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    assert!(ctx.put_enum(name, 99, timeout).is_err());
    assert!(ctx.put_enum(name, -1, timeout).is_err());

    server.stop_drop().expect("stop server");
}

#[test]
fn test_pv_remote_enum_choices_immutable() {
    let _guard = lock_env();
    let timeout = 2.0;
    let name = "remote:enum:immutable";
    let choices = vec!["CHOICE_1", "CHOICE_2", "CHOICE_3", "CHOICE_4"];

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_enum(name, choices.clone(), 0, NTEnumMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    let initial = ctx
        .get(name, timeout)
        .expect("get initial")
        .get_field_string_array("value.choices")
        .expect("value.choices initial");

    for idx in 0..choices.len() {
        ctx.put_enum(name, idx as i16, timeout).expect("put enum index");
    }

    let final_choices = ctx
        .get(name, timeout)
        .expect("get final")
        .get_field_string_array("value.choices")
        .expect("value.choices final");

    assert_eq!(initial, final_choices);
    server.stop_drop().expect("stop server");
}
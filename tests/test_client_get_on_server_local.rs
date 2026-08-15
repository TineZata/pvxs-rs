// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{
    AlarmMetadata, AlarmSeverity, AlarmStatus, Context, ControlMetadata, NTEnumMetadataBuilder,
    NTScalarMetadataBuilder, Server,
};
use std::sync::{Mutex, MutexGuard, OnceLock};

fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    match LOCK.get_or_init(|| Mutex::new(())).lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn context_for(server: &Server) -> Context {
    std::env::set_var("EPICS_PVA_ADDR_LIST", format!("127.0.0.1:{}", server.udp_port()));
    std::env::set_var("EPICS_PVA_AUTO_ADDR_LIST", "NO");
    std::env::set_var("EPICS_PVA_BROADCAST_PORT", server.udp_port().to_string());
    Context::from_env().expect("context from env")
}

#[test]
fn client_get_int32_from_isolated_server() {
    let _guard = env_lock();
    let server = Server::start_isolated().expect("start server");
    let name = "test:local:int32";
    let metadata = NTScalarMetadataBuilder::new()
        .control(ControlMetadata {
            limit_low: 0.0,
            limit_high: 100.0,
            min_step: 1.0,
        })
        .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "OK")
        .alarm_metadata(
            AlarmMetadata::new()
                .active(true)
                .low_alarm_limit(10.0)
                .low_warning_limit(20.0)
                .high_warning_limit(80.0)
                .high_alarm_limit(90.0)
                .low_alarm_severity(AlarmSeverity::Major)
                .low_warning_severity(AlarmSeverity::Minor)
                .high_warning_severity(AlarmSeverity::Minor)
                .high_alarm_severity(AlarmSeverity::Major),
        );
    server
        .create_pv_int32(name, 42, metadata)
        .expect("create int32 pv");

    let fetched = server.fetch_int32(name).expect("fetch int32");
    assert_eq!(fetched.value, 42);
    assert_eq!(fetched.alarm_severity, AlarmSeverity::NoAlarm);

    let mut context = context_for(&server);
    let value = context.get(name, 2.0).expect("get int32");
    assert_eq!(value.get_field_int32("value").expect("value"), 42);
    server.stop_drop().expect("stop server");
}

#[test]
fn client_get_double_from_isolated_server() {
    let _guard = env_lock();
    let server = Server::start_isolated().expect("start server");
    let name = "test:local:double";
    server
        .create_pv_double(name, 3.25, NTScalarMetadataBuilder::new())
        .expect("create double pv");

    let mut context = context_for(&server);
    let value = context.get(name, 2.0).expect("get double");
    assert_eq!(value.get_field_double("value").expect("value"), 3.25);
    server.stop_drop().expect("stop server");
}

#[test]
fn client_get_enum_from_isolated_server() {
    let _guard = env_lock();
    let server = Server::start_isolated().expect("start server");
    let name = "test:local:enum";
    server
        .create_pv_enum(
            name,
            vec!["RED", "GREEN", "BLUE"],
            1,
            NTEnumMetadataBuilder::new(),
        )
        .expect("create enum pv");

    let mut context = context_for(&server);
    let value = context.get(name, 2.0).expect("get enum");
    assert_eq!(value.get_field_enum("value").expect("value"), 1);
    assert_eq!(
        value
            .get_field_string_array("value.choices")
            .expect("choices"),
        vec!["RED".to_string(), "GREEN".to_string(), "BLUE".to_string()]
    );
    server.stop_drop().expect("stop server");
}

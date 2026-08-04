// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{
    AlarmMetadata, AlarmSeverity, AlarmStatus, Context, ControlMetadata, NTScalarMetadataBuilder,
    Server,
};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread;
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

fn make_alarm_meta() -> AlarmMetadata {
    AlarmMetadata::new()
        .active(true)
        .low_alarm_limit(10.0)
        .low_warning_limit(20.0)
        .high_warning_limit(80.0)
        .high_alarm_limit(90.0)
        .low_alarm_severity(AlarmSeverity::Major)
        .low_warning_severity(AlarmSeverity::Minor)
        .high_warning_severity(AlarmSeverity::Minor)
        .high_alarm_severity(AlarmSeverity::Major)
        .hysteresis(0)
}

fn get_alarm_field_i32(value: &pvxs::Value, field: &str) -> Option<i32> {
    value.get_field_int32(field).ok()
}

#[test]
fn test_control_and_value_alarms_combined() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");
    let pv = "test:combined:control_value";

    let metadata = NTScalarMetadataBuilder::new()
        .control(ControlMetadata {
            limit_low: 0.0,
            limit_high: 100.0,
            min_step: 0.1,
        })
        .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "Ok")
        .alarm_metadata(make_alarm_meta());

    server.create_pv_double(pv, 50.0, metadata).expect("create pv");
    let mut ctx = setup_client_for(&server);

    assert!(server.post_double(pv, 150.0).is_err());
    thread::sleep(Duration::from_millis(50));
    let value = ctx.get(pv, 2.0).expect("get rejected value");
    assert_eq!(value.get_field_double("value").expect("value"), 50.0);
    if let Some(sev) = get_alarm_field_i32(&value, "alarm.severity") {
        assert_eq!(sev, AlarmSeverity::Invalid as i32);
    }
    if let Some(status) = get_alarm_field_i32(&value, "alarm.status") {
        assert_eq!(status, AlarmStatus::RecordStatus as i32);
    }

    server.post_double(pv, 85.0).expect("post high-warning");
    thread::sleep(Duration::from_millis(50));
    let value = ctx.get(pv, 2.0).expect("get warning value");
    assert_eq!(value.get_field_double("value").expect("value"), 85.0);
    if let Some(sev) = get_alarm_field_i32(&value, "alarm.severity") {
        assert_eq!(sev, AlarmSeverity::Minor as i32);
    }

    server.stop_drop().expect("stop server");
}

#[test]
fn test_alarm_transitions() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");
    let pv = "test:transitions";
    let metadata = NTScalarMetadataBuilder::new()
        .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "OK")
        .alarm_metadata(make_alarm_meta());
    server.create_pv_double(pv, 50.0, metadata).expect("create pv");

    let mut ctx = setup_client_for(&server);
    for (v, expected) in [
        (50.0, AlarmSeverity::NoAlarm),
        (85.0, AlarmSeverity::Minor),
        (95.0, AlarmSeverity::Major),
        (50.0, AlarmSeverity::NoAlarm),
        (15.0, AlarmSeverity::Minor),
        (5.0, AlarmSeverity::Major),
    ] {
        server.post_double(pv, v).expect("post transition value");
        thread::sleep(Duration::from_millis(40));
        if let Ok(got) = ctx
            .get(pv, 2.0)
            .expect("get transition value")
            .get_field_int32("alarm.severity")
        {
            assert_eq!(got, expected as i32);
        }
    }

    server.stop_drop().expect("stop server");
}

#[test]
fn test_multiple_pvs_with_different_alarms() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");

    let pv1 = "test:multi:pv1";
    let pv2 = "test:multi:pv2";
    let metadata1 = NTScalarMetadataBuilder::new()
        .control(ControlMetadata {
            limit_low: 0.0,
            limit_high: 100.0,
            min_step: 0.1,
        })
        .alarm_metadata(make_alarm_meta());
    let metadata2 = NTScalarMetadataBuilder::new().control(ControlMetadata {
        limit_low: -100.0,
        limit_high: 100.0,
        min_step: 1.0,
    });

    server.create_pv_double(pv1, 50.0, metadata1).expect("create pv1");
    server.create_pv_double(pv2, 1.0, metadata2).expect("create pv2");

    let mut ctx = setup_client_for(&server);
    server.post_double(pv1, 95.0).expect("post pv1");
    thread::sleep(Duration::from_millis(40));
    let v1 = ctx.get(pv1, 2.0).expect("get pv1");
    assert_eq!(v1.get_field_double("value").expect("value"), 95.0);
    if let Some(sev) = get_alarm_field_i32(&v1, "alarm.severity") {
        assert_eq!(sev, AlarmSeverity::Major as i32);
    }

    assert!(server.post_double(pv2, 150.0).is_err());
    thread::sleep(Duration::from_millis(40));
    let v2 = ctx.get(pv2, 2.0).expect("get pv2");
    assert_eq!(v2.get_field_double("value").expect("value"), 1.0);
    if let Some(sev) = get_alarm_field_i32(&v2, "alarm.severity") {
        assert_eq!(sev, AlarmSeverity::Invalid as i32);
    }

    server.stop_drop().expect("stop server");
}

#[test]
fn test_boundary_alarm_conditions() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");
    let pv = "test:boundary:alarms";
    let metadata = NTScalarMetadataBuilder::new()
        .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "OK")
        .alarm_metadata(make_alarm_meta());
    server.create_pv_double(pv, 50.0, metadata).expect("create pv");

    let mut ctx = setup_client_for(&server);
    for v in [10.0, 20.0, 15.0] {
        server.post_double(pv, v).expect("post boundary value");
        thread::sleep(Duration::from_millis(40));
        if let Ok(status) = ctx
            .get(pv, 2.0)
            .expect("get boundary value")
            .get_field_int32("alarm.status")
        {
            assert_eq!(status, AlarmStatus::DeviceStatus as i32);
        }
    }

    server.stop_drop().expect("stop server");
}
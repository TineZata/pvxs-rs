// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{
    AlarmMetadata, AlarmSeverity, AlarmStatus, Context, NTScalarMetadataBuilder, Server,
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

fn standard_alarm_meta(active: bool) -> AlarmMetadata {
    AlarmMetadata::new()
        .active(active)
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

fn maybe_alarm_i32(value: &pvxs::Value, field: &str) -> Option<i32> {
    value.get_field_int32(field).ok()
}

#[test]
fn test_high_alarm_limit() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");
    let pv = "test:alarm:high";
    let metadata = NTScalarMetadataBuilder::new()
        .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "")
        .alarm_metadata(standard_alarm_meta(true));

    server.create_pv_double(pv, 50.0, metadata).expect("create pv");
    let mut ctx = setup_client_for(&server);

    server.post_double(pv, 95.0).expect("post high alarm value");
    thread::sleep(Duration::from_millis(50));
    let value = ctx.get(pv, 2.0).expect("get");
    assert_eq!(value.get_field_double("value").expect("value"), 95.0);
    if let Some(severity) = maybe_alarm_i32(&value, "alarm.severity") {
        assert_eq!(severity, AlarmSeverity::Major as i32);
    }
    if let Some(status) = maybe_alarm_i32(&value, "alarm.status") {
        assert_eq!(status, AlarmStatus::DeviceStatus as i32);
    }

    server.stop_drop().expect("stop server");
}

#[test]
fn test_warning_and_low_alarm_limits() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");
    let pv = "test:alarm:warn_low";
    let metadata = NTScalarMetadataBuilder::new()
        .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "Ok")
        .alarm_metadata(standard_alarm_meta(true));

    server.create_pv_double(pv, 50.0, metadata).expect("create pv");
    let mut ctx = setup_client_for(&server);

    server.post_double(pv, 85.0).expect("post high warning value");
    thread::sleep(Duration::from_millis(40));
    let value = ctx.get(pv, 2.0).expect("get high warning");
    if let Some(severity) = maybe_alarm_i32(&value, "alarm.severity") {
        assert_eq!(severity, AlarmSeverity::Minor as i32);
    }

    server.post_double(pv, 15.0).expect("post low warning value");
    thread::sleep(Duration::from_millis(40));
    let value = ctx.get(pv, 2.0).expect("get low warning");
    if let Some(severity) = maybe_alarm_i32(&value, "alarm.severity") {
        assert_eq!(severity, AlarmSeverity::Minor as i32);
    }

    server.post_double(pv, 5.0).expect("post low alarm value");
    thread::sleep(Duration::from_millis(40));
    let value = ctx.get(pv, 2.0).expect("get low alarm");
    if let Some(severity) = maybe_alarm_i32(&value, "alarm.severity") {
        assert_eq!(severity, AlarmSeverity::Major as i32);
    }

    server.stop_drop().expect("stop server");
}

#[test]
fn test_no_alarm_within_normal_range() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");
    let pv = "test:alarm:normal";
    let metadata = NTScalarMetadataBuilder::new()
        .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "Ok")
        .alarm_metadata(standard_alarm_meta(true));

    server.create_pv_double(pv, 50.0, metadata).expect("create pv");
    let mut ctx = setup_client_for(&server);

    server.post_double(pv, 50.0).expect("post nominal");
    thread::sleep(Duration::from_millis(40));
    let value = ctx.get(pv, 2.0).expect("get nominal");
    if let Some(severity) = maybe_alarm_i32(&value, "alarm.severity") {
        assert_eq!(severity, AlarmSeverity::NoAlarm as i32);
    }
    if let Some(status) = maybe_alarm_i32(&value, "alarm.status") {
        assert_eq!(status, AlarmStatus::NoAlarm as i32);
    }

    server.stop_drop().expect("stop server");
}

#[test]
fn test_inactive_value_alarm() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");
    let pv = "test:alarm:inactive";
    let metadata = NTScalarMetadataBuilder::new()
        .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "Ok")
        .alarm_metadata(standard_alarm_meta(false));

    server.create_pv_double(pv, 50.0, metadata).expect("create pv");
    let mut ctx = setup_client_for(&server);

    server.post_double(pv, 95.0).expect("post potential alarm value");
    thread::sleep(Duration::from_millis(40));
    let value = ctx.get(pv, 2.0).expect("get");
    if let Some(severity) = maybe_alarm_i32(&value, "alarm.severity") {
        assert_eq!(severity, AlarmSeverity::NoAlarm as i32);
    }

    server.stop_drop().expect("stop server");
}

#[test]
fn test_alarm_severity_levels_int32() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");
    let pv = "test:alarm:int32:severity";
    let metadata = NTScalarMetadataBuilder::new()
        .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "Ok")
        .alarm_metadata(
            AlarmMetadata::new()
                .active(true)
                .low_alarm_limit(5.0)
                .low_warning_limit(10.0)
                .high_warning_limit(90.0)
                .high_alarm_limit(95.0)
                .low_alarm_severity(AlarmSeverity::Invalid)
                .low_warning_severity(AlarmSeverity::Minor)
                .high_warning_severity(AlarmSeverity::Minor)
                .high_alarm_severity(AlarmSeverity::Invalid)
                .hysteresis(0),
        );

    server.create_pv_int32(pv, 50, metadata).expect("create pv");
    let mut ctx = setup_client_for(&server);

    server.post_int32(pv, 100).expect("post high alarm value");
    thread::sleep(Duration::from_millis(40));
    let value = ctx.get(pv, 2.0).expect("get");
    if let Some(severity) = maybe_alarm_i32(&value, "alarm.severity") {
        assert_eq!(severity, AlarmSeverity::Invalid as i32);
    }

    server.stop_drop().expect("stop server");
}
// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{AlarmMetadata, AlarmSeverity, AlarmStatus, Context, NTScalarMetadataBuilder, Server};

fn alarm_metadata() -> AlarmMetadata {
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
}

#[test]
fn double_alarm_transitions_are_fetchable() {
    let server = Server::start_isolated().expect("start server");
    let name = "test:alarm:double";
    let metadata = NTScalarMetadataBuilder::new()
        .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "OK")
        .alarm_metadata(alarm_metadata());

    server
        .create_pv_double(name, 50.0, metadata)
        .expect("create double pv");

    let initial = server.fetch_double(name).expect("fetch initial value");
    assert_eq!(initial.alarm_severity, AlarmSeverity::NoAlarm);
    assert_eq!(initial.alarm_status, AlarmStatus::NoAlarm);

    server.post_double(name, 85.0).expect("post warning value");
    let warning = server.fetch_double(name).expect("fetch warning value");
    assert_eq!(warning.value, 85.0);
    assert_eq!(warning.alarm_severity, AlarmSeverity::Minor);
    assert_eq!(warning.alarm_status, AlarmStatus::DeviceStatus);
    assert_eq!(warning.alarm_message, "HIGH_WARNING");

    server.post_double(name, 95.0).expect("post alarm value");
    let alarm = server.fetch_double(name).expect("fetch alarm value");
    assert_eq!(alarm.alarm_severity, AlarmSeverity::Major);
    assert_eq!(alarm.alarm_message, "HIGH_ALARM");

    server.stop_drop().expect("stop server");
}

#[test]
fn alarmed_value_is_visible_to_remote_client() {
    let server = Server::start_isolated().expect("start server");
    let name = "test:alarm:remote";
    let metadata = NTScalarMetadataBuilder::new().alarm_metadata(alarm_metadata());
    server
        .create_pv_double(name, 50.0, metadata)
        .expect("create double pv");

    std::env::set_var(
        "EPICS_PVA_ADDR_LIST",
        format!("127.0.0.1:{}", server.udp_port()),
    );
    std::env::set_var("EPICS_PVA_AUTO_ADDR_LIST", "NO");
    std::env::set_var("EPICS_PVA_BROADCAST_PORT", server.udp_port().to_string());
    let mut context = Context::from_env().expect("create context");

    server.post_double(name, 95.0).expect("post alarm value");
    let value = context.get(name, 2.0).expect("get remote value");
    assert_eq!(value.get_field_double("value").expect("value"), 95.0);

    server.stop_drop().expect("stop server");
}

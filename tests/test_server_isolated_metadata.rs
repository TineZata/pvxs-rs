// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{
    AlarmMetadata, AlarmSeverity, AlarmStatus, ControlMetadata, DisplayMetadata,
    NTScalarMetadataBuilder, Server,
};

#[test]
fn scalar_metadata_is_preserved_and_enforced() {
    let server = Server::start_isolated().expect("start server");
    let name = "test:metadata:double";
    let display = DisplayMetadata {
        limit_low: 0,
        limit_high: 100,
        description: "Test PV with metadata".to_string(),
        units: "units".to_string(),
        precision: 2,
    };
    let control = ControlMetadata {
        limit_low: -10.0,
        limit_high: 110.0,
        min_step: 0.5,
    };
    let alarm = AlarmMetadata::new()
        .active(true)
        .low_warning_limit(20.0)
        .high_warning_limit(80.0)
        .low_alarm_limit(10.0)
        .high_alarm_limit(90.0)
        .low_warning_severity(AlarmSeverity::Minor)
        .high_warning_severity(AlarmSeverity::Minor)
        .low_alarm_severity(AlarmSeverity::Major)
        .high_alarm_severity(AlarmSeverity::Major);
    let metadata = NTScalarMetadataBuilder::new()
        .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "OK")
        .display(display.clone())
        .control(control.clone())
        .alarm_metadata(alarm.clone());

    server
        .create_pv_double(name, 50.0, metadata)
        .expect("create metadata pv");

    let fetched = server.fetch_double(name).expect("fetch metadata pv");
    let fetched_display = fetched.display_metadata.expect("display metadata");
    assert_eq!(fetched_display.units, display.units);
    assert_eq!(fetched_display.precision, display.precision);
    assert_eq!(
        fetched
            .control_metadata
            .expect("control metadata")
            .limit_high,
        control.limit_high
    );
    assert_eq!(
        fetched
            .alarm_metadata
            .expect("alarm metadata")
            .high_alarm_limit,
        alarm.high_alarm_limit
    );

    assert!(server.post_double(name, 120.0).is_err());
    assert_eq!(
        server
            .fetch_double(name)
            .expect("fetch retained value")
            .value,
        50.0
    );

    server.stop_drop().expect("stop server");
}

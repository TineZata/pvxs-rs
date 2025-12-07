mod test_server_isolated_metadate {
    use epics_pvxs_sys::{ControlMetadata, DisplayMetadata, NTScalarMetadataBuilder, ValueAlarmMetadata};


    #[test]
    fn test_create_pv_with_metadata() {
        use pvxs::server::Server;
        use pvxs::{NTScalarMetadata, NTScalarAlarm, NTScalarTime};

        let name = "TEST:PV:WITH:METADATA";
        let mut server = Server::new_isolated().expect("Failed to create server");
        let metadata = NTScalarMetadataBuilder::new()
            .alarm(0, 0, "")
            .display(DisplayMetadata {
                limit_low: Some(0.0),
                limit_high: Some(100.0),
                description: Some("Test PV with metadata".to_string()),
                units: Some("units".to_string()),
                precision: Some(2),
            })
            .control(ControlMetadata {
                limit_low: Some(-10.0),
                limit_high: Some(110.0),
            })
            .value_alarm(ValueAlarmMetadata {
                active: true,
                low_alarm_limit: Some(10.0),
                high_alarm_limit: Some(90.0),
                low_warning_limit: Some(20.0),
                high_warning_limit: Some(80.0),
                low_alarm_severity: Some(2),
                high_alarm_severity: Some(2),
                low_warning_severity: Some(1),
                high_warning_severity: Some(1),
                hysteresis: Some(0.5),
            })
            .with_form(true);
        
        let mut pv_with_metadata = server.add_double_pv_with_metadata(pv_name, initial_value, metadata);
    }
}
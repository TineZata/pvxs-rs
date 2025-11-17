mod test_server_alarm_serverity {

    #[test]
    fn test_double_with_alarm_isolated() {
        use pvxs::Server;
        use pvxs::types::{AlarmInfo, AlarmSeverity, AlarmStatus};
        
        let name = "TEST:PV:ALARM";
        let start_value = 0.0;

        // Create isolated server with a single PV
        let mut server = Server::new_isolated().unwrap();

        let mut pv_with_alarm = server.add_double_pv(name, start_value).unwrap();
        assert_eq!(pv_with_alarm.name(), name);

        // Initially, alarm severity should be NO_ALARM
        let initial_fetch = pv_with_alarm.fetch().unwrap();
        assert_eq!(initial_fetch.alarm_info(), AlarmInfo {severity: AlarmSeverity::None, status: AlarmStatus::None, message: "".to_string(),});

        // Post a new value with MINOR alarm severity
        pv_with_alarm.post_double_with_alarm(3.14, AlarmSeverity::Minor, AlarmStatus::Device, "Warning").unwrap();

        let fetch_minor = pv_with_alarm.fetch().unwrap();
        assert_eq!(fetch_minor.as_double().unwrap(), 3.14);
        let alarm_info = fetch_minor.alarm_info();
        assert_eq!(alarm_info.severity, AlarmSeverity::Minor);
        assert_eq!(alarm_info.status, AlarmStatus::Device);
        assert_eq!(alarm_info.message, "Warning");

        // Post a new value with MAJOR alarm severity
        pv_with_alarm.post_double_with_alarm(6.28,  AlarmSeverity::Major, AlarmStatus::Driver, "Driver depricated").unwrap();
        let fetch_major = pv_with_alarm.fetch().unwrap();
        let alarm_info_major = fetch_major.alarm_info();
        assert_eq!(fetch_major.as_double().unwrap(), 6.28);
        assert_eq!(alarm_info_major.severity, AlarmSeverity::Major);
        assert_eq!(alarm_info_major.status, AlarmStatus::Driver);
        assert_eq!(alarm_info_major.message, "Driver depricated");

        // Post a new value with NO_ALARM severity
        pv_with_alarm.post_double_with_alarm(0.0, AlarmSeverity::None, AlarmStatus::None, "").unwrap();
        let fetch_no_alarm = pv_with_alarm.fetch().unwrap();
        assert_eq!(fetch_no_alarm.as_double().unwrap(), 0.0);
        let alarm_info_none = fetch_no_alarm.alarm_info();
        assert_eq!(alarm_info_none.severity, AlarmSeverity::None);
        assert_eq!(alarm_info_none.status, AlarmStatus::None);
        assert_eq!(alarm_info_none.message, "");

    }

    #[test]
    fn test_int_with_alarm_isolated() {
        use pvxs::Server;
        use pvxs::types::{AlarmInfo, AlarmSeverity, AlarmStatus};
        
        let name = "TEST:PV:INT:ALARM";
        let start_value = 10;

        // Create isolated server with a single PV
        let mut server = Server::new_isolated().unwrap();

        let mut pv_with_alarm = server.add_int32_pv(name, start_value).unwrap();
        assert_eq!(pv_with_alarm.name(), name);

        // Initially, alarm severity should be NO_ALARM
        let initial_fetch = pv_with_alarm.fetch().unwrap();
        assert_eq!(initial_fetch.alarm_info(), AlarmInfo {severity: AlarmSeverity::None, status: AlarmStatus::None, message: "".to_string(),});

        // Post a new value with MINOR alarm severity
        pv_with_alarm.post_int32_with_alarm(20, AlarmSeverity::Minor, AlarmStatus::Device, "Minor issue").unwrap();

        let fetch_minor = pv_with_alarm.fetch().unwrap();
        assert_eq!(fetch_minor.as_int().unwrap(), 20);
        let alarm_info = fetch_minor.alarm_info();
        assert_eq!(alarm_info.severity, AlarmSeverity::Minor);
        assert_eq!(alarm_info.status, AlarmStatus::Device);
        assert_eq!(alarm_info.message, "Minor issue");
    }

    #[test]
    fn test_string_with_alarm_isolated() {
        use pvxs::Server;
        use pvxs::types::{AlarmInfo, AlarmSeverity, AlarmStatus};
        
        let name = "TEST:PV:STRING:ALARM";
        let start_value = "OK";

        // Create isolated server with a single PV
        let mut server = Server::new_isolated().unwrap();

        let mut pv_with_alarm = server.add_string_pv(name, start_value).unwrap();
        assert_eq!(pv_with_alarm.name(), name);

        // Initially, alarm severity should be NO_ALARM
        let initial_fetch = pv_with_alarm.fetch().unwrap();
        assert_eq!(initial_fetch.alarm_info(), AlarmInfo {severity: AlarmSeverity::None, status: AlarmStatus::None, message: "".to_string(),});

        // Post a new value with MAJOR alarm severity
        pv_with_alarm.post_string_with_alarm("ERROR", AlarmSeverity::Major, AlarmStatus::Driver, "System failure").unwrap();

        let fetch_major = pv_with_alarm.fetch().unwrap();
        assert_eq!(fetch_major.as_string().unwrap(), "ERROR");
        let alarm_info = fetch_major.alarm_info();
        assert_eq!(alarm_info.severity, AlarmSeverity::Major);
        assert_eq!(alarm_info.status, AlarmStatus::Driver);
        assert_eq!(alarm_info.message, "System failure");
    }

    #[test]
    fn test_enum_with_alarm_isolated() {
        use pvxs::Server;
        use pvxs::types::{AlarmInfo, AlarmSeverity, AlarmStatus};
        
        let name = "TEST:PV:ENUM:ALARM";
        let choices = vec!["RED", "GREEN", "BLUE"];
        let start_index = 0;

        // Create isolated server with a single PV
        let mut server = Server::new_isolated().unwrap();

        let mut pv_with_alarm = server.add_enum_pv(name, choices.clone(), start_index).unwrap();
        assert_eq!(pv_with_alarm.name(), name);

        // Initially, alarm severity should be NO_ALARM
        let initial_fetch = pv_with_alarm.fetch().unwrap();
        assert_eq!(initial_fetch.alarm_info(), AlarmInfo {severity: AlarmSeverity::None, status: AlarmStatus::None, message: "".to_string(),});

        // Post a new value with MAJOR alarm severity
        pv_with_alarm.post_enum_with_alarm(2, AlarmSeverity::Major, AlarmStatus::Driver, "Color error").unwrap();

        let fetch_major = pv_with_alarm.fetch().unwrap();
        assert_eq!(fetch_major.as_enum_index().unwrap(), 2);
        let alarm_info = fetch_major.alarm_info();
        assert_eq!(alarm_info.severity, AlarmSeverity::Major);
        assert_eq!(alarm_info.status, AlarmStatus::Driver);
        assert_eq!(alarm_info.message, "Color error");
    }

    #[test]
    fn test_double_with_alarm_remote() {
        use pvxs::Server;
        use pvxs::client::Client;
        use pvxs::types::{AlarmInfo, AlarmSeverity, AlarmStatus};
        use std::time::Duration;

        let server_name = "TEST:PV:ALARM:REMOTE";
        let start_value = 1.0;

        // Create isolated server with a single PV
        let mut server = Server::new().unwrap();
        let mut pv_with_alarm = server.add_double_pv(server_name, start_value).unwrap();

        // Start the server in a background thread
        assert!(server.start().is_ok());

        // Give the server a moment to start
        std::thread::sleep(Duration::from_millis(500));

        // Create a client to connect to the server
        let mut client = Client::new().unwrap();

        // Initial get should have NO_ALARM
        let initial_value = client.get(server_name, 2.0).unwrap();
        assert_eq!(initial_value.as_double().unwrap(), start_value);
        let initial_alarm = initial_value.alarm_info();
        assert_eq!(initial_alarm, AlarmInfo {severity: AlarmSeverity::None, status: AlarmStatus::None, message: "".to_string(),});

        // Post a new value with MAJOR alarm severity from the server side
        pv_with_alarm.post_double_with_alarm(9.81, AlarmSeverity::Major, AlarmStatus::Driver, "Critical failure").unwrap();

        // Fetch the updated value from the client
        let updated_value = client.get(server_name, 2.0).unwrap();
        assert_eq!(updated_value.as_double().unwrap(), 9.81);
        let updated_alarm = updated_value.alarm_info();
        assert_eq!(updated_alarm.severity, AlarmSeverity::Major);
        assert_eq!(updated_alarm.status, AlarmStatus::Driver);
        assert_eq!(updated_alarm.message, "Critical failure");
    }

    #[test]
    fn test_int_with_alarm_remote() {
        use pvxs::Server;
        use pvxs::client::Client;
        use pvxs::types::{AlarmInfo, AlarmSeverity, AlarmStatus};
        use std::time::Duration;

        let server_name = "TEST:PV:INT:ALARM:REMOTE";
        let start_value = 42;

        // Create isolated server with a single PV
        let mut server = Server::new().unwrap();
        let mut pv_with_alarm = server.add_int32_pv(server_name, start_value).unwrap();

        // Start the server in a background thread
        assert!(server.start().is_ok());

        // Give the server a moment to start
        std::thread::sleep(Duration::from_millis(500));

        // Create a client to connect to the server
        let mut client = Client::new().unwrap();

        // Initial get should have NO_ALARM
        let initial_value = client.get(server_name, 2.0).unwrap();
        assert_eq!(initial_value.as_int().unwrap(), start_value);
        let initial_alarm = initial_value.alarm_info();
        assert_eq!(initial_alarm, AlarmInfo {severity: AlarmSeverity::None, status: AlarmStatus::None, message: "".to_string(),});

        // Post a new value with MINOR alarm severity from the server side
        pv_with_alarm.post_int32_with_alarm(100, AlarmSeverity::Minor, AlarmStatus::Device, "Minor issue").unwrap();

        // Fetch the updated value from the client
        let updated_value = client.get(server_name, 2.0).unwrap();
        assert_eq!(updated_value.as_int().unwrap(), 100);
        let updated_alarm = updated_value.alarm_info();
        assert_eq!(updated_alarm.severity, AlarmSeverity::Minor);
        assert_eq!(updated_alarm.status, AlarmStatus::Device);
        assert_eq!(updated_alarm.message, "Minor issue");
    }

    #[test]
    fn test_string_with_alarm_remote() {
        use pvxs::Server;
        use pvxs::client::Client;
        use pvxs::types::{AlarmInfo, AlarmSeverity, AlarmStatus};
        use std::time::Duration;

        let server_name = "TEST:PV:STRING:ALARM:REMOTE";
        let start_value = "INITIAL";

        // Create isolated server with a single PV
        let mut server = Server::new().unwrap();
        let mut pv_with_alarm = server.add_string_pv(server_name, start_value).unwrap();

        // Start the server in a background thread
        assert!(server.start().is_ok());

        // Give the server a moment to start
        std::thread::sleep(Duration::from_millis(500));

        // Create a client to connect to the server
        let mut client = Client::new().unwrap();

        // Initial get should have NO_ALARM
        let initial_value = client.get(server_name, 2.0).unwrap();
        assert_eq!(initial_value.as_string().unwrap(), start_value);
        let initial_alarm = initial_value.alarm_info();
        assert_eq!(initial_alarm, AlarmInfo {severity: AlarmSeverity::None, status: AlarmStatus::None, message: "".to_string(),});

        // Post a new value with MAJOR alarm severity from the server side
        pv_with_alarm.post_string_with_alarm("FAILURE", AlarmSeverity::Major, AlarmStatus::Driver, "System failure").unwrap();

        // Fetch the updated value from the client
        let updated_value = client.get(server_name, 2.0).unwrap();
        assert_eq!(updated_value.as_string().unwrap(), "FAILURE");
        let updated_alarm = updated_value.alarm_info();
        assert_eq!(updated_alarm.severity, AlarmSeverity::Major);
        assert_eq!(updated_alarm.status, AlarmStatus::Driver);
        assert_eq!(updated_alarm.message, "System failure");
    }

    #[test]
    fn test_enum_with_alarm_remote() {
        use pvxs::Server;
        use pvxs::client::Client;
        use pvxs::types::{AlarmInfo, AlarmSeverity, AlarmStatus};
        use std::time::Duration;

        let server_name = "TEST:PV:ENUM:ALARM:REMOTE";
        let choices = vec!["ONE", "TWO", "THREE"];
        let start_index = 1;

        // Create isolated server with a single PV
        let mut server = Server::new().unwrap();
        let mut pv_with_alarm = server.add_enum_pv(server_name, choices.clone(), start_index).unwrap();

        // Start the server in a background thread
        assert!(server.start().is_ok());

        // Give the server a moment to start
        std::thread::sleep(Duration::from_millis(500));

        // Create a client to connect to the server
        let mut client = Client::new().unwrap();

        // Initial get should have NO_ALARM
        let initial_value = client.get(server_name, 2.0).unwrap();
        assert_eq!(initial_value.as_enum_index().unwrap(), start_index);
        let initial_alarm = initial_value.alarm_info();
        assert_eq!(initial_alarm, AlarmInfo {severity: AlarmSeverity::None, status: AlarmStatus::None, message: "".to_string(),});

        // Post a new value with MAJOR alarm severity from the server side
        pv_with_alarm.post_enum_with_alarm(0, AlarmSeverity::Major, AlarmStatus::Driver, "Enum error").unwrap();

        // Fetch the updated value from the client
        let updated_value = client.get(server_name, 2.0).unwrap();
        assert_eq!(updated_value.as_enum_index().unwrap(), 0);
        let updated_alarm = updated_value.alarm_info();
        assert_eq!(updated_alarm.severity, AlarmSeverity::Major);
        assert_eq!(updated_alarm.status, AlarmStatus::Driver);
        assert_eq!(updated_alarm.message, "Enum error");
    }
}

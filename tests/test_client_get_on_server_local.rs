#[cfg(feature = "client")]
#[cfg(feature = "server")]
#[cfg(test)]

mod test_client_get_on_server_local {
    use std::thread;
    use pvxs::{Server, Client};

    #[test]
    fn test_client_get_int32_on_server_local() -> Result<(), Box<dyn std::error::Error>> {
    
        let name = "TEST:PV:LOCAL";
        let start_value = 42;
        // Create isolated server with a single PV
        let mut server = Server::new_isolated()?;

        let mut pv = server.add_int32_pv(name, start_value)?;
        // Start the server
        server.start()?;
        // Give server time to start
        thread::sleep(std::time::Duration::from_millis(100));

        // Server should be listening on a port
        assert!(server.tcp_port() > 0 || server.udp_port() > 0);

        // Check the data is the expected start value, doing a fetch
        let fetched_value = pv.fetch()?;
        let fetched_int = fetched_value.as_int()?;
        assert_eq!(fetched_int, start_value);

        // Instantiate a client
        let mut client = Client::new()?;
        // Client should timeout and not get a value
        match client.get(name, 3.0) {
            Ok(_) => assert!(false,"Client should not be able to get value from isolated server"),
            Err(_) => (), // Expected error
        }
        Ok(())
    }

    #[cfg(feature = "client")]
    #[cfg(feature = "server")]
    #[test]
    fn test_client_get_double_on_server_local() -> Result<(), Box<dyn std::error::Error>> {
        let name = "TEST:PV:LOCAL:DOUBLE";
        let start_value = 3.14;
        // Create isolated server with a single PV
        let mut server = Server::new_isolated()?;

        let mut pv = server.add_double_pv(name, start_value)?;
        // Start the server
        server.start()?;
        // Give server time to start
        thread::sleep(std::time::Duration::from_millis(100));

        // Server should be listening on a port
        assert!(server.tcp_port() > 0 || server.udp_port() > 0);

        // Check the data is the expected start value, doing a fetch
        let fetched_value = pv.fetch()?;
        let fetched_double = fetched_value.as_double()?;
        assert_eq!(fetched_double, start_value);

        // Instantiate a client
        let mut client = Client::new()?;
        // Client should timeout and not get a value
        match client.get(name, 3.0) {
            Ok(_) => assert!(false,"Client should not be able to get value from isolated server"),
            Err(_) => (), // Expected error
        }
        Ok(())
    }

    #[cfg(feature = "client")]
    #[cfg(feature = "server")]
    #[test]
    fn test_client_get_enum_on_server_local() -> Result<(), Box<dyn std::error::Error>> {

        let name = "TEST:PV:LOCAL:ENUM";
        let enum_choices = vec!["RED", "GREEN", "BLUE"];
        let start_value = 1; // GREEN
        // Create isolated server with a single PV
        let mut server = Server::new_isolated()?;

        let mut pv = server.add_enum_pv(name, enum_choices.clone(), start_value)?;
        // Start the server
        server.start()?;
        // Give server time to start
        thread::sleep(std::time::Duration::from_millis(100));

        // Server should be listening on a port
        assert!(server.tcp_port() > 0 || server.udp_port() > 0);

        // Check the data is the expected start value, doing a fetch
        let fetched_value = pv.fetch()?;
        let fetched_enum = fetched_value.as_enum_index()?;
        let fetched_choices = fetched_value.as_enum_choices()?;
        assert_eq!(fetched_enum, start_value);
        assert_eq!(fetched_choices, enum_choices);

        // Instantiate a client
        let mut client = Client::new()?;
        // Client should timeout and not get a value
        match client.get(name, 3.0) {
            Ok(_) => assert!(false,"Client should not be able to get value from isolated server"),
            Err(_) => (), // Expected error
        }
        Ok(())
    }
}


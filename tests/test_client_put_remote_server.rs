
mod test_client_put_remote_server {
    use std::thread;
    use pvxs::{Server, Client};

    #[cfg(feature = "client")]
    #[cfg(feature = "server")]
    #[test]
    fn test_client_put_int32_on_server_remote() -> Result<(), Box<dyn std::error::Error>> {
        let name = "TEST:PV:REMOTE";
        let start_value = 100;
        let put_value = 200;
        // Create isolated server with a single PV
        let mut server = Server::new_isolated()?;

        let mut pv = server.add_int32_pv(name, start_value)?;
        // Start the server
        server.start()?;
        // Give server time to start
        thread::sleep(std::time::Duration::from_millis(100));

        // Server should be listening on a port
        assert!(server.tcp_port() > 0 || server.udp_port() > 0);

        // Instantiate a client
        let mut client = Client::new()?;

        // Client puts a new value
        client.put(name, put_value, 3.0)?;

        // Fetch the value from the PV to verify the put worked
        let fetched_value = pv.fetch()?;
        let fetched_int = fetched_value.as_int()?;
        assert_eq!(fetched_int, put_value);

        Ok(())
    }
}
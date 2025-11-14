#[cfg(all(feature = "server", feature = "client"))]
#[cfg(test)]
mod test_server_client_interaction {
    use pvxs::{Server, Client};
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_isolated_server_client_interaction() {
        // This test verifies that remote CLIENT operations do not work
        // for isolated servers
        
        let mut server = Server::new_isolated().unwrap();

        assert!(server.tcp_port() > 0, "Expected tcp_port to be non-zero for isolated server");
        assert!(server.udp_port() > 0, "Expected udp_port to be non-zero for isolated server");

        let pv_name = "test:client:needs:start";
        let test_value = 123.45;
        
        let mut pv = server.add_double_pv(pv_name, test_value).unwrap();
        
        // Server-side fetch works immediately (no start needed)
        let server_fetch = pv.fetch();
        assert!(server_fetch.is_ok(), "Server-side fetch should work without start()");
        
        // Create client that will try to connect to the server
        let mut client = Client::new().unwrap();
        
        thread::sleep(Duration::from_millis(200));
        
        // Try client get - this should not work without start()
        let client_result = client.get(pv_name, 1.0);
        assert!(client_result.is_err(), "Client GET should fail without server.start()");

        // Now start the server
        server.start().unwrap();
        thread::sleep(Duration::from_millis(200));

        // Try client get again - this should not work as well
        let client_result = client.get(pv_name, 1.0);
        assert!(client_result.is_err(), "Client GET should fail even after server.start() for isolated server");
        
        // Stop the server
        server.stop().unwrap();

        // Try client get again - this should fail after stop()
        let client_result = client.get(pv_name, 1.0);
        assert!(client_result.is_err(), "Client GET should fail after server.stop()");
    }

    #[test]
    fn test_remote_server_client_interaction() {
        // This test uses the standard EPICS ports (from environment)
        // so client discovery should work
        
        let mut server = Server::new().unwrap(); // Uses configured EPICS ports
        
        assert_eq!(server.tcp_port(), 5075, "Expected tcp_port to be 5075 from EPICS v7 configuration");
        assert_eq!(server.udp_port(), 5076, "Expected udp_port to be 5076 from EPICS v7 configuration");

        let pv_name = "test:remote:client:get";
        let test_value = 999.99;
        let mut pv = server.add_double_pv(pv_name, test_value).unwrap();
        
        // Server-side fetch works immediately
        let server_fetch = pv.fetch();
        assert!(server_fetch.is_ok(), "Server-side fetch should work without start()");
        
        // Create client
        let mut client = Client::new().unwrap();

        // Client tries to get the PV value via network WITHOUT server.start()
        assert!(client.get(pv_name, 1.0).is_err(), "Client GET should fail without server.start()");

        // Start server for client access
        server.start().unwrap();
        thread::sleep(Duration::from_millis(200));
        
        // Try to get the PV value via network
        let client_result = client.get(pv_name, 1.0);
        assert!(client_result.is_ok(), "Client GET should succeed after server.start()");
        assert_eq!(client_result.unwrap().as_double().unwrap(), test_value, "Client retrieved value should match test value");
        
        server.stop().unwrap();

        // Try client get again after stop - should fail
        let client_result = client.get(pv_name, 1.0);
        assert!(client_result.is_err(), "Client GET should fail after server.stop()");

    }
}

#[cfg(all(feature = "server", feature = "client"))]
#[cfg(test)]
mod test_server_start_necessity {
    use pvxs::{Server, Client};
    use std::time::Duration;
    use std::thread;

    // EPICS PVA default ports
    const EPICS_PVA_DEFAULT_TCP_PORT: u16 = 5075;
    const EPICS_PVA_DEFAULT_UDP_PORT: u16 = 5076;

    /// Check if server fell back to a random port (couldn't bind to configured port)
    fn is_port_fallback(server: &Server) -> bool {
        let tcp_port = server.tcp_port();
        let udp_port = server.udp_port();
        
        // If either port differs from the default, a fallback occurred
        tcp_port != EPICS_PVA_DEFAULT_TCP_PORT || udp_port != EPICS_PVA_DEFAULT_UDP_PORT
    }

    #[test]
    fn test_client_get_without_server_start() {
        // This test verifies what happens when we try client GET without calling start()
        
        let mut server = Server::new().unwrap();
        let pv_name = "test:no:start:client:get";
        let test_value = 777.77;
        
        let mut pv = server.add_double_pv(pv_name, test_value).unwrap();
        
        // Server gets ports from EPICS configuration (default 5075/5076)
        // but may fall back to random ports if those are in use
        let tcp_port = server.tcp_port();
        let udp_port = server.udp_port();
        
        if is_port_fallback(&server) {
            assert!(tcp_port > 0, "TCP port should be assigned even after fallback");
            assert!(udp_port > 0, "UDP port should be assigned even after fallback");
        } else {
            assert_eq!(tcp_port, EPICS_PVA_DEFAULT_TCP_PORT, "TCP port should be the configured EPICS default port");
            assert_eq!(udp_port, EPICS_PVA_DEFAULT_UDP_PORT, "UDP port should be the configured EPICS default port");
        }
        
        assert!(tcp_port > 0, "TCP port should be assigned");
        assert!(udp_port > 0, "UDP port should be assigned");
        
        // Server-side fetch works
        let server_fetch = pv.fetch();
        assert!(server_fetch.is_ok());
        
        // Give time for any background tasks
        thread::sleep(Duration::from_millis(200));
        
        // Try client get WITHOUT calling start()
        let mut client = Client::new().unwrap();
        
        let timeout = 2.0; // seconds
        let client_result = client.get(pv_name, timeout);
        assert!(client_result.is_err(), "Client GET should fail without server.start()");
        
        // Clean up
        server.stop().unwrap();
    }

    #[test]
    fn test_client_get_with_server_start() {
        // Control test: verify client GET works WITH start()
        
        let mut server = Server::new().unwrap();
        let pv_name = "test:with:start:client:get";
        let test_value = 888.88;
        
        let _pv = server.add_double_pv(pv_name, test_value).unwrap();
        
        // Check what port was actually assigned (may differ from config if port is in use)
        let tcp_port_before = server.tcp_port();
        let udp_port_before = server.udp_port();
        
        if is_port_fallback(&server) {
            assert!(tcp_port_before > 0, "TCP port should be assigned even after fallback");
            assert!(udp_port_before > 0, "UDP port should be assigned even after fallback");
        } else {
            assert_eq!(tcp_port_before, EPICS_PVA_DEFAULT_TCP_PORT, "TCP port should be the configured EPICS default port");
            assert_eq!(udp_port_before, EPICS_PVA_DEFAULT_UDP_PORT, "UDP port should be the configured EPICS default port");
        }
        
        assert!(tcp_port_before > 0, "TCP port should be assigned");
        assert!(udp_port_before > 0, "UDP port should be assigned");
        
        server.start().unwrap();
        
        // Port should remain the same after start()
        let tcp_port_after = server.tcp_port();
        let udp_port_after = server.udp_port();
        assert_eq!(tcp_port_before, tcp_port_after, "TCP port should not change after start()");
        assert_eq!(udp_port_before, udp_port_after, "UDP port should not change after start()");
        
        thread::sleep(Duration::from_millis(200));
        
        let mut client = Client::new().unwrap();
        
        let timeout = 2.0;
        let client_result = client.get(pv_name, timeout);
        
        match client_result {
            Ok(value) => {
                assert!((value.as_double().unwrap() - test_value).abs() < 1e-6, "Client GET returned unexpected value with server.start()");
            }
            Err(e) => {
                assert!(false, "Client GET FAILED even with server.start(): {:?}", e);
            }
        }
        
        server.stop().unwrap();
    }
}

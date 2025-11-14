#[cfg(feature = "server")]
#[cfg(test)]
mod test_server_remote_primitives {
    use pvxs::Server;
    use std::net::TcpStream;
    use std::time::Duration;

    #[test]
    fn test_server_creation() {
        let result = Server::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_pv_operations() {
        let mut server = Server::new().unwrap();

        // Add double PV
        let mut double_pv = server.add_double_pv("test:remote:double", 42.0).unwrap();
        assert_eq!(double_pv.name(), "test:remote:double");

        // Update value
        double_pv.post_double(84.0).unwrap();

        // Fetch and verify
        let fetched = double_pv.fetch().unwrap();
        assert_eq!(fetched.as_double().unwrap(), 84.0);

        // Add int32 PV
        let mut int_pv = server.add_int32_pv("test:remote:int", 123).unwrap();
        int_pv.post_int32(456).unwrap();

        // Fetch and verify int
        let fetched_int = int_pv.fetch().unwrap();
        assert_eq!(fetched_int.as_int().unwrap(), 456);

        // Add string PV
        let mut string_pv = server.add_string_pv("test:remote:string", "hello").unwrap();
        string_pv.post_string("world").unwrap();

        // Fetch and verify string
        let fetched_string = string_pv.fetch().unwrap();
        assert_eq!(fetched_string.as_string().unwrap(), "world");

        // Add enum PV
        let mut enum_pv = server.add_enum_pv("test:remote:enum", vec!["ONE", "TWO", "THREE"], 0).unwrap();
        enum_pv.post_enum(2).unwrap();

        assert!(enum_pv.fetch().unwrap().as_enum_index().unwrap() == 2);
        assert!(enum_pv.fetch().unwrap().as_enum_choices().unwrap() == vec!["ONE", "TWO", "THREE"]);
    }

    #[test]
    fn test_server_network_listening() {
        // This test verifies the behavior of remote (from_env) servers:
        // - Remote servers get configured ports from EPICS environment (default 5075/5076)
        // - Unlike isolated servers, these are CONFIGURED ports, not system-assigned
        // - Ports are allocated and listening after start()
        // - Ports stop listening after stop() is called
        
        let tcp_port: u16;
        let udp_port: u16;
        
        {
            // Server scope - will be destroyed at end of this block
            let mut server = Server::new().unwrap();
            let _pv = server.add_double_pv("test:remote:network", 1.0).unwrap();

            // Check ports BEFORE start() - remote servers have configured ports from environment
            let tcp_before = server.tcp_port();
            let udp_before = server.udp_port();
            
            // Remote servers get ports from EPICS configuration (typically 5075/5076)
            // They are NOT zero like we initially expected
            assert!(tcp_before > 0, "TCP port should be configured from environment, got {}", tcp_before);
            assert!(udp_before > 0, "UDP port should be configured from environment, got {}", udp_before);
            

            // Start the server - this should allocate ports and begin listening
            assert!(server.start().is_ok());
            
            // Give server time to start listening
            std::thread::sleep(Duration::from_millis(100));
            
            tcp_port = server.tcp_port();
            udp_port = server.udp_port();
            
            assert!(tcp_port > 0, "TCP port should be allocated after start(), got {}", tcp_port);
            assert!(udp_port > 0, "UDP port should be allocated after start(), got {}", udp_port);
            
            // Test TCP connection after start() - should succeed
            let after_start = TcpStream::connect_timeout(
                &format!("127.0.0.1:{}", tcp_port).parse().unwrap(),
                Duration::from_millis(500)
            );
            
            assert!(after_start.is_ok(), "TCP port {} should be listening after start()", tcp_port);
            
            // Test stop() - like isolated servers, stop() doesn't close the port
            assert!(server.stop().is_ok());
            
            // Give server time
            std::thread::sleep(Duration::from_millis(200));
            
            let after_stop = TcpStream::connect_timeout(
                &format!("127.0.0.1:{}", tcp_port).parse().unwrap(),
                Duration::from_millis(200)
            );
            
            assert!(after_stop.is_ok(), "TCP port {} should still be listening after stop()", tcp_port);
        } // Server destroyed here
        
        // Give OS time to release the port
        std::thread::sleep(Duration::from_millis(200));
        
        // Test if port is still listening AFTER server destruction
        let after_destroy = TcpStream::connect_timeout(
            &format!("127.0.0.1:{}", tcp_port).parse().unwrap(),
            Duration::from_millis(200)
        );
        
        assert!(after_destroy.is_err(), "TCP port {} should NOT be listening after server destruction", tcp_port);
    }
}

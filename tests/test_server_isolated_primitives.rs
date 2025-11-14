#[cfg(feature = "server")]
#[cfg(test)]
mod test_server_isolated_primitives {
    use pvxs::Server;
    use std::net::TcpStream;
    use std::time::Duration;

    #[test]
    fn test_server_creation() {
        let result = Server::new_isolated();
        assert!(result.is_ok());
    }

    #[test]
    fn test_pv_operations() {
        let mut server = Server::new_isolated().unwrap();

        assert!(server.tcp_port() > 0, "Expected tcp_port to be non-zero for isolated server");
        assert!(server.udp_port() > 0, "Expected udp_port to be non-zero for isolated server");
        
        // Add double PV
        let mut double_pv = server.add_double_pv("test:double", 42.0).unwrap();
        assert_eq!(double_pv.name(), "test:double");
        
        // Update value
        double_pv.post_double(84.0).unwrap();
        
        // Fetch and verify
        let fetched = double_pv.fetch().unwrap();
        let value = fetched.as_double().unwrap();
        assert!((value - 84.0).abs() < 1e-6);
        
        // Add int32 PV
        let mut int_pv = server.add_int32_pv("test:int", 123).unwrap();
        int_pv.post_int32(456).unwrap();
        
        // Fetch and verify int
        let fetched_int = int_pv.fetch().unwrap();
        assert_eq!(fetched_int.as_int().unwrap(), 456);
        
        // Add string PV
        let mut string_pv = server.add_string_pv("test:string", "hello").unwrap();
        string_pv.post_string("world").unwrap();
        
        // Fetch and verify string
        let fetched_string = string_pv.fetch().unwrap();
        assert_eq!(fetched_string.as_string().unwrap(), "world");

        // Add enum PV
        let mut enum_pv = server.add_enum_pv("test:enum", vec!["ONE", "TWO", "THREE"], 0).unwrap();
        enum_pv.post_enum(2).unwrap();
        assert!(enum_pv.fetch().unwrap().as_enum_index().unwrap() == 2);
        assert!(enum_pv.fetch().unwrap().as_enum_choices().unwrap() == vec!["ONE", "TWO", "THREE"]);
    }

    #[test]
    fn test_server_network_listening() {
        // This test verifies the actual behavior of isolated servers:
        // - Ports are allocated during create_isolated()
        // - Server is ALREADY listening on those ports (no explicit start needed)
        // - Ports are released when server object is destroyed
        
        let tcp_port: u16;
        let udp_port: u16;
        
        {
            // Server scope - will be destroyed at end of this block
            let mut server = Server::new_isolated().unwrap();
            let _pv = server.add_double_pv("test:network", 1.0).unwrap();

            tcp_port = server.tcp_port();
            udp_port = server.udp_port();
            
            // Ports should be allocated (non-zero)
            assert!(tcp_port > 0, "TCP port should be allocated, got {}", tcp_port);
            assert!(udp_port > 0, "UDP port should be allocated, got {}", udp_port);
            
            // Test TCP connection - isolated servers are already listening
            let connection = TcpStream::connect_timeout(
                &format!("127.0.0.1:{}", tcp_port).parse().unwrap(),
                Duration::from_millis(200)
            );
            
            assert!(connection.is_ok(), "TCP port {} should be listening", tcp_port);
            
            // Calling start() is redundant for isolated servers
            
            // Wait at least 2 seconds to ensure any delayed effects
            std::thread::sleep(Duration::from_millis(2000));

            // Verify still listening
            let after_start = TcpStream::connect_timeout(
                &format!("127.0.0.1:{}", tcp_port).parse().unwrap(),
                Duration::from_millis(200)
            );
            assert!(after_start.is_ok(), "TCP port {} should still be listening after 2s delay", tcp_port);
            
            // Test stop()
            server.stop().unwrap();
            std::thread::sleep(Duration::from_millis(50));
            
            let after_stop = TcpStream::connect_timeout(
                &format!("127.0.0.1:{}", tcp_port).parse().unwrap(),
                Duration::from_millis(200)
            );
            
            assert!(after_stop.is_ok(), "TCP port {} should still be listening after stop() for isolated server", tcp_port);
            
            // Remove pv and check if still listening
            let removed = server.remove_pv("test:network");
            assert!(removed.is_ok(), "Removing PV should succeed");

            let after_remove = TcpStream::connect_timeout(
                &format!("127.0.0.1:{}", tcp_port).parse().unwrap(),
                Duration::from_millis(200)
            );
            assert!(after_remove.is_ok(), "TCP port {} should still be listening after removing PV for isolated server", tcp_port);
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

    #[test]
    fn test_server_add_remove_pv() {
        let mut server = Server::new_isolated().unwrap();

        server.start().unwrap();
        
        // Add PV
        let mut pv1 = server.add_double_pv("test:addremove1", 0.0).unwrap();
        assert_eq!(pv1.name(), "test:addremove1");
        let mut pv2 = server.add_int32_pv("test:addremove2", 2).unwrap();
        assert_eq!(pv2.name(), "test:addremove2");
        
        // Remove PV pv 1 and check pv2 still exists
        let result = server.remove_pv("test:addremove1");
        assert!(result.is_ok());

        let fetched2 = pv2.fetch().unwrap();
        assert_eq!(fetched2.as_int().unwrap(), 2);
        
        // Attempt to fetch removed PV should fail
        let fetch_result = pv1.fetch();
        assert!(fetch_result.is_err());

        server.stop().unwrap();
    }
}
#[cfg(feature = "server")]
#[cfg(test)]
mod test_server_pv_tracking {
    use pvxs::Server;

    #[test]
    fn test_duplicate_pv_detection() {
        let mut server = Server::new_isolated().unwrap();
        
        // Add a PV
        let _pv1 = server.add_int32_pv("test:duplicate", 42).unwrap();
        
        // Try to add the same PV name again - should fail
        let result = server.add_int32_pv("test:duplicate", 99);
        
        assert!(result.is_err(), "Should not allow duplicate PV names");
        if let Err(e) = result {
            let msg = format!("{:?}", e);
            assert!(msg.contains("already exists"), "Error should mention PV already exists: {}", msg);
        }
    }

    #[test]
    fn test_remove_and_readd_pv() {
        let mut server = Server::new_isolated().unwrap();
        
        // Add a PV
        let _pv1 = server.add_double_pv("test:readd", 1.23).unwrap();
        
        // Remove it
        server.remove_pv("test:readd").unwrap();
        
        // Should be able to add it again
        let pv2 = server.add_double_pv("test:readd", 4.56);
        assert!(pv2.is_ok(), "Should allow adding PV after removal");
    }
}

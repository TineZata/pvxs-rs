use pvxs::{Server, Client};
use tokio_test::assert_err;
use std::thread;

#[test]
fn test_client_get_on_server_local() -> Result<(), Box<dyn std::error::Error>> {
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

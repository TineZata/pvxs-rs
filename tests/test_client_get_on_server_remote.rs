use pvxs::{Server, Client};
use tokio_test::assert_ok;
use std::thread;

#[cfg(feature = "client")]
#[cfg(feature = "server")]
#[test]
fn test_client_get_on_server_remote() -> Result<(), Box<dyn std::error::Error>> {
    let name = "TEST:PV:REMOTE";
    let start_value = 100;
    // Create server with a single PV
    let mut server = Server::new()?;

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
    // Client should be able to get the value from the server
    let client_value = assert_ok!(client.get(name, 3.0));
    let client_int = client_value.as_int()?;
    assert_eq!(client_int, start_value);

    Ok(())
}
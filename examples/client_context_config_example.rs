use pvxs::client::config::Config;

fn main() {
    let config: Config = pvxs::get_client_config();
    println!("Address list: {}", unsafe { config.address_list.to_rust_string_lossy() });
    println!("Interfaces: {}", unsafe { config.interfaces.to_rust_string_lossy() });
    println!("Name servers: {}", unsafe { config.name_servers.to_rust_string_lossy() });
    println!("UDP port: {}", config.udp_port);
    println!("TCP port: {}", config.tcp_port);
    println!("TCP timeout: {}", config.tcp_timeout);
    println!("Auto address list: {}", if config.auto_addr_list { "true" } else { "false" });
}

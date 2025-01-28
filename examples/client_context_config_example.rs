use pvxs::client::Config;

fn main() {
    let config: Config = pvxs::get_context_config();

    let addr = unsafe { config.address_list.to_rust_string() };
    println!("Address list: {}", addr);

    let interfaces = unsafe { config.interfaces.to_rust_string() };
    println!("Interfaces: {}", interfaces);

    let name_servers = unsafe { config.name_servers.to_rust_string() };
    println!("Name servers: {}", name_servers);

    println!("UDP port: {}", config.udp_port);
    println!("TCP port: {}", config.tcp_port);
    println!("TCP timeout: {}", config.tcp_timeout);
    println!("Auto address list: {}", if config.auto_addr_list { "true" } else { "false" });
}

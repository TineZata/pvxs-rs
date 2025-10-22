use pvxs::client::config::Config;

fn main() {
    let config: Config = pvxs::get_client_config();
    println!("Address list: {:?}", config.address_list.extract_strings());
    println!("Interfaces: {:?}", config.interfaces.extract_strings());
    println!("Name servers: {:?}", config.name_servers.extract_strings());
    println!("UDP port: {}", config.udp_port);
    println!("TCP port: {}", config.tcp_port);
    println!("TCP timeout: {}", config.tcp_timeout);
    println!("Auto address list: {}", if config.auto_addr_list { "true" } else { "false" });
}

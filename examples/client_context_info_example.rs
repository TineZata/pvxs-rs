use pvxs::client::{Context, Config};

fn main() {
    let ctx: Context = pvxs::get_context_from_env();
    if ctx._private._base._ptr.is_null() {
        println!("Failed to create context from environment");
        return;
    }
    println!("Context created from environment");
    //dbg!(ctx);

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
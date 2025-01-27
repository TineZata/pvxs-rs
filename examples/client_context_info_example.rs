use pvxs::client::{Context, Config};

fn main() {
    let ctx: Context = pvxs::get_context_from_env();
    if ctx._private._base._ptr.is_null() {
        println!("Failed to create context from environment");
        return;
    }
    println!("Context created from environment");
    dbg!(ctx);

    let config: Config = pvxs::get_context_config();
    if config.udp_port != 5076 {
        println!("UDP port should be default 5076");
        return;
    }
    if config.tcp_port != 5075 {
        println!("TCP port should be default 5075");
        return;
    }
    if config.tcp_timeout != 40.0 {
        println!("TCP timeout should be default 40.0s");
        return;
    }
    dbg!(config);
}
use std::collections::HashMap;

/// Configuration for a PVXS Client
#[derive(Debug, Clone)]
pub struct Config {
    /// List of unicast, multicast, and broadcast addresses to which search requests will be sent.
    /// Entries may take the forms:
    /// - `<ipaddr>[:<port#>]`
    /// - `<ipmultiaddr>[:<port>][,<ttl>][@<ifaceaddr>]`
    pub address_list: Vec<String>,

    /// List of local interface addresses on which beacons may be received.
    /// Also constrains `auto_address_list` to only consider broadcast addresses of listed interfaces.
    /// Empty implies wildcard `0.0.0.0`.
    pub interfaces: Vec<String>,

    /// List of TCP name servers.
    /// Client context will maintain connections, and send search requests, to these servers.
    pub name_servers: Vec<String>,

    /// UDP port to bind. Default is 5076. May be zero for dynamic allocation.
    pub udp_port: u16,

    /// Default TCP port for name servers.
    pub tcp_port: u16,

    /// Whether to extend the `address_list` with local interface broadcast addresses (recommended).
    pub auto_address_list: bool,

    /// Inactivity timeout interval for TCP connections (seconds).
    pub tcp_timeout: f64,
}

/*
impl Config {
    /// Creates a default configuration
    pub fn new() -> Self {
        Self {
            address_list: vec![],
            interfaces: vec![],
            name_servers: vec![],
            udp_port: 5076,
            tcp_port: 5075,
            auto_address_list: true,
            tcp_timeout: 40.0,
        }
    }

    /// Apply configuration from environment variables (`EPICS_PVA*`).
    /// This simulates environment-based initialization.
    pub fn from_env() -> Self {
        // Example of using environment variables; customize as needed
        let udp_port = std::env::var("EPICS_PVA_UDP_PORT")
            .unwrap_or_else(|_| "5076".to_string())
            .parse::<u16>()
            .unwrap_or(5076);

        let tcp_port = std::env::var("EPICS_PVA_TCP_PORT")
            .unwrap_or_else(|_| "5075".to_string())
            .parse::<u16>()
            .unwrap_or(5075);

        let auto_address_list = std::env::var("EPICS_PVA_AUTO_ADDR_LIST")
            .map(|val| val.eq_ignore_ascii_case("true"))
            .unwrap_or(true);

        let tcp_timeout = std::env::var("EPICS_PVA_TCP_TIMEOUT")
            .unwrap_or_else(|_| "40.0".to_string())
            .parse::<f64>()
            .unwrap_or(40.0);

        Self {
            udp_port,
            tcp_port,
            auto_address_list,
            tcp_timeout,
            ..Self::new()
        }
    }

    /// Apply definitions as with `EPICS_PVA*` environment variables
    pub fn apply_defs(mut self, defs: HashMap<String, String>) -> Self {
        if let Some(udp_port) = defs.get("EPICS_PVA_UDP_PORT") {
            self.udp_port = udp_port.parse::<u16>().unwrap_or(self.udp_port);
        }
        if let Some(tcp_port) = defs.get("EPICS_PVA_TCP_PORT") {
            self.tcp_port = tcp_port.parse::<u16>().unwrap_or(self.tcp_port);
        }
        if let Some(auto_address_list) = defs.get("EPICS_PVA_AUTO_ADDR_LIST") {
            self.auto_address_list = auto_address_list.eq_ignore_ascii_case("true");
        }
        if let Some(tcp_timeout) = defs.get("EPICS_PVA_TCP_TIMEOUT") {
            self.tcp_timeout = tcp_timeout.parse::<f64>().unwrap_or(self.tcp_timeout);
        }
        self
    }

    /// Update definitions based on the current configuration
    pub fn update_defs(&self, defs: &mut HashMap<String, String>) {
        defs.insert("EPICS_PVA_UDP_PORT".to_string(), self.udp_port.to_string());
        defs.insert("EPICS_PVA_TCP_PORT".to_string(), self.tcp_port.to_string());
        defs.insert("EPICS_PVA_AUTO_ADDR_LIST".to_string(), self.auto_address_list.to_string(),);
        defs.insert("EPICS_PVA_TCP_TIMEOUT".to_string(), self.tcp_timeout.to_string());
    }

    /// Create a new client Context using the current configuration
    pub fn build(self) -> Context {
        Context::new(self)
    }
}
    */

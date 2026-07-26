#[derive(Debug, Clone, Default)]
pub struct ClientConfig {
    /// Resolved from `EPICS_PVA_ADDR_LIST`
    pub(crate) addr_list: Vec<String>,
    /// Resolved from `EPICS_PVA_AUTO_ADDR_LIST` (default YES)
    pub(crate) auto_addr_list: bool,
    /// Resolved from `EPICS_PVA_BROADCAST_PORT` (default 5076)
    pub(crate) broadcast_port: u16,
}

impl ClientConfig {
    pub(crate) fn from_env() -> Self {
        let addr_list = std::env::var("EPICS_PVA_ADDR_LIST")
            .unwrap_or_default()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let auto_addr_list = std::env::var("EPICS_PVA_AUTO_ADDR_LIST")
            .map(|v| !v.eq_ignore_ascii_case("NO"))
            .unwrap_or(true);

        let broadcast_port = std::env::var("EPICS_PVA_BROADCAST_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5076);

        Self {
            addr_list,
            auto_addr_list,
            broadcast_port,
        }
    }
}
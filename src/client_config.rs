#[repr(C)]
#[derive(Debug)]
pub struct StdVector {
    pub _mypair: Vec<u8>,
}

#[repr(C)]
#[derive(Debug)]
pub struct ClientConfig {
    #[doc = " List of unicast, multicast, and broadcast addresses to which search requests will be sent.\n\n Entries may take the forms:\n - ``<ipaddr>[:<port#>]``\n - ``<ipmultiaddr>[:<port>][,<ttl>][@<ifaceaddr>]``"]
    pub address_list: StdVector,
    #[doc = "! List of local interface addresses on which beacons may be received.\n! Also constrains autoAddrList to only consider broadcast addresses of listed interfaces.\n! Empty implies wildcard 0.0.0.0"]
    pub interfaces: StdVector,
    #[doc = "! List of TCP name servers.\n! Client context will maintain connections, and send search requests, to these servers.\n! @since 0.2.0"]
    pub name_servers: StdVector,
    #[doc = "! UDP port to bind.  Default is 5076.  May be zero, cf. Server::config() to find allocated port."]
    pub udp_port: ::std::os::raw::c_ushort,
    #[doc = "! Default TCP port for name servers\n! @since 0.2.0"]
    pub tcp_port: ::std::os::raw::c_ushort,
    #[doc = "! Whether to extend the addressList with local interface broadcast addresses.  (recommended)"]
    pub auto_addr_list: bool,
    #[doc = "! Inactivity timeout interval for TCP connections.  (seconds)\n! @since 0.2.0"]
    pub tcp_timeout: f64,
    pub be: bool,
    pub udp: bool,
}

#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_client_Config"][::std::mem::size_of::<ClientConfig>() - 96usize];
    ["Alignment of pvxs_client_Config"][::std::mem::align_of::<ClientConfig>() - 8usize];
    ["Offset of field: pvxs_client_Config::addressList"]
        [::std::mem::offset_of!(ClientConfig, address_list) - 0usize];
    ["Offset of field: pvxs_client_Config::interfaces"]
        [::std::mem::offset_of!(ClientConfig, interfaces) - 24usize];
    ["Offset of field: pvxs_client_Config::nameServers"]
        [::std::mem::offset_of!(ClientConfig, name_servers) - 48usize];
    ["Offset of field: pvxs_client_Config::udp_port"]
        [::std::mem::offset_of!(ClientConfig, udp_port) - 72usize];
    ["Offset of field: pvxs_client_Config::tcp_port"]
        [::std::mem::offset_of!(ClientConfig, tcp_port) - 74usize];
    ["Offset of field: pvxs_client_Config::autoAddrList"]
        [::std::mem::offset_of!(ClientConfig, auto_addr_list) - 76usize];
    ["Offset of field: pvxs_client_Config::tcpTimeout"]
        [::std::mem::offset_of!(ClientConfig, tcp_timeout) - 80usize];
    ["Offset of field: pvxs_client_Config::BE"]
        [::std::mem::offset_of!(ClientConfig, be) - 88usize];
    ["Offset of field: pvxs_client_Config::UDP"]
        [::std::mem::offset_of!(ClientConfig, udp) - 89usize];
};

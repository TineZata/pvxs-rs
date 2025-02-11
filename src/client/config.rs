use crate::std_types::{StdMap, StdString, _OpaqueArray};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Config {
    /// List of unicast, multicast, and broadcast addresses to which search requests will be sent.
    /// 
    /// Entries may take the forms:
    /// - ``<ipaddr>[:<port#>]``
    /// - ``<ipmultiaddr>[:<port>][,<ttl>][@<ifaceaddr>]``
    pub address_list: _OpaqueArray<u64, 3usize>,
    /// List of local interface addresses on which beacons may be received.
    /// Also constrains autoAddrList to only consider broadcast addresses of listed interfaces.
    /// Empty implies wildcard 0.0.0.0
    pub interfaces: _OpaqueArray<u64, 3usize>,
    /// List of TCP name servers.
    /// Client context will maintain connections, and send search requests, to these servers.
    /// @since 0.2.0
    pub name_servers: _OpaqueArray<u64, 3usize>,
    /// UDP port to bind.  Default is 5076.  May be zero, cf. Server::config() to find allocated port.
    pub udp_port: ::std::os::raw::c_ushort,
    /// Default TCP port for name servers\n! @since 0.2.0
    pub tcp_port: ::std::os::raw::c_ushort,
    /// Whether to extend the addressList with local interface broadcast addresses.  (recommended)
    pub auto_addr_list: bool,
    /// Inactivity timeout interval for TCP connections.  (seconds)
    /// @since 0.2.0
    pub tcp_timeout: f64,
    pub be: bool,
    pub udp: bool,
}
pub type ConfigDefsT = StdMap;
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_client_Config"][::std::mem::size_of::<Config>() - 96usize];
    ["Alignment of pvxs_client_Config"][::std::mem::align_of::<Config>() - 8usize];
    ["Offset of field: pvxs_client_Config::addressList"]
        [::std::mem::offset_of!(Config, address_list) - 0usize];
    ["Offset of field: pvxs_client_Config::interfaces"]
        [::std::mem::offset_of!(Config, interfaces) - 24usize];
    ["Offset of field: pvxs_client_Config::nameServers"]
        [::std::mem::offset_of!(Config, name_servers) - 48usize];
    ["Offset of field: pvxs_client_Config::udp_port"]
        [::std::mem::offset_of!(Config, udp_port) - 72usize];
    ["Offset of field: pvxs_client_Config::tcp_port"]
        [::std::mem::offset_of!(Config, tcp_port) - 74usize];
    ["Offset of field: pvxs_client_Config::autoAddrList"]
        [::std::mem::offset_of!(Config, auto_addr_list) - 76usize];
    ["Offset of field: pvxs_client_Config::tcpTimeout"]
        [::std::mem::offset_of!(Config, tcp_timeout) - 80usize];
    ["Offset of field: pvxs_client_Config::BE"]
        [::std::mem::offset_of!(Config, be) - 88usize];
    ["Offset of field: pvxs_client_Config::UDP"]
        [::std::mem::offset_of!(Config, udp) - 89usize];
};
unsafe extern "C" {
    #[doc = "Update using defined EPICS_PVA* environment variables"]
    #[link_name = "\u{1}?applyEnv@Config@client@pvxs@@QEAAAEAU123@XZ"]
    pub fn pvxs_client_Config_applyEnv(this: *mut Config) -> *mut Config;
}
unsafe extern "C" {
    #[doc = "Update with definitions as with EPICS_PVA* environment variables\n! Process environment is not changed."]
    #[link_name = "\u{1}?applyDefs@Config@client@pvxs@@QEAAAEAU123@AEBV?$map@V?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@V12@U?$less@V?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@@2@V?$allocator@U?$pair@$$CBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@V12@@std@@@2@@std@@@Z"]
    pub fn pvxs_client_Config_applyDefs(
        this: *mut Config,
        defs: *const ConfigDefsT,
    ) -> *mut Config;
}
unsafe extern "C" {
    #[doc = "Extract definitions with environment variable names as keys.\n! Process environment is not changed."]
    #[link_name = "\u{1}?updateDefs@Config@client@pvxs@@QEBAXAEAV?$map@V?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@V12@U?$less@V?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@@2@V?$allocator@U?$pair@$$CBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@V12@@std@@@2@@std@@@Z"]
    pub fn pvxs_client_Config_updateDefs(
        this: *const Config,
        defs: *mut ConfigDefsT,
    );
}
unsafe extern "C" {
    #[doc = " Apply rules to translate current requested configuration\n  into one which can actually be loaded based on current host network configuration.\n\n  Explicit use of expand() is optional as the Context ctor expands any Config given.\n  expand() is provided as a aid to help understand how Context::effective() is arrived at.\n\n  @post autoAddrList==false"]
    #[link_name = "\u{1}?expand@Config@client@pvxs@@QEAAXXZ"]
    pub fn pvxs_client_Config_expand(this: *mut Config);
}
impl Config {
    #[inline]
    pub unsafe fn apply_env(&mut self) -> *mut Config {
        pvxs_client_Config_applyEnv(self)
    }
    #[inline]
    pub unsafe fn apply_defs(
        &mut self,
        defs: *const ConfigDefsT,
    ) -> *mut Config {
        pvxs_client_Config_applyDefs(self, defs)
    }
    #[inline]
    pub unsafe fn update_defs(&self, defs: *mut ConfigDefsT) {
        pvxs_client_Config_updateDefs(self, defs)
    }
    #[inline]
    pub unsafe fn expand(&mut self) {
        pvxs_client_Config_expand(self)
    }
}

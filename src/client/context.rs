use crate::{PvxsError, Result, Value};
use crate::client::{Monitor, MonitorBuilder, Rpc};
use crate::client::config::ClientConfig;

/// A pvAccess client context.
///
/// The context manages network connections and provides methods for GET, PUT,
/// Monitor, and RPC operations.  Thread-safe (`Send + Sync`).
///
/// # Network status
///
/// TODO: pvAccess TCP/UDP transport not yet implemented.
/// All network operations return an error until the transport layer is added.
pub struct Context {
    pub(crate) _config: ClientConfig,
    _rt: tokio::runtime::Runtime,
}

impl Context {
    /// Create a new Context configured from environment variables.
    ///
    /// Reads `EPICS_PVA_ADDR_LIST`, `EPICS_PVA_AUTO_ADDR_LIST`, and
    /// `EPICS_PVA_BROADCAST_PORT`.
    pub fn from_env() -> Result<Self> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PvxsError::new(format!("tokio runtime: {e}")))?;
        Ok(Self {
            _config: ClientConfig::from_env(),
            _rt: rt,
        })
    }

    // ── GET ─────────────────────────────────────────────────────────────────

    /// Perform a synchronous GET operation.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn get(&mut self, pv_name: &str, timeout: f64) -> Result<Value> {
        crate::net::blocking_get(&self._config, &self._rt, pv_name, timeout)
    }

    // ── PUT ─────────────────────────────────────────────────────────────────

    /// Perform a synchronous PUT with a double value.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn put_double(&mut self, pv_name: &str, value: f64, timeout: f64) -> Result<()> {
        crate::net::blocking_put(&self._config, &self._rt, pv_name, crate::net::PutValue::Double(value), timeout)
    }

    /// Perform a synchronous PUT with an int32 value.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn put_int32(&mut self, pv_name: &str, value: i32, timeout: f64) -> Result<()> {
        crate::net::blocking_put(&self._config, &self._rt, pv_name, crate::net::PutValue::Int32(value), timeout)
    }

    /// Perform a synchronous PUT with a string value.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn put_string(&mut self, pv_name: &str, value: &str, timeout: f64) -> Result<()> {
        crate::net::blocking_put(&self._config, &self._rt, pv_name, crate::net::PutValue::String(value), timeout)
    }

    /// Perform a synchronous PUT with an enum index (i16).
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn put_enum(&mut self, pv_name: &str, value: i16, timeout: f64) -> Result<()> {
        crate::net::blocking_put(&self._config, &self._rt, pv_name, crate::net::PutValue::Enum(value), timeout)
    }

    /// Perform a synchronous PUT with a double array.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn put_double_array(
        &mut self,
        pv_name: &str,
        value: Vec<f64>,
        timeout: f64,
    ) -> Result<()> {
        crate::net::blocking_put(
            &self._config,
            &self._rt,
            pv_name,
            crate::net::PutValue::DoubleArray(value),
            timeout,
        )
    }

    /// Perform a synchronous PUT with an int32 array.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn put_int32_array(
        &mut self,
        pv_name: &str,
        value: Vec<i32>,
        timeout: f64,
    ) -> Result<()> {
        crate::net::blocking_put(
            &self._config,
            &self._rt,
            pv_name,
            crate::net::PutValue::Int32Array(value),
            timeout,
        )
    }

    /// Perform a synchronous PUT with a string array.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn put_string_array(
        &mut self,
        pv_name: &str,
        value: Vec<String>,
        timeout: f64,
    ) -> Result<()> {
        crate::net::blocking_put(
            &self._config,
            &self._rt,
            pv_name,
            crate::net::PutValue::StringArray(value),
            timeout,
        )
    }

    // ── Monitor ──────────────────────────────────────────────────────────────

    /// Create a simple monitor subscription.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn monitor(&mut self, pv_name: &str) -> Result<Monitor> {
        Ok(Monitor::new(pv_name.to_string()))
    }

    /// Create a [`MonitorBuilder`] for advanced monitor configuration.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn monitor_builder(&mut self, pv_name: &str) -> Result<MonitorBuilder> {
        Ok(MonitorBuilder::new(pv_name.to_string()))
    }

    // ── RPC ──────────────────────────────────────────────────────────────────

    /// Create an RPC builder for a named service PV.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn rpc(&mut self, pv_name: &str) -> Result<Rpc> {
        Ok(Rpc::new(pv_name.to_string()))
    }
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

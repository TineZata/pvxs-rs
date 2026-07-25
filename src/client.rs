// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
//! Pure-Rust client for pvAccess.
//!
//! The public API mirrors `pvxs-sys::Context`, `Monitor`, `MonitorBuilder`
//! and `Rpc` exactly so callers can swap the crate without source changes.
//!
//! # Network status
//!
//! The in-process state machine (value conversion, alarm, metadata) is fully
//! implemented in pure Rust.  The TCP/UDP pvAccess transport layer is a
//! TODO — see TODO.md for the implementation plan.  All network-bound
//! methods return `Err(PvxsError)` until that layer lands.

use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::{PvxsError, Result, Value};

// ---------------------------------------------------------------------------
// MonitorEvent
// ---------------------------------------------------------------------------

/// Events that can be returned by [`Monitor::pop`].
#[derive(Debug, Clone, PartialEq)]
pub enum MonitorEvent {
    /// Connection event (maskConnected(false)).
    Connected(String),
    /// Disconnection event (maskDisconnected(false)).
    Disconnected(String),
    /// Subscription completed — no more events will arrive.
    Finished(String),
    /// Remote error from the server.
    RemoteError(String),
    /// Client-side error.
    ClientError(String),
}

impl fmt::Display for MonitorEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MonitorEvent::Connected(msg) => write!(f, "Monitor connected: {}", msg),
            MonitorEvent::Disconnected(msg) => write!(f, "Monitor disconnected: {}", msg),
            MonitorEvent::Finished(msg) => write!(f, "Monitor finished: {}", msg),
            MonitorEvent::RemoteError(msg) => write!(f, "Monitor remote error: {}", msg),
            MonitorEvent::ClientError(msg) => write!(f, "Monitor client error: {}", msg),
        }
    }
}

impl std::error::Error for MonitorEvent {}

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

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
    /// Configuration sourced from environment or explicit settings.
    _config: ClientConfig,
}

#[derive(Debug, Clone, Default)]
struct ClientConfig {
    /// Resolved from `EPICS_PVA_ADDR_LIST`
    addr_list: Vec<String>,
    /// Resolved from `EPICS_PVA_AUTO_ADDR_LIST` (default YES)
    auto_addr_list: bool,
    /// Resolved from `EPICS_PVA_BROADCAST_PORT` (default 5076)
    broadcast_port: u16,
}

impl ClientConfig {
    fn from_env() -> Self {
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

impl Context {
    /// Create a new Context configured from environment variables.
    ///
    /// Reads `EPICS_PVA_ADDR_LIST`, `EPICS_PVA_AUTO_ADDR_LIST`, and
    /// `EPICS_PVA_BROADCAST_PORT`.
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            _config: ClientConfig::from_env(),
        })
    }

    // ── GET ─────────────────────────────────────────────────────────────────

    /// Perform a synchronous GET operation.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn get(&mut self, _pv_name: &str, _timeout: f64) -> Result<Value> {
        Err(PvxsError::new(
            "pvAccess network transport not yet implemented — see TODO.md",
        ))
    }

    // ── PUT ─────────────────────────────────────────────────────────────────

    /// Perform a synchronous PUT with a double value.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn put_double(&mut self, _pv_name: &str, _value: f64, _timeout: f64) -> Result<()> {
        Err(PvxsError::new(
            "pvAccess network transport not yet implemented — see TODO.md",
        ))
    }

    /// Perform a synchronous PUT with an int32 value.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn put_int32(&mut self, _pv_name: &str, _value: i32, _timeout: f64) -> Result<()> {
        Err(PvxsError::new(
            "pvAccess network transport not yet implemented — see TODO.md",
        ))
    }

    /// Perform a synchronous PUT with a string value.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn put_string(&mut self, _pv_name: &str, _value: &str, _timeout: f64) -> Result<()> {
        Err(PvxsError::new(
            "pvAccess network transport not yet implemented — see TODO.md",
        ))
    }

    /// Perform a synchronous PUT with an enum index (i16).
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn put_enum(&mut self, _pv_name: &str, _value: i16, _timeout: f64) -> Result<()> {
        Err(PvxsError::new(
            "pvAccess network transport not yet implemented — see TODO.md",
        ))
    }

    /// Perform a synchronous PUT with a double array.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn put_double_array(
        &mut self,
        _pv_name: &str,
        _value: Vec<f64>,
        _timeout: f64,
    ) -> Result<()> {
        Err(PvxsError::new(
            "pvAccess network transport not yet implemented — see TODO.md",
        ))
    }

    /// Perform a synchronous PUT with an int32 array.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn put_int32_array(
        &mut self,
        _pv_name: &str,
        _value: Vec<i32>,
        _timeout: f64,
    ) -> Result<()> {
        Err(PvxsError::new(
            "pvAccess network transport not yet implemented — see TODO.md",
        ))
    }

    /// Perform a synchronous PUT with a string array.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn put_string_array(
        &mut self,
        _pv_name: &str,
        _value: Vec<String>,
        _timeout: f64,
    ) -> Result<()> {
        Err(PvxsError::new(
            "pvAccess network transport not yet implemented — see TODO.md",
        ))
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

// ---------------------------------------------------------------------------
// Monitor
// ---------------------------------------------------------------------------

/// Internal shared queue between the network driver and the consumer.
struct MonitorInner {
    name: String,
    running: bool,
    connected: bool,
    queue: VecDeque<Value>,
    connect_exception: bool,
    disconnect_exception: bool,
}

/// A subscription to value changes for a process variable.
///
/// Mirrors the `pvxs-sys::Monitor` API exactly.
///
/// TODO(network): pop() / try_get_update() will block/return None until the
/// pvAccess transport layer delivers real data.
pub struct Monitor {
    inner: Arc<Mutex<MonitorInner>>,
}

impl Monitor {
    fn new(name: String) -> Self {
        Self {
            inner: Arc::new(Mutex::new(MonitorInner {
                name,
                running: false,
                connected: false,
                queue: VecDeque::new(),
                connect_exception: false,
                disconnect_exception: true,
            })),
        }
    }

    /// Start monitoring.
    pub fn start(&mut self) -> Result<()> {
        // TODO(network): open a pvAccess subscription channel.
        self.inner.lock().unwrap().running = true;
        Ok(())
    }

    /// Stop monitoring.
    pub fn stop(&mut self) -> Result<()> {
        // TODO(network): close the subscription channel.
        self.inner.lock().unwrap().running = false;
        Ok(())
    }

    /// Returns `true` if monitoring is active.
    pub fn is_running(&self) -> bool {
        self.inner.lock().unwrap().running
    }

    /// Returns `true` if updates are available in the queue.
    pub fn has_update(&self) -> bool {
        !self.inner.lock().unwrap().queue.is_empty()
    }

    /// Returns `true` if connected to the remote PV.
    pub fn is_connected(&self) -> bool {
        self.inner.lock().unwrap().connected
    }

    /// The PV name being monitored.
    pub fn name(&self) -> String {
        self.inner.lock().unwrap().name.clone()
    }

    /// Get the next update, blocking until one arrives or the timeout elapses.
    ///
    /// TODO(network): will block forever until the transport layer pushes data.
    pub fn get_update(&mut self, timeout: f64) -> Result<Value> {
        let deadline = Instant::now() + Duration::from_secs_f64(timeout);
        loop {
            {
                let mut guard = self.inner.lock().unwrap();
                if let Some(v) = guard.queue.pop_front() {
                    return Ok(v);
                }
            }
            if Instant::now() >= deadline {
                return Err(PvxsError::new("monitor get_update timed out"));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Try to get the next update without blocking.
    pub fn try_get_update(&mut self) -> Result<Option<Value>> {
        Ok(self.inner.lock().unwrap().queue.pop_front())
    }

    /// Pop the next item from the subscription queue (PVXS-style).
    ///
    /// Returns:
    /// - `Ok(Some(Value))` — a new value is available.
    /// - `Ok(None)` — the queue is empty.
    /// - `Err(MonitorEvent::Connected)` — connection event (when `connect_exception` is set).
    /// - `Err(MonitorEvent::Disconnected)` — disconnection event.
    /// - `Err(MonitorEvent::Finished)` — subscription ended.
    pub fn pop(&mut self) -> std::result::Result<Option<Value>, MonitorEvent> {
        Ok(self.inner.lock().unwrap().queue.pop_front())
    }
}

// ---------------------------------------------------------------------------
// MonitorBuilder
// ---------------------------------------------------------------------------

/// Builder for creating monitors with advanced configuration.
///
/// Mirrors `pvxs-sys::MonitorBuilder` exactly.
pub struct MonitorBuilder {
    name: String,
    connect_exception: bool,
    disconnect_exception: bool,
}

impl MonitorBuilder {
    fn new(name: String) -> Self {
        Self {
            name,
            connect_exception: false,
            disconnect_exception: true,
        }
    }

    /// Enable or disable connection exceptions in the monitor queue.
    ///
    /// `true` = throw `MonitorEvent::Connected` on connect.
    /// `false` = suppress connection events (default).
    pub fn connect_exception(mut self, enable: bool) -> Self {
        self.connect_exception = enable;
        self
    }

    /// Enable or disable disconnection exceptions in the monitor queue.
    ///
    /// `true` = throw `MonitorEvent::Disconnected` on disconnect (default).
    /// `false` = suppress disconnection events.
    pub fn disconnect_exception(mut self, enable: bool) -> Self {
        self.disconnect_exception = enable;
        self
    }

    /// Finalise the builder and start the subscription.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn exec(self) -> Result<Monitor> {
        let mut m = Monitor::new(self.name);
        {
            let mut guard = m.inner.lock().unwrap();
            guard.connect_exception = self.connect_exception;
            guard.disconnect_exception = self.disconnect_exception;
        }
        Ok(m)
    }
}

// ---------------------------------------------------------------------------
// Rpc
// ---------------------------------------------------------------------------

/// RPC (Remote Procedure Call) builder.
///
/// Mirrors `pvxs-sys::Rpc` exactly.
///
/// TODO(network): pvAccess TCP transport not yet implemented.
pub struct Rpc {
    _name: String,
    _args: Value,
}

impl Rpc {
    fn new(name: String) -> Self {
        Self {
            _name: name,
            _args: Value::new(),
        }
    }

    /// Add a string argument.
    pub fn arg_string(&mut self, field: &str, value: &str) {
        self._args.set_field_string(field, value.to_string());
    }

    /// Add a double argument.
    pub fn arg_double(&mut self, field: &str, value: f64) {
        self._args.set_field_double(field, value);
    }

    /// Add an int32 argument.
    pub fn arg_int32(&mut self, field: &str, value: i32) {
        self._args.set_field_int32(field, value);
    }

    /// Add a boolean argument (stored as int32 0/1).
    pub fn arg_bool(&mut self, field: &str, value: bool) {
        self._args.set_field_int32(field, value as i32);
    }

    /// Execute the RPC synchronously.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn execute(&mut self, _timeout: f64) -> Result<Value> {
        Err(PvxsError::new(
            "pvAccess network transport not yet implemented — see TODO.md",
        ))
    }
}

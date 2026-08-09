// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use crate::client::config::ClientConfig;
use crate::client::{Monitor, MonitorBuilder, Rpc};
use crate::{PvxsError, Result, Value};
use std::sync::{Arc, Mutex};

pub(crate) struct RuntimeOwner {
    handle: tokio::runtime::Handle,
    shutdown_tx: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
    worker: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl RuntimeOwner {
    pub(crate) fn start() -> Result<Arc<Self>> {
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let worker = std::thread::Builder::new()
            .name("pvxs-client-runtime".to_string())
            .spawn(move || {
                let runtime = match tokio::runtime::Runtime::new() {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        let _ = ready_tx.send(Err(error.to_string()));
                        return;
                    }
                };

                if ready_tx.send(Ok(runtime.handle().clone())).is_err() {
                    return;
                }

                runtime.block_on(async {
                    let _ = shutdown_rx.await;
                });
            })
            .map_err(|error| PvxsError::new(format!("runtime thread: {error}")))?;

        let handle = ready_rx
            .recv()
            .map_err(|_| PvxsError::new("runtime thread stopped during startup"))?
            .map_err(|error| PvxsError::new(format!("tokio runtime: {error}")))?;

        Ok(Arc::new(Self {
            handle,
            shutdown_tx: Mutex::new(Some(shutdown_tx)),
            worker: Mutex::new(Some(worker)),
        }))
    }

    pub(crate) fn handle(&self) -> tokio::runtime::Handle {
        self.handle.clone()
    }
}

impl Drop for RuntimeOwner {
    fn drop(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.get_mut().unwrap().take() {
            let _ = shutdown_tx.send(());
        }

        if let Some(worker) = self.worker.get_mut().unwrap().take() {
            if worker.thread().id() == std::thread::current().id() {
                let _ = std::thread::Builder::new()
                    .name("pvxs-runtime-reaper".to_string())
                    .spawn(move || {
                        let _ = worker.join();
                    });
            } else {
                let _ = worker.join();
            }
        }
    }
}

/// A pvAccess client context.
///
/// The context manages network connections and provides methods for GET, PUT,
/// Monitor, and RPC operations.  Thread-safe (`Send + Sync`).
///
/// # Network status
///
/// UDP discovery and TCP GET, PUT, and Monitor operations are implemented.
/// RPC transport remains incomplete.
pub struct Context {
    pub(crate) _config: ClientConfig,
    runtime: Arc<RuntimeOwner>,
}

impl Context {
    /// Create a new Context configured from environment variables.
    ///
    /// Reads `EPICS_PVA_ADDR_LIST`, `EPICS_PVA_AUTO_ADDR_LIST`, and
    /// `EPICS_PVA_BROADCAST_PORT`.
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            _config: ClientConfig::from_env(),
            runtime: RuntimeOwner::start()?,
        })
    }

    // ── GET ─────────────────────────────────────────────────────────────────

    /// Perform a synchronous GET operation.
    pub fn get(&mut self, pv_name: &str, timeout: f64) -> Result<Value> {
        crate::net::blocking_get(&self._config, &self.runtime.handle(), pv_name, timeout)
    }

    /// Get type/field information about a process variable.
    ///
    /// In the pure-Rust backend this currently shares the same transport path
    /// as `get()` and returns the current value payload.
    pub fn info(&mut self, pv_name: &str, timeout: f64) -> Result<Value> {
        self.get(pv_name, timeout)
    }

    // ── PUT ─────────────────────────────────────────────────────────────────

    /// Perform a synchronous PUT with a double value.
    pub fn put_double(&mut self, pv_name: &str, value: f64, timeout: f64) -> Result<()> {
        crate::net::blocking_put(
            &self._config,
            &self.runtime.handle(),
            pv_name,
            crate::net::PutValue::Double(value),
            timeout,
        )
    }

    /// Perform a synchronous PUT with an int32 value.
    pub fn put_int32(&mut self, pv_name: &str, value: i32, timeout: f64) -> Result<()> {
        crate::net::blocking_put(
            &self._config,
            &self.runtime.handle(),
            pv_name,
            crate::net::PutValue::Int32(value),
            timeout,
        )
    }

    /// Perform a synchronous PUT with a string value.
    pub fn put_string(&mut self, pv_name: &str, value: &str, timeout: f64) -> Result<()> {
        crate::net::blocking_put(
            &self._config,
            &self.runtime.handle(),
            pv_name,
            crate::net::PutValue::String(value),
            timeout,
        )
    }

    /// Perform a synchronous PUT with an enum index (i16).
    pub fn put_enum(&mut self, pv_name: &str, value: i16, timeout: f64) -> Result<()> {
        crate::net::blocking_put(
            &self._config,
            &self.runtime.handle(),
            pv_name,
            crate::net::PutValue::Enum(value),
            timeout,
        )
    }

    /// Perform a synchronous PUT with a double array.
    pub fn put_double_array(&mut self, pv_name: &str, value: Vec<f64>, timeout: f64) -> Result<()> {
        crate::net::blocking_put(
            &self._config,
            &self.runtime.handle(),
            pv_name,
            crate::net::PutValue::DoubleArray(value),
            timeout,
        )
    }

    /// Perform a synchronous PUT with an int32 array.
    pub fn put_int32_array(&mut self, pv_name: &str, value: Vec<i32>, timeout: f64) -> Result<()> {
        crate::net::blocking_put(
            &self._config,
            &self.runtime.handle(),
            pv_name,
            crate::net::PutValue::Int32Array(value),
            timeout,
        )
    }

    /// Perform a synchronous PUT with a string array.
    pub fn put_string_array(
        &mut self,
        pv_name: &str,
        value: Vec<String>,
        timeout: f64,
    ) -> Result<()> {
        crate::net::blocking_put(
            &self._config,
            &self.runtime.handle(),
            pv_name,
            crate::net::PutValue::StringArray(value),
            timeout,
        )
    }

    // ── Monitor ──────────────────────────────────────────────────────────────

    /// Create a simple monitor subscription.
    pub fn monitor(&mut self, pv_name: &str) -> Result<Monitor> {
        Ok(Monitor::new(
            pv_name.to_string(),
            Arc::clone(&self.runtime),
            self._config.clone(),
        ))
    }

    /// Create a [`MonitorBuilder`] for advanced monitor configuration.
    pub fn monitor_builder(&mut self, pv_name: &str) -> Result<MonitorBuilder> {
        Ok(MonitorBuilder::new(
            pv_name.to_string(),
            Arc::clone(&self.runtime),
            self._config.clone(),
        ))
    }

    // ── RPC ──────────────────────────────────────────────────────────────────

    /// Create an RPC builder for a named service PV.
    ///
    /// RPC execution transport is not yet implemented.
    pub fn rpc(&mut self, pv_name: &str) -> Result<Rpc> {
        Ok(Rpc::new(pv_name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn context_can_drop_inside_async_runtime() {
        let context = Context::from_env().expect("context");
        drop(context);
    }

    #[tokio::test]
    async fn monitor_keeps_runtime_alive_after_context_drop() {
        let mut context = Context::from_env().expect("context");
        let monitor = context.monitor("test:runtime:lifetime").expect("monitor");

        drop(context);
        drop(monitor);
    }
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

/// Async parity surface with `pvxs-sys`.
#[cfg(feature = "async")]
impl Context {
    /// Asynchronously perform a GET operation.
    pub async fn get_async(&mut self, pv_name: &str, timeout: f64) -> Result<Value> {
        self.get(pv_name, timeout)
    }

    /// Asynchronously perform a double PUT operation.
    pub async fn put_double_async(
        &mut self,
        pv_name: &str,
        value: f64,
        timeout: f64,
    ) -> Result<()> {
        self.put_double(pv_name, value, timeout)
    }

    /// Asynchronously fetch field/type information.
    pub async fn info_async(&mut self, pv_name: &str, timeout: f64) -> Result<Value> {
        self.info(pv_name, timeout)
    }
}

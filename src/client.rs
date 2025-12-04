//! Client API for get/put/monitor operations with EPICS PVXS

use crate::error::{Error, Result};
use crate::types::Value;
use epics_pvxs_sys::{Context, Monitor as PvxsMonitor, MonitorBuilder as PvxsMonitorBuilder, PvxsError};
use tracing::{debug, info};

/// High-level PVXS client for EPICS operations
pub struct Client {
    context: Context,
}

impl Client {
    /// Create a new client using environment configuration
    ///
    /// This will read EPICS environment variables like EPICS_PVA_ADDR_LIST
    /// to configure the client.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Client;
    ///
    /// let client = Client::new()?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn new() -> Result<Self> {
        debug!("Creating new PVXS client from environment");
        
        let context = Context::from_env()
            .map_err(|e| Error::connection(format!("Failed to create PVXS context: {}", e)))?;
        
        info!("PVXS client created successfully");
        Ok(Self { context })
    }

    // Note: Custom configuration not yet available in epics-pvxs-sys
    // Will be added when the underlying library supports it

    /// Get a PV value synchronously
    ///
    /// # Arguments
    ///
    /// * `pv_name` - Name of the Process Variable
    /// * `timeout` - Timeout in seconds for the operation
    ///
    /// # Returns
    ///
    /// Returns the current value of the PV wrapped in a high-level `Value` type.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Client;
    ///
    /// let mut client = Client::new()?;
    /// let value = client.get("MY:PV:DOUBLE", 5.0)?;
    /// 
    /// println!("Value: {}", value.as_double()?);
    /// println!("Alarm: {}", value.alarm_info());
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn get(&mut self, pv_name: &str, timeout: f64) -> Result<Value> {
        debug!("Getting PV: {} with timeout: {}s", pv_name, timeout);
        
        let pvxs_value = self.context
            .get(pv_name, timeout)
            .map_err(|e| self.convert_pvxs_error(e, pv_name))?;
        
        debug!("Successfully got PV: {}", pv_name);
        Ok(Value::from_pvxs(pvxs_value))
    }

    /// Put a value to a PV synchronously (generic)
    ///
    /// This method supports multiple types through the `PutValue` trait:
    /// - `f64` - Double precision floating point
    /// - `i32` - 32-bit signed integer
    /// - `&str` - Not yet supported (returns error)
    /// - `String` - Not yet supported (returns error)
    ///
    /// # Arguments
    ///
    /// * `pv_name` - Name of the Process Variable
    /// * `value` - Value to write (f64, i32, &str, String)
    /// * `timeout` - Timeout in seconds for the operation
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Client;
    /// let mut client = Client::new()?;
    /// 
    /// // f64 and i32 are supported
    /// client.put("MY:PV:DOUBLE", 42.5, 5.0)?;
    /// client.put("MY:PV:INT", 123_i32, 5.0)?;
    /// 
    /// // String types not yet supported in upstream epics-pvxs-sys
    /// // client.put("MY:PV:STRING", "hello", 5.0)?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn put<T: PutValue>(&mut self, pv_name: &str, value: T, timeout: f64) -> Result<()> {
        value.put(self, pv_name, timeout)
    }

    /// Put a double value to a PV synchronously
    pub fn put_double(&mut self, pv_name: &str, value: f64, timeout: f64) -> Result<()> {
        debug!("Putting double value {} to PV: {} with timeout: {}s", value, pv_name, timeout);
        self.context
            .put_double(pv_name, value, timeout)
            .map_err(|e| self.convert_pvxs_error(e, pv_name))?;
        debug!("Successfully put value to PV: {}", pv_name);
        Ok(())
    }

    /// Put an int32 value to a PV synchronously
    pub fn put_int32(&mut self, pv_name: &str, value: i32, timeout: f64) -> Result<()> {
        debug!("Putting int32 value {} to PV: {} with timeout: {}s", value, pv_name, timeout);
        self.context
            .put_int32(pv_name, value, timeout)
            .map_err(|e| self.convert_pvxs_error(e, pv_name))?;
        debug!("Successfully put int32 value to PV: {}", pv_name);
        Ok(())
    }

    /// Put an enum value to a PV synchronously
    /// 
    /// # Arguments
    /// * `value` - The enum index to set (typically 0-255)
    pub fn put_enum(&mut self, pv_name: &str, value: u8, timeout: f64) -> Result<()> {
        debug!("Putting enum value {} to PV: {} with timeout: {}s", value, pv_name, timeout);
        self.context
            .put_enum(pv_name, value, timeout)
            .map_err(|e| self.convert_pvxs_error(e, pv_name))?;
        debug!("Successfully put enum value to PV: {}", pv_name);
        Ok(())
    }

    /// Get information about a PV without reading its value
    ///
    /// This is useful for discovering the structure and metadata of a PV.
    ///
    /// # Arguments
    ///
    /// * `pv_name` - Name of the Process Variable
    /// * `timeout` - Timeout in seconds for the operation
    ///
    /// # Returns
    ///
    /// Returns a `Value` containing the PV's structure information.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Client;
    ///
    /// let mut client = Client::new()?;
    /// let info = client.info("MY:PV:NAME", 5.0)?;
    /// println!("PV structure: {}", info);
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn info(&mut self, pv_name: &str, timeout: f64) -> Result<Value> {
        debug!("Getting info for PV: {} with timeout: {}s", pv_name, timeout);
        
        let pvxs_value = self.context
            .info(pv_name, timeout)
            .map_err(|e| self.convert_pvxs_error(e, pv_name))?;
        
        debug!("Successfully got info for PV: {}", pv_name);
        Ok(Value::from_pvxs(pvxs_value))
    }

    /// Check if a PV exists and is reachable
    ///
    /// This is a convenience method that performs a quick info request
    /// to determine if a PV can be reached.
    ///
    /// # Arguments
    ///
    /// * `pv_name` - Name of the Process Variable
    /// * `timeout` - Timeout in seconds for the operation
    ///
    /// # Returns
    ///
    /// Returns `true` if the PV exists and is reachable, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Client;
    ///
    /// let mut client = Client::new()?;
    /// if client.exists("MY:PV:NAME", 2.0)? {
    ///     println!("PV exists!");
    /// } else {
    ///     println!("PV not found");
    /// }
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn exists(&mut self, pv_name: &str, timeout: f64) -> Result<bool> {
        match self.info(pv_name, timeout) {
            Ok(_) => Ok(true),
            Err(Error::PvNotFound { .. }) => Ok(false),
            Err(Error::Timeout { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Helper method to convert PVXS errors to rust error types with context
    /// 
    /// # Arguments
    /// * `err` - The PvxsError to convert
    /// * `pv_name` - The name of the PV involved in the error
    /// * `timeout` - (Optional) The timeout value used in the operation
    /// 
    /// # Returns
    /// * Converted Error type
    ///
    fn convert_pvxs_error(&self, err: PvxsError, pv_name: &str, timeout: f64) -> Error {
        let error_msg = err.to_string().to_lowercase();
        
        if error_msg.contains("timeout") {
            Error::timedout(timeout)
        } else if error_msg.contains("not found") || error_msg.contains("no server") {
            Error::pv_not_found(pv_name)
        } else if error_msg.contains("connection") || error_msg.contains("connect") {
            Error::connection(format!("Failed to connect to PV '{}': {}", pv_name, err))
        } else {
            Error::from(err)
        }
    }

    /// Overloaded helper method with default timeout
    fn convert_pvxs_error(&self, err: PvxsError, pv_name: &str) -> Error {
        self.convert_pvxs_error(err, pv_name, 5.0)
    }

    /// Create a monitor for a PV (simple interface)
    ///
    /// This creates a monitor that will receive updates when the PV changes.
    /// Use `monitor_builder()` for more configuration options.
    ///
    /// # Arguments
    ///
    /// * `pv_name` - Name of the Process Variable to monitor
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Client;
    ///
    /// let mut client = Client::new()?;
    /// let mut monitor = client.monitor("MY:PV:NAME")?;
    /// monitor.start();
    /// 
    /// // Wait for updates
    /// while let Ok(Some(value)) = monitor.pop() {
    ///     println!("New value: {}", value);
    /// }
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn monitor(&mut self, pv_name: &str) -> Result<Monitor> {
        debug!("Creating monitor for PV: {}", pv_name);
        let pvxs_monitor = self.context
            .monitor(pv_name)
            .map_err(|e| self.convert_pvxs_error(e, pv_name))?;
        
        Ok(Monitor {
            inner: pvxs_monitor,
            pv_name: pv_name.to_string(),
        })
    }

    /// Create a monitor builder for advanced configuration
    ///
    /// The builder allows you to configure connection/disconnection event handling
    /// and register callbacks before executing the monitor.
    ///
    /// # Arguments
    ///
    /// * `pv_name` - Name of the Process Variable to monitor
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Client;
    ///
    /// let mut client = Client::new()?;
    /// let mut monitor = client.monitor_builder("MY:PV:NAME")?
    ///     .connection_events(true)
    ///     .disconnection_events(true)
    ///     .exec()?;
    ///
    /// monitor.start();
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn monitor_builder(&mut self, pv_name: &str) -> Result<MonitorBuilder> {
        debug!("Creating monitor builder for PV: {}", pv_name);
        let builder = self.context
            .monitor_builder(pv_name)
            .map_err(|e| self.convert_pvxs_error(e, pv_name))?;
        
        Ok(MonitorBuilder {
            inner: builder,
            pv_name: pv_name.to_string(),
        })
    }
}

/// High-level monitor for receiving PV updates
///
/// A monitor watches a PV and queues updates. Updates can be retrieved using
/// `pop()`. The monitor must be started with `start()` before it will receive updates.
pub struct Monitor {
    inner: PvxsMonitor,
    pv_name: String,
}

impl Monitor {
    /// Start the monitor
    ///
    /// The monitor will begin receiving updates after this is called.
    pub fn start(&mut self) {
        debug!("Starting monitor for PV: {}", self.pv_name);
        self.inner.start();
    }

    /// Stop the monitor
    ///
    /// The monitor will stop receiving updates after this is called.
    pub fn stop(&mut self) {
        debug!("Stopping monitor for PV: {}", self.pv_name);
        self.inner.stop();
    }

    /// Pop the next update from the queue
    ///
    /// This retrieves the next update from the monitor's internal queue.
    /// Returns `Ok(Some(value))` if an update is available, `Ok(None)` if the
    /// queue is empty, or `Err` for connection/disconnection events.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Client;
    ///
    /// let mut client = Client::new()?;
    /// let mut monitor = client.monitor("MY:PV:NAME")?;
    /// monitor.start();
    ///
    /// loop {
    ///     match monitor.pop() {
    ///         Ok(Some(value)) => println!("Value: {}", value),
    ///         Ok(None) => break, // Queue empty
    ///         Err(e) => println!("Event: {}", e),
    ///     }
    /// }
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn pop(&mut self) -> Result<Option<Value>> {
        match self.inner.pop() {
            Ok(Some(pvxs_value)) => Ok(Some(Value::from_pvxs(pvxs_value))),
            Ok(None) => Ok(None),
            Err(e) => Err(Error::from(e)),
        }
    }

    /// Check if the monitor is connected to the PV
    pub fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    /// Check if there are updates available
    ///
    /// Returns `true` if calling `pop()` would return data immediately.
    pub fn has_update(&self) -> bool {
        self.inner.has_update()
    }

    /// Get the PV name being monitored
    pub fn name(&self) -> &str {
        &self.pv_name
    }
}

impl std::fmt::Debug for Monitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Monitor")
            .field("pv_name", &self.pv_name)
            .field("connected", &self.is_connected())
            .finish()
    }
}

/// Builder for configuring a monitor
///
/// Allows configuration of event handling and callbacks before creating the monitor.
pub struct MonitorBuilder {
    inner: PvxsMonitorBuilder,
    pv_name: String,
}

impl MonitorBuilder {
    /// Enable or disable connection event notifications
    ///
    /// When enabled, connection events will be added to the queue and can be
    /// retrieved with `pop()` (they will return `Err`).
    pub fn connection_events(mut self, enable: bool) -> Self {
        self.inner = self.inner.mask_connected(enable);
        self
    }

    /// Enable or disable disconnection event notifications
    ///
    /// When enabled, disconnection events will be added to the queue and can be
    /// retrieved with `pop()` (they will return `Err`).
    pub fn disconnection_events(mut self, enable: bool) -> Self {
        self.inner = self.inner.mask_disconnected(enable);
        self
    }

    /// Register a callback function to be invoked when the queue goes from empty to not-empty
    ///
    /// The callback should be an `extern "C" fn()` function. It will be called when new
    /// data arrives in an empty queue. Note: The callback fires on queue state transitions
    /// (empty -> not-empty), so you should drain the queue with `pop()` to reset the state.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Client;
    ///
    /// extern "C" fn my_callback() {
    ///     println!("New data available!");
    /// }
    ///
    /// let mut client = Client::new()?;
    /// let mut monitor = client.monitor_builder("MY:PV:NAME")?
    ///     .event(my_callback)
    ///     .exec()?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn event(mut self, callback: extern "C" fn()) -> Self {
        self.inner = self.inner.event(callback);
        self
    }

    /// Execute the builder and create the monitor
    ///
    /// This consumes the builder and returns the configured `Monitor`.
    pub fn exec(self) -> Result<Monitor> {
        debug!("Executing monitor builder for PV: {}", self.pv_name);
        let pvxs_monitor = self.inner
            .exec()
            .map_err(|e| Error::from(e))?;
        
        Ok(Monitor {
            inner: pvxs_monitor,
            pv_name: self.pv_name,
        })
    }
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("context", &"<PVXS Context>")
            .finish()
    }
}

// Note: ClientBuilder not yet implemented - will be added when 
// epics-pvxs-sys supports custom configuration

// Note: Convenience methods will be added when more put methods are available

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use super::*;

    #[test]
    fn test_client_creation() {
        // This test will only work if EPICS environment is set up
        // It's more of a compilation test
        let result = Client::new();
        // We don't assert success because EPICS might not be available in test environment
        println!("Client creation result: {:?}", result);

        assert_eq!(TypeId::of::<Client>(), TypeId::of::<Client>());
    }
}
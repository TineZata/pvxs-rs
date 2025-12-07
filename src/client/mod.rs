//! Client API for get/put/monitor operations with EPICS PVXS

pub mod putvalue;
pub mod monitor;

// Re-export for convenience
pub use monitor::{Monitor, MonitorBuilder};
pub use putvalue::PutValue;

use crate::error::{Error, Result};
use crate::types::Value;
use epics_pvxs_sys::{Context, PvxsError};
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
            .map_err(|e| Error::pvxs(format!("Failed to create PVXS context: {}", e)))?;
        
        info!("PVXS client created successfully");
        Ok(Self { context })
    }

    // TODO: Add custom configuration... not yet available in epics-pvxs-sys

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
        
        let value = self.context.get(pv_name, timeout)
            .map_err(|e| self.convert_pvxs_error(e, pv_name, timeout))?;
        
        debug!("Successfully got PV: {}", pv_name);
        Ok(Value::from_pvxs(value))
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
        self.context.put_double(pv_name, value, timeout)
            .map_err(|e| self.convert_pvxs_error(e, pv_name, timeout))?;
        debug!("Successfully put value to PV: {}", pv_name);
        Ok(())
    }

    /// Put an int32 value to a PV synchronously
    pub fn put_int32(&mut self, pv_name: &str, value: i32, timeout: f64) -> Result<()> {
        debug!("Putting int32 value {} to PV: {} with timeout: {}s", value, pv_name, timeout);
        self.context
            .put_int32(pv_name, value, timeout)
            .map_err(|e| self.convert_pvxs_error(e, pv_name, timeout))?;
        debug!("Successfully put int32 value to PV: {}", pv_name);
        Ok(())
    }

    /// Put an enum value to a PV synchronously
    /// 
    /// # Arguments
    /// * `value` - The enum index to set (typically 0-255)
    pub fn put_enum(&mut self, pv_name: &str, value: i16, timeout: f64) -> Result<()> {
        debug!("Putting enum value {} to PV: {} with timeout: {}s", value, pv_name, timeout);
        self.context
            .put_enum(pv_name, value, timeout)
            .map_err(|e| self.convert_pvxs_error(e, pv_name, timeout))?;
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
            .map_err(|e| self.convert_pvxs_error(e, pv_name, timeout))?;
        
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
            Err(Error::Timedout { .. }) => Ok(false),
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
        // TODO: Create varification tests for each of these cases
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

    /// Create a monitor for a PV (simple interface)
    ///
    /// This creates a monitor that will receive updates when the PV changes.
    /// Use `monitor_builder()` for more configuration options.
    ///
    /// # Arguments
    ///
    /// * `pv_name` - Name of the Process Variable to monitor
    /// * `timeout` - Timeout in seconds for the operation
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Client;
    ///
    /// let mut client = Client::new()?;
    /// let mut monitor = client.monitor("MY:PV:NAME", 5.0)?;
    /// monitor.start();
    /// 
    /// // Wait for updates
    /// while let Ok(Some(value)) = monitor.pop() {
    ///     println!("New value: {}", value);
    /// }
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn monitor(&mut self, pv_name: &str, timeout: f64) -> Result<Monitor> {
        debug!("Creating monitor for PV: {}", pv_name);
        let pvxs_monitor = self.context
            .monitor(pv_name)
            .map_err(|e| self.convert_pvxs_error(e, pv_name, timeout))?;
        
        Ok(Monitor::new(pvxs_monitor, pv_name.to_string()))
    }

    /// Create a monitor builder for advanced configuration
    ///
    /// The builder allows you to configure connection/disconnection event handling
    /// and register callbacks before executing the monitor.
    ///
    /// # Arguments
    ///
    /// * `pv_name` - Name of the Process Variable to monitor
    /// * `timeout` - Timeout in seconds for the operation
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
    pub fn monitor_builder(&mut self, pv_name: &str, timeout: f64) -> Result<MonitorBuilder> {
        debug!("Creating monitor builder for PV: {}", pv_name);
        let builder = self.context
            .monitor_builder(pv_name)
            .map_err(|e| self.convert_pvxs_error(e, pv_name, timeout))?;
        
        Ok(MonitorBuilder::new(builder, pv_name.to_string()))
    }
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("context", &"<PVXS Context>")
            .finish()
    }
}

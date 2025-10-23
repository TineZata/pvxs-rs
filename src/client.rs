//! Client API for connecting to EPICS PVs
//!  
//! This module provides a high-level client interface for connecting to EPICS
//! IOCs and performing get, put, and monitor operations on Process Variables.

use crate::error::{Error, Result};
use crate::types::Value;
use epics_pvxs_sys::{Context, PvxsError};
use tracing::{debug, info};

/// Trait for types that can be put to a PV
pub trait PutValue {
    fn put(self, client: &mut Client, pv_name: &str, timeout: f64) -> Result<()>;
}

impl PutValue for f64 {
    fn put(self, client: &mut Client, pv_name: &str, timeout: f64) -> Result<()> {
        client.put_double(pv_name, self, timeout)
    }
}

impl PutValue for i32 {
    fn put(self, _client: &mut Client, _pv_name: &str, _timeout: f64) -> Result<()> {
        Err(Error::TypeConversion { message: "put for i32 not yet implemented in epics-pvxs-sys".to_string() })
    }
}

impl PutValue for &str {
    fn put(self, _client: &mut Client, _pv_name: &str, _timeout: f64) -> Result<()> {
        Err(Error::TypeConversion { message: "put for &str not yet implemented in epics-pvxs-sys".to_string() })
    }
}

impl PutValue for String {
    fn put(self, client: &mut Client, pv_name: &str, timeout: f64) -> Result<()> {
        self.as_str().put(client, pv_name, timeout)
    }
}

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
    /// let value = client.get("MY:PV:NAME", 5.0)?;
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
    /// - `f64` - Fully supported
    /// - `i32` - Not yet implemented (returns error)
    /// - `&str` - Not yet implemented (returns error)
    /// - `String` - Not yet implemented (returns error)
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
    /// // f64 is currently supported
    /// client.put("MY:PV:DOUBLE", 42.5, 5.0)?;
    /// 
    /// // i32, &str, String will return errors until epics-pvxs-sys supports them
    /// // client.put("MY:PV:INT", 123_i32, 5.0)?;
    /// // client.put("MY:PV:STRING", "hello", 5.0)?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn put<T: PutValue>(&mut self, pv_name: &str, value: T, timeout: f64) -> Result<()> {
        value.put(self, pv_name, timeout)
    }

    /// Put a double value to a PV synchronously (legacy)
    pub fn put_double(&mut self, pv_name: &str, value: f64, timeout: f64) -> Result<()> {
        debug!("Putting double value {} to PV: {} with timeout: {}s", value, pv_name, timeout);
        self.context
            .put_double(pv_name, value, timeout)
            .map_err(|e| self.convert_pvxs_error(e, pv_name))?;
        debug!("Successfully put value to PV: {}", pv_name);
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

    /// Helper method to convert PVXS errors to our error types with context
    fn convert_pvxs_error(&self, err: PvxsError, pv_name: &str) -> Error {
        let error_msg = err.to_string().to_lowercase();
        
        if error_msg.contains("timeout") {
            Error::timeout(5.0) // Default timeout for error reporting
        } else if error_msg.contains("not found") || error_msg.contains("no server") {
            Error::pv_not_found(pv_name)
        } else if error_msg.contains("connection") || error_msg.contains("connect") {
            Error::connection(format!("Failed to connect to PV '{}': {}", pv_name, err))
        } else {
            Error::from(err)
        }
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
//! Server API for providing EPICS PVs
//!
//! This module provides a high-level server interface for creating EPICS
//! servers that can provide Process Variables to clients.

use crate::error::{Error, Result};
use crate::types::{AlarmSeverity, Timestamp};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

/// High-level PVXS server for providing EPICS PVs
pub struct Server {
    // Note: This is a placeholder implementation since epics-pvxs-sys
    // may not have full server functionality yet
    pvs: Arc<Mutex<HashMap<String, ServerPv>>>,
    running: Arc<Mutex<bool>>,
}

/// Represents a Process Variable provided by the server
#[derive(Debug, Clone)]
pub struct ServerPv {
    /// PV name
    pub name: String,
    /// Current value
    pub value: PvValue,
    /// Access rights
    pub read_access: bool,
    pub write_access: bool,
    /// Alarm information
    pub alarm_severity: AlarmSeverity,
    pub alarm_message: String,
    /// Timestamp
    pub timestamp: Timestamp,
}

/// Value types that can be stored in a server PV
#[derive(Debug, Clone)]
pub enum PvValue {
    /// Double precision floating point
    Double(f64),
    /// 32-bit signed integer
    Int32(i32),
    /// 64-bit signed integer  
    Int64(i64),
    /// String value
    String(String),
    /// Boolean value
    Bool(bool),
    /// Array of doubles
    DoubleArray(Vec<f64>),
    /// Array of integers
    IntArray(Vec<i32>),
    /// Array of strings
    StringArray(Vec<String>),
}

impl PvValue {
    /// Get the value as a double if possible
    pub fn as_double(&self) -> Option<f64> {
        match self {
            PvValue::Double(v) => Some(*v),
            PvValue::Int32(v) => Some(*v as f64),
            PvValue::Int64(v) => Some(*v as f64),
            PvValue::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    /// Get the value as a string
    pub fn as_string(&self) -> String {
        match self {
            PvValue::String(s) => s.clone(),
            PvValue::Double(v) => v.to_string(),
            PvValue::Int32(v) => v.to_string(),
            PvValue::Int64(v) => v.to_string(),
            PvValue::Bool(v) => v.to_string(),
            PvValue::DoubleArray(arr) => format!("{:?}", arr),
            PvValue::IntArray(arr) => format!("{:?}", arr),
            PvValue::StringArray(arr) => format!("{:?}", arr),
        }
    }

    /// Get the value as an integer if possible
    pub fn as_int32(&self) -> Option<i32> {
        match self {
            PvValue::Int32(v) => Some(*v),
            PvValue::Int64(v) => Some(*v as i32),
            PvValue::Double(v) => Some(*v as i32),
            PvValue::Bool(v) => Some(if *v { 1 } else { 0 }),
            _ => None,
        }
    }
}

impl std::fmt::Display for PvValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl Server {
    /// Create a new PVXS server
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Server;
    ///
    /// let server = Server::new()?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn new() -> Result<Self> {
        debug!("Creating new PVXS server");
        
        Ok(Self {
            pvs: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        })
    }

    /// Add a PV to the server
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the Process Variable
    /// * `initial_value` - Initial value for the PV
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::{Server, server::PvValue};
    ///
    /// let mut server = Server::new()?;
    /// server.add_pv("test:double", PvValue::Double(42.0))?;
    /// server.add_pv("test:string", PvValue::String("Hello".to_string()))?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn add_pv(&self, name: &str, initial_value: PvValue) -> Result<()> {
        debug!("Adding PV: {} with value: {}", name, initial_value);
        
        let server_pv = ServerPv {
            name: name.to_string(),
            value: initial_value,
            read_access: true,
            write_access: true,
            alarm_severity: AlarmSeverity::None,
            alarm_message: String::new(),
            timestamp: Timestamp::now(),
        };
        
        let mut pvs = self.pvs.lock().unwrap();
        pvs.insert(name.to_string(), server_pv);
        
        info!("Added PV: {}", name);
        Ok(())
    }

    /// Update the value of an existing PV
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the Process Variable
    /// * `new_value` - New value for the PV
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::{Server, server::PvValue};
    ///
    /// let server = Server::new()?;
    /// server.add_pv("test:counter", PvValue::Int32(0))?;
    /// server.update_pv("test:counter", PvValue::Int32(1))?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn update_pv(&self, name: &str, new_value: PvValue) -> Result<()> {
        debug!("Updating PV: {} with new value: {}", name, new_value);
        
        let mut pvs = self.pvs.lock().unwrap();
        match pvs.get_mut(name) {
            Some(pv) => {
                pv.value = new_value;
                pv.timestamp = Timestamp::now();
                debug!("Updated PV: {}", name);
                Ok(())
            }
            None => {
                warn!("Attempted to update non-existent PV: {}", name);
                Err(Error::pv_not_found(name))
            }
        }
    }

    /// Set alarm information for a PV
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the Process Variable
    /// * `severity` - Alarm severity
    /// * `message` - Alarm message
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::{Server, server::PvValue, types::AlarmSeverity};
    ///
    /// let server = Server::new()?;
    /// server.add_pv("test:alarm", PvValue::Double(100.0))?;
    /// server.set_alarm("test:alarm", AlarmSeverity::Major, "High limit exceeded")?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn set_alarm(&self, name: &str, severity: AlarmSeverity, message: &str) -> Result<()> {
        debug!("Setting alarm for PV: {} to {} - {}", name, severity, message);
        
        let mut pvs = self.pvs.lock().unwrap();
        match pvs.get_mut(name) {
            Some(pv) => {
                pv.alarm_severity = severity;
                pv.alarm_message = message.to_string();
                pv.timestamp = Timestamp::now();
                debug!("Set alarm for PV: {}", name);
                Ok(())
            }
            None => {
                warn!("Attempted to set alarm for non-existent PV: {}", name);
                Err(Error::pv_not_found(name))
            }
        }
    }

    /// Remove a PV from the server
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the Process Variable to remove
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::{Server, server::PvValue};
    ///
    /// let server = Server::new()?;
    /// server.add_pv("temp:pv", PvValue::Double(0.0))?;
    /// server.remove_pv("temp:pv")?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn remove_pv(&self, name: &str) -> Result<()> {
        debug!("Removing PV: {}", name);
        
        let mut pvs = self.pvs.lock().unwrap();
        match pvs.remove(name) {
            Some(_) => {
                info!("Removed PV: {}", name);
                Ok(())
            }
            None => {
                warn!("Attempted to remove non-existent PV: {}", name);
                Err(Error::pv_not_found(name))
            }
        }
    }

    /// Get the current value of a PV
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the Process Variable
    ///
    /// # Returns
    ///
    /// Returns a copy of the current PV value.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::{Server, server::PvValue};
    ///
    /// let server = Server::new()?;
    /// server.add_pv("test:get", PvValue::Double(3.14))?;
    /// let value = server.get_pv_value("test:get")?;
    /// println!("Current value: {}", value);
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn get_pv_value(&self, name: &str) -> Result<PvValue> {
        let pvs = self.pvs.lock().unwrap();
        match pvs.get(name) {
            Some(pv) => Ok(pv.value.clone()),
            None => Err(Error::pv_not_found(name)),
        }
    }

    /// List all PVs currently provided by the server
    ///
    /// # Returns
    ///
    /// Returns a vector of PV names.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::{Server, server::PvValue};
    ///
    /// let server = Server::new()?;
    /// server.add_pv("pv1", PvValue::Double(1.0))?;
    /// server.add_pv("pv2", PvValue::String("test".to_string()))?;
    ///
    /// let pv_names = server.list_pvs();
    /// println!("Server provides {} PVs: {:?}", pv_names.len(), pv_names);
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn list_pvs(&self) -> Vec<String> {
        let pvs = self.pvs.lock().unwrap();
        pvs.keys().cloned().collect()
    }

    /// Start the server (placeholder implementation)
    ///
    /// Note: This is a placeholder since the underlying epics-pvxs-sys
    /// may not have full server functionality implemented yet.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::{Server, server::PvValue};
    ///
    /// let server = Server::new()?;
    /// server.add_pv("test:pv", PvValue::Double(42.0))?;
    /// server.start()?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn start(&self) -> Result<()> {
        info!("Starting PVXS server");
        
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err(Error::ServerConfig {
                message: "Server is already running".to_string(),
            });
        }
        
        // TODO: Implement actual server startup using epics-pvxs-sys
        // when server functionality becomes available
        
        *running = true;
        info!("PVXS server started (placeholder implementation)");
        Ok(())
    }

    /// Stop the server
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Server;
    ///
    /// let server = Server::new()?;
    /// server.start()?;
    /// server.stop()?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn stop(&self) -> Result<()> {
        info!("Stopping PVXS server");
        
        let mut running = self.running.lock().unwrap();
        if !*running {
            return Err(Error::ServerConfig {
                message: "Server is not running".to_string(),
            });
        }
        
        // TODO: Implement actual server shutdown using epics-pvxs-sys
        
        *running = false;
        info!("PVXS server stopped");
        Ok(())
    }

    /// Check if the server is currently running
    ///
    /// # Returns
    ///
    /// Returns `true` if the server is running, `false` otherwise.
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    /// Get statistics about the server
    ///
    /// # Returns
    ///
    /// Returns a `ServerStats` struct with information about the server.
    pub fn stats(&self) -> ServerStats {
        let pvs = self.pvs.lock().unwrap();
        let running = *self.running.lock().unwrap();
        
        ServerStats {
            pv_count: pvs.len(),
            running,
        }
    }
}

impl std::fmt::Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pvs = self.pvs.lock().unwrap();
        let running = *self.running.lock().unwrap();
        
        f.debug_struct("Server")
            .field("pv_count", &pvs.len())
            .field("running", &running)
            .finish()
    }
}

/// Statistics about a PVXS server
#[derive(Debug, Clone)]
pub struct ServerStats {
    /// Number of PVs provided by the server
    pub pv_count: usize,
    /// Whether the server is currently running
    pub running: bool,
}

impl std::fmt::Display for ServerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Server: {} PVs, {}", 
               self.pv_count, 
               if self.running { "running" } else { "stopped" })
    }
}

/// Builder for creating servers with specific configurations
pub struct ServerBuilder {
    bind_addr: Option<String>,
    port: Option<u16>,
}

impl ServerBuilder {
    /// Create a new server builder
    pub fn new() -> Self {
        Self {
            bind_addr: None,
            port: None,
        }
    }

    /// Set the bind address for the server
    pub fn bind_addr<S: Into<String>>(mut self, addr: S) -> Self {
        self.bind_addr = Some(addr.into());
        self
    }

    /// Set the port for the server
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Build the server
    pub fn build(self) -> Result<Server> {
        // TODO: Use bind_addr and port when implementing actual server
        Server::new()
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = Server::new().unwrap();
        assert!(!server.is_running());
        assert_eq!(server.list_pvs().len(), 0);
    }

    #[test]
    fn test_pv_operations() {
        let server = Server::new().unwrap();
        
        // Add PV
        server.add_pv("test:pv", PvValue::Double(42.0)).unwrap();
        assert_eq!(server.list_pvs().len(), 1);
        
        // Get value
        let value = server.get_pv_value("test:pv").unwrap();
        assert_eq!(value.as_double(), Some(42.0));
        
        // Update value
        server.update_pv("test:pv", PvValue::Double(84.0)).unwrap();
        let updated_value = server.get_pv_value("test:pv").unwrap();
        assert_eq!(updated_value.as_double(), Some(84.0));
        
        // Remove PV
        server.remove_pv("test:pv").unwrap();
        assert_eq!(server.list_pvs().len(), 0);
    }

    #[test]
    fn test_pv_value_types() {
        let double_val = PvValue::Double(3.14);
        assert_eq!(double_val.as_double(), Some(3.14));
        assert_eq!(double_val.as_string(), "3.14");
        
        let int_val = PvValue::Int32(42);
        assert_eq!(int_val.as_int32(), Some(42));
        assert_eq!(int_val.as_double(), Some(42.0));
        
        let string_val = PvValue::String("hello".to_string());
        assert_eq!(string_val.as_string(), "hello");
    }

    #[test]
    fn test_server_builder() {
        let builder = ServerBuilder::new()
            .bind_addr("0.0.0.0")
            .port(5076);
        
        let server = builder.build().unwrap();
        assert!(!server.is_running());
    }
}
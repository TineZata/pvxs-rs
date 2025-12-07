//! Server API for providing EPICS PVs
//!
//! This module provides a high-level server interface for creating EPICS
//! servers that can provide Process Variables to clients.

use crate::error::{Error, Result};
use crate::types::{Value, AlarmSeverity, AlarmStatus};
use epics_pvxs_sys::{NTScalarMetadataBuilder, Server as PvxsServer, SharedPV};
use tracing::{debug, info};
use std::collections::HashSet;

/// High-level PVXS server for providing EPICS PVs
pub struct Server {
    inner: PvxsServer,
    pv_names: HashSet<String>,
}

impl Server {
    /// Create a new PVXS server from environment configuration
    ///
    /// This will read EPICS environment variables to configure the server.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Server;
    ///
    /// let mut server = Server::new()?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn new() -> Result<Self> {
        debug!("Creating new PVXS server from environment");
        
        let inner = PvxsServer::from_env()
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to create PVXS server: {}", e),
            })?;
        
        info!("PVXS server created successfully");
        Ok(Self { 
            inner,
            pv_names: HashSet::new(),
        })
    }

    /// Create a new isolated PVXS server
    ///
    /// An isolated server uses a random port and doesn't interact with
    /// normal EPICS network discovery. Useful for testing.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Server;
    ///
    /// let mut server = Server::new_isolated()?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn new_isolated() -> Result<Self> {
        debug!("Creating new isolated PVXS server");
        
        let inner = PvxsServer::create_isolated()
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to create isolated PVXS server: {}", e),
            })?;
        
        info!("Isolated PVXS server created successfully");
        Ok(Self { 
            inner,
            pv_names: HashSet::new(),
        })
    }

    /// Create and add a new double PV to the server
    ///
    /// # Arguments
    ///
    /// * `pv_name` - Name of the Process Variable
    /// * `initial_value` - Initial value for the PV
    /// * `metadata` - Metadata for the scalar PV
    ///
    /// # Returns
    ///
    /// Returns a `Pv` handle that can be used to update the PV value.
    /// If a PV with this name already exists, returns a handle to the existing PV.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Server;
    ///
    /// let mut server = Server::new()?;
    /// let mut pv = server.add_double_pv("test:voltage", 3.3)?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn add_double_pv(&mut self, pv_name: &str, initial_value: f64, metadata: NTScalarMetadataBuilder) -> Result<Pv> {
        // Check if PV already exists
        if self.pv_names.contains(pv_name) {
            return Err(Error::ServerConfig {
                message: format!("PV '{}' already exists on this server", pv_name),
            });
        }

        debug!("Adding double PV: {} with value: {}", pv_name, initial_value);
        
        let shared_pv = self.inner.create_pv_double(pv_name, initial_value, metadata)
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to create double PV: {}", e),
            })?;
        
        self.pv_names.insert(pv_name.to_string());
        
        info!("Added double PV: {}", pv_name);
        Ok(Pv {
            inner: shared_pv,
            name: pv_name.to_string(),
        })
    }

    /// Create and add a new double PV with metadata to the server
    /// 
    /// # Arguments
    /// * `pv_name` - Name of the Process Variable
    /// * `initial_value` - Initial value for the PV
    /// * `metadata` - Metadata for the scalar PV
    /// # Returns
    /// 
    /// Returns a `Pv` handle that can be used to update the PV value.
    /// If a PV with this name already exists, returns an error.
    /// # Example
    /// ```rust,no_run
    /// use pvxs::Server;
    /// use epics_pvxs_sys::NTScalarMetadataBuilder;
    /// 
    /// let metadata = NTScalarMetadataBuilder::default();
    /// let mut server = Server::new()?;
    /// let mut pv = server.add_double_pv_with_metadata("test:temperature",
    ///   25.0, metadata)?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    /// 
    pub fn add_double_pv_with_metadata(&mut self, pv_name: &str, initial_value: f64, metadata: NTScalarMetadataBuilder) -> Result<Pv> {
        // Check if PV already exists
        if self.pv_names.contains(pv_name) {
            return Err(Error::ServerConfig {
                message: format!("PV '{}' already exists on this server", pv_name),
            });
        }

        debug!("Adding double PV with metadata: {} with value: {}", pv_name, initial_value);
        
        let shared_pv = self.inner
            .create_pv_double(pv_name, initial_value, metadata)
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to create double PV with metadata: {}", e),
            })?;
        
        self.pv_names.insert(pv_name.to_string());
        
        info!("Added double PV with metadata: {}", pv_name);
        Ok(Pv {
            inner: shared_pv,
            name: pv_name.to_string(),
        })
    }

    /// Create and add a new int32 PV to the server
    ///
    /// # Arguments
    ///
    /// * `pv_name` - Name of the Process Variable
    /// * `initial_value` - Initial value for the PV
    ///
    /// # Returns
    ///
    /// Returns a `Pv` handle that can be used to update the PV value.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Server;
    ///
    /// let mut server = Server::new()?;
    /// let mut pv = server.add_int32_pv("test:counter", 0)?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn add_int32_pv(&mut self, pv_name: &str, initial_value: i32) -> Result<Pv> {
        // Check if PV already exists
        if self.pv_names.contains(pv_name) {
            return Err(Error::ServerConfig {
                message: format!("PV '{}' already exists on this server", pv_name),
            });
        }

        debug!("Adding int32 PV: {} with value: {}", pv_name, initial_value);
        
        let shared_pv = self.inner
            .create_pv_int32(pv_name, initial_value, NTScalarMetadataBuilder::default())
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to create int32 PV: {}", e),
            })?;
        
        self.pv_names.insert(pv_name.to_string());
        
        info!("Added int32 PV: {}", pv_name);
        Ok(Pv {
            inner: shared_pv,
            name: pv_name.to_string(),
        })
    }

    /// Create and add a new string PV to the server
    ///
    /// # Arguments
    ///
    /// * `pv_name` - Name of the Process Variable
    /// * `initial_value` - Initial value for the PV
    ///
    /// # Returns
    ///
    /// Returns a `Pv` handle that can be used to update the PV value.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Server;
    ///
    /// let mut server = Server::new()?;
    /// let mut pv = server.add_string_pv("test:status", "IDLE")?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn add_string_pv(&mut self, pv_name: &str, initial_value: &str) -> Result<Pv> {
        // Check if PV already exists
        if self.pv_names.contains(pv_name) {
            return Err(Error::ServerConfig {
                message: format!("PV '{}' already exists on this server", pv_name),
            });
        }

        debug!("Adding string PV: {} with value: {}", pv_name, initial_value);
        
        let shared_pv = self.inner
            .create_pv_string(pv_name, initial_value, NTScalarMetadataBuilder::default())
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to create string PV: {}", e),
            })?;
        
        self.pv_names.insert(pv_name.to_string());
        
        info!("Added string PV: {}", pv_name);
        Ok(Pv {
            inner: shared_pv,
            name: pv_name.to_string(),
        })
    }

    /// Create and add a new enum PV to the server
    /// 
    /// # Arguments
    /// * `pv_name` - Name of the Process Variable
    /// * `choices` - List of enum choices
    /// * `selected_value` - Initial value for the PV
    /// # Returns
    ///
    /// Returns a `Pv` handle that can be used to update the PV value.
    /// # Example
    /// ```rust,no_run
    /// use pvxs::Server;
    /// 
    /// let mut server = Server::new()?;
    /// let mut pv = server.add_enum_pv("test:mode", "AUTO", "MANUAL", "TECTONIC", 0)?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn add_enum_pv(&mut self, pv_name: &str, choices: Vec<&str>, selected_value: i16) -> Result<Pv> {
        // Check if PV already exists
        if self.pv_names.contains(pv_name) {
            return Err(Error::ServerConfig {
                message: format!("PV '{}' already exists on this server", pv_name),
            });
        }

        debug!("Adding enum PV: {} with value: {}", pv_name, selected_value);
        
        let shared_pv = self.inner
            .create_pv_enum(pv_name, choices, selected_value, Default::default())
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to create enum PV: {}", e),
            })?;
        
        self.pv_names.insert(pv_name.to_string());
        
        info!("Added enum PV: {}", pv_name);
        Ok(Pv {
            inner: shared_pv,
            name: pv_name.to_string(),
        })
    }

    /// Create and add a new read-only double PV to the server
    ///
    /// # Arguments
    ///
    /// * `pv_name` - Name of the Process Variable
    /// * `initial_value` - Initial value for the PV
    ///
    /// # Returns
    ///
    /// Returns a `Pv` handle that can be used to update the PV value.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Server;
    ///
    /// let mut server = Server::new()?;
    /// let mut pv = server.add_readonly_double_pv("test:constant", 299792458.0)?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn add_readonly_double_pv(&mut self, pv_name: &str, initial_value: f64) -> Result<Pv> {
        // Check if PV already exists
        if self.pv_names.contains(pv_name) {
            return Err(Error::ServerConfig {
                message: format!("PV '{}' already exists on this server", pv_name),
            });
        }

        debug!("Adding readonly double PV: {} with value: {}", pv_name, initial_value);
        
        let shared_pv = self.inner
            .create_readonly_pv_double(pv_name, initial_value, NTScalarMetadataBuilder::default())
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to create readonly double PV: {}", e),
            })?;
        
        self.pv_names.insert(pv_name.to_string());
        
        info!("Added readonly double PV: {}", pv_name);
        Ok(Pv {
            inner: shared_pv,
            name: pv_name.to_string(),
        })
    }

    /// Remove a PV from the server
    ///
    /// # Arguments
    ///
    /// * `pv_name` - Name of the Process Variable to remove
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Server;
    ///
    /// let mut server = Server::new()?;
    /// server.add_double_pv("temp:pv", 0.0)?;
    /// server.remove_pv("temp:pv")?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn remove_pv(&mut self, pv_name: &str) -> Result<()> {
        debug!("Removing PV: {}", pv_name);
        
        self.inner
            .remove_pv(pv_name)
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to remove PV '{}': {}", pv_name, e),
            })?;
        
        // Remove from tracking set
        self.pv_names.remove(pv_name);
        
        info!("Removed PV: {}", pv_name);
        Ok(())
    }

    /// Start the server
    ///
    /// Begins listening for client connections and serving PVs.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Server;
    ///
    /// let mut server = Server::new()?;
    /// server.add_double_pv("test:pv", 42.0)?;
    /// server.start()?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn start(&mut self) -> Result<()> {
        info!("Starting PVXS server");
        
        self.inner
            .start()
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to start server: {}", e),
            })?;
        
        info!("PVXS server started successfully on TCP port {}, UDP port {}", 
               self.tcp_port(), self.udp_port());
        Ok(())
    }

    /// Stop the server
    ///
    /// Stops listening for connections and shuts down the server.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Server;
    ///
    /// let mut server = Server::new()?;
    /// server.start()?;
    /// server.stop()?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn stop(&mut self) -> Result<()> {
        info!("Stopping PVXS server");
        
        self.inner
            .stop()
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to stop server: {}", e),
            })?;
        
        info!("PVXS server stopped");
        Ok(())
    }

    /// Get the TCP port the server is using
    ///
    /// Returns 
    ///  - non-zero for isolated servers.
    ///  - For remoted servers, tries to bind to 5075 (EPICS v7) first but may fall back to a random port if that is in use.
    pub fn tcp_port(&self) -> u16 {
        self.inner.tcp_port()
    }

    /// Get the UDP port the server is using
    ///
    /// Returns 
    ///  - non-zero for isolated servers.
    ///  - For remote servers, tries to bind to 5076 (EPICS v7) first but may fall back to a random port if that is in use.
    pub fn udp_port(&self) -> u16 {
        self.inner.udp_port()
    }
}

impl std::fmt::Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("tcp_port", &self.tcp_port())
            .field("udp_port", &self.udp_port())
            .finish()
    }
}

/// A handle to a Process Variable in the server
///
/// This allows updating the PV value and reading its current value.
pub struct Pv {
    inner: SharedPV,
    name: String,
}

impl Pv {
    /// Update the PV with a double value
    ///
    /// # Arguments
    ///
    /// * `value` - New value to post
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Server;
    /// # let mut server = Server::new().unwrap();
    /// let mut pv = server.add_double_pv("test:voltage", 3.3)?;
    /// pv.post_double(5.0)?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn post_double(&mut self, value: f64) -> Result<()> {
        debug!("Posting double value {} to PV: {}", value, self.name);
        
        self.inner
            .post_double(value)
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to post double value to '{}': {}", self.name, e),
            })?;
        
        Ok(())
    }

    /// Update the PV with an int32 value
    ///
    /// # Arguments
    ///
    /// * `value` - New value to post
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Server;
    /// # let mut server = Server::new().unwrap();
    /// let mut pv = server.add_int32_pv("test:counter", 0)?;
    /// pv.post_int32(42)?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn post_int32(&mut self, value: i32) -> Result<()> {
        debug!("Posting int32 value {} to PV: {}", value, self.name);
        
        self.inner
            .post_int32(value)
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to post int32 value to '{}': {}", self.name, e),
            })?;
        
        Ok(())
    }

    /// Update the PV with a string value
    ///
    /// # Arguments
    ///
    /// * `value` - New value to post
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Server;
    /// # let mut server = Server::new().unwrap();
    /// let mut pv = server.add_string_pv("test:status", "IDLE")?;
    /// pv.post_string("RUNNING")?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn post_string(&mut self, value: &str) -> Result<()> {
        debug!("Posting string value '{}' to PV: {}", value, self.name);
        
        self.inner
            .post_string(value)
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to post string value to '{}': {}", self.name, e),
            })?;
        
        Ok(())
    }

    /// Update the PV enum selected value
    /// 
    /// # Arguments
    /// * `value` - New enum index to post
    /// # Example
    /// ```rust,no_run
    /// # use pvxs::Server;
    /// # let mut server = Server::new().unwrap();
    /// let mut pv = server.add_enum_pv("test:mode", &["AUTO", "MANUAL", "TECTONIC"], 0)?;
    /// pv.post_enum(1)?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn post_enum(&mut self, value: i16) -> Result<()> {
        debug!("Posting enum index {} to PV: {}", value, self.name);
        
        self.inner
            .post_enum(value)
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to post enum index to '{}': {}", self.name, e),
            })?;
        
        Ok(())
    }

    /// Update the PV with a double value and alarm information
    ///
    /// # Arguments
    ///
    /// * `value` - New value to post
    /// * `severity` - Alarm severity (0=NO_ALARM, 1=MINOR, 2=MAJOR, 3=INVALID)
    /// * `status` - Alarm status code (0=NO_ALARM, 1=DEVICE, 2=DRIVER, 3=RECORD, 4=DATABASE, 5=CONFIG, 6=UNDEFINED, 7=CLIENT)
    /// * `message` - Alarm message
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Server;
    /// # let mut server = Server::new().unwrap();
    /// let mut pv = server.add_double_pv("test:temp", 20.0)?;
    /// pv.post_double_with_alarm(100.0, 2, 0, "High temperature")?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn post_double_with_alarm(&mut self, value: f64, severity: AlarmSeverity, status: AlarmStatus, message: &str) -> Result<()> {
        debug!("Posting double value {} with alarm to PV: {}", value, self.name);
        
        self.inner
            .post_double_with_alarm(value, severity.as_int(), status.as_int(), message)
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to post double value with alarm to '{}': {}", self.name, e),
            })?;
        
        Ok(())
    }

    /// Update the PV with an int32 value and alarm information
    ///
    /// # Arguments
    ///
    /// * `value` - New value to post
    /// * `severity` - Alarm severity (0=NO_ALARM, 1=MINOR, 2=MAJOR, 3=INVALID)
    /// * `status` - Alarm status code
    /// * `message` - Alarm message
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Server;
    /// # let mut server = Server::new().unwrap();
    /// let mut pv = server.add_int32_pv("test:errors", 0)?;
    /// pv.post_int32_with_alarm(10, 1, 0, "Error count elevated")?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn post_int32_with_alarm(&mut self, value: i32, severity: AlarmSeverity, status: AlarmStatus, message: &str) -> Result<()> {
        debug!("Posting int32 value {} with alarm to PV: {}", value, self.name);
        
        self.inner
            .post_int32_with_alarm(value, severity.as_int(), status.as_int(), message)
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to post int32 value with alarm to '{}': {}", self.name, e),
            })?;
        
        Ok(())
    }

    /// Update the PV with a string value and alarm information
    ///
    /// # Arguments
    ///
    /// * `value` - New value to post
    /// * `severity` - Alarm severity (0=NO_ALARM, 1=MINOR, 2=MAJOR, 3=INVALID)
    /// * `status` - Alarm status code
    /// * `message` - Alarm message
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Server;
    /// # let mut server = Server::new().unwrap();
    /// let mut pv = server.add_string_pv("test:status", "OK")?;
    /// pv.post_string_with_alarm("ERROR", 2, 0, "System failure")?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn post_string_with_alarm(&mut self, value: &str, severity: AlarmSeverity, status: AlarmStatus, message: &str) -> Result<()> {
        debug!("Posting string value '{}' with alarm to PV: {}", value, self.name);
        
        self.inner
            .post_string_with_alarm(value, severity.as_int(), status.as_int(), message)
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to post string value with alarm to '{}': {}", self.name, e),
            })?;
        
        Ok(())
    }

    /// Update the PV enum selected value with alarm information
    /// 
    /// # Arguments
    /// * `value` - New enum index to post
    /// * `severity` - Alarm severity (0=NO_ALARM, 1=
    /// MINOR, 2=MAJOR, 3=INVALID)
    /// * `status` - Alarm status code
    /// * `message` - Alarm message
    /// # Example
    /// ```rust,no_run
    /// # use pvxs::Server;
    /// # let mut server = Server::new().unwrap();
    /// let mut pv = server.add_enum_pv("test:mode", &["AUTO", "MANUAL", "TECTONIC"], 0)?;
    /// pv.post_enum_with_alarm(1, AlarmSeverity::Minor, AlarmStatus::Device, "Mode changed")?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    /// 
    pub fn post_enum_with_alarm(&mut self, value: i16, severity: AlarmSeverity, status: AlarmStatus, message: &str) -> Result<()> {
        debug!("Posting enum index {} with alarm to PV: {}", value, self.name);
        
        self.inner
            .post_enum_with_alarm(value, severity.as_int(), status.as_int(), message)
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to post enum index with alarm to '{}': {}", self.name, e),
            })?;
        
        Ok(())
    }

    /// Fetch the current value of the PV (server-side read)
    ///
    /// This performs a server-side fetch to read the current value stored in the PV.
    /// This is useful for verifying what value is currently being served to clients.
    ///
    /// # Returns
    ///
    /// Returns a `Value` containing the current PV data including the value field
    /// and any metadata (timestamp, alarm information, etc.).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Server;
    /// # let mut server = Server::new().unwrap();
    /// let mut pv = server.add_double_pv("test:voltage", 3.3)?;
    /// pv.post_double(5.0)?;
    /// 
    /// // Fetch and verify the value
    /// let value = pv.fetch()?;
    /// assert_eq!(value.as_double()?, 5.0);
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn fetch(&mut self) -> Result<Value> {
        debug!("Fetching value from PV: {}", self.name);
        
        let pvxs_value = self.inner
            .fetch()
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to fetch value from '{}': {}", self.name, e),
            })?;
        
        Ok(Value::from_pvxs(pvxs_value))
    }

    /// Get the name of this PV
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl std::fmt::Debug for Pv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pv")
            .field("name", &self.name)
            .finish()
    }
}

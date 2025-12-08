use epics_pvxs_sys::SharedPV;
use crate::error::{Error, Result};
use tracing::debug;
use crate::types::Value;

/// A handle to a Process Variable in the server
///
/// This allows updating the PV value and reading its current value.
pub struct Pv {
    pub(super) inner: SharedPV,
    pub(super) name: String,
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
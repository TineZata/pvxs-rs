use epics_pvxs_sys::{NTScalarMetadataBuilder, DisplayMetadata};
use crate::error::{Error, Result};
use crate::server::Server;
use crate::server::pv::Pv;
use tracing::{debug, info};

/// Builder for creating a string PV
///
/// # Example
///
/// ```rust,no_run
/// use pvxs::Server;
///
/// let mut server = Server::new()?;
/// 
/// let status = server.string_pv("device:status")
///     .initial_value("IDLE")
///     .description("Device status message")
///     .add()?;
/// # Ok::<(), pvxs::Error>(())
/// ```
pub struct StringPvBuilder<'a> {
    server: &'a mut Server,
    name: String,
    initial_value: String,
    metadata: NTScalarMetadataBuilder,
    has_metadata: bool,
}

impl<'a> StringPvBuilder<'a> {
    pub(super) fn new(server: &'a mut Server, name: impl Into<String>) -> Self {
        Self {
            server,
            name: name.into(),
            initial_value: String::new(),
            metadata: NTScalarMetadataBuilder::new(),
            has_metadata: false,
        }
    }

    /// Set the initial value for this PV
    pub fn initial_value(mut self, value: impl Into<String>) -> Self {
        self.initial_value = value.into();
        self
    }

    /// Set a description for this PV
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.has_metadata = true;
        let display = DisplayMetadata {
            limit_low: 0,
            limit_high: 0,
            description: desc.into(),
            units: String::new(),
            precision: 0,
        };
        self.metadata = self.metadata.display(display);
        self
    }

    /// Add this PV to the server
    pub fn add(self) -> Result<Pv> {
        if self.server.pv_names.contains(&self.name) {
            return Err(Error::ServerConfig {
                message: format!("PV '{}' already exists on this server", self.name),
            });
        }

        debug!("Adding string PV '{}' with value: '{}'", self.name, self.initial_value);

        let metadata = if self.has_metadata {
            self.metadata
        } else {
            NTScalarMetadataBuilder::default()
        };

        let shared_pv = self.server.inner.create_pv_string(&self.name, &self.initial_value, metadata)
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to create string PV '{}': {}", self.name, e),
            })?;

        self.server.pv_names.insert(self.name.clone());
        info!("Added string PV: {}", self.name);

        Ok(Pv {
            inner: shared_pv,
            name: self.name,
        })
    }
}

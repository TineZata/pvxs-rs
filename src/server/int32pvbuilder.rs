use epics_pvxs_sys::{NTScalarMetadataBuilder, DisplayMetadata};
use crate::error::{Error, Result};
use crate::server::Server;
use crate::server::pv::Pv;
use tracing::{debug, info};

/// Builder for creating an integer (int32) PV
///
/// # Example
///
/// ```rust,no_run
/// use pvxs::Server;
///
/// let mut server = Server::new()?;
/// 
/// let counter = server.int32_pv("device:counter")
///     .initial_value(0)
///     .description("Event counter")
///     .add()?;
/// # Ok::<(), pvxs::Error>(())
/// ```
pub struct Int32PvBuilder<'a> {
    server: &'a mut Server,
    name: String,
    initial_value: i32,
    metadata: NTScalarMetadataBuilder,
    has_metadata: bool,
}

impl<'a> Int32PvBuilder<'a> {
    pub(super) fn new(server: &'a mut Server, name: impl Into<String>) -> Self {
        Self {
            server,
            name: name.into(),
            initial_value: 0,
            metadata: NTScalarMetadataBuilder::new(),
            has_metadata: false,
        }
    }

    /// Set the initial value for this PV
    pub fn initial_value(mut self, value: i32) -> Self {
        self.initial_value = value;
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

    /// Set the engineering units
    pub fn units(mut self, units: impl Into<String>) -> Self {
        self.has_metadata = true;
        let display = DisplayMetadata {
            limit_low: 0,
            limit_high: 0,
            description: String::new(),
            units: units.into(),
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

        debug!("Adding int32 PV '{}' with value: {}", self.name, self.initial_value);

        let metadata = if self.has_metadata {
            self.metadata
        } else {
            NTScalarMetadataBuilder::default()
        };

        let shared_pv = self.server.inner.create_pv_int32(&self.name, self.initial_value, metadata)
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to create int32 PV '{}': {}", self.name, e),
            })?;

        self.server.pv_names.insert(self.name.clone());
        info!("Added int32 PV: {}", self.name);

        Ok(Pv {
            inner: shared_pv,
            name: self.name,
        })
    }
}
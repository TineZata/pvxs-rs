use epics_pvxs_sys::{NTScalarMetadataBuilder, DisplayMetadata, ControlMetadata};
use crate::error::{Error, Result};
use crate::server::Server;
use crate::server::pv::Pv;
use tracing::{debug, info};


// ============================================================================
// PV Builder Pattern - High-level ergonomic API
// ============================================================================

/// Builder for creating a double-precision floating point PV
///
/// This provides a fluent API for configuring PVs with sensible defaults.
/// Metadata fields are optional and only included if explicitly set.
///
/// # Example
///
/// ```rust,no_run
/// use pvxs::{Server, DoublePvBuilder};
///
/// let mut server = Server::new()?;
/// 
/// // Simple PV with just a value
/// let pv = server.double_pv("simple:voltage")
///     .initial_value(3.3)
///     .add()?;
///
/// // PV with metadata
/// let pv = server.double_pv("device:temperature")
///     .initial_value(20.0)
///     .units("degC")
///     .display_range(0.0, 100.0)
///     .precision(2)
///     .description("Device temperature sensor")
///     .add()?;
/// # Ok::<(), pvxs::Error>(())
/// ```
pub struct DoublePvBuilder<'a> {
    server: &'a mut Server,
    name: String,
    initial_value: f64,
    metadata: NTScalarMetadataBuilder,
    has_metadata: bool,
}

impl<'a> DoublePvBuilder<'a> {
    pub(super) fn new(server: &'a mut Server, name: impl Into<String>) -> Self {
        Self {
            server,
            name: name.into(),
            initial_value: 0.0,
            metadata: NTScalarMetadataBuilder::new(),
            has_metadata: false,
        }
    }

    /// Set the initial value for this PV
    pub fn initial_value(mut self, value: f64) -> Self {
        self.initial_value = value;
        self
    }

    /// Set the engineering units for this value (e.g., "mm", "degC", "V")
    pub fn units(mut self, units: impl Into<String>) -> Self {
        self.has_metadata = true;
        // Update display metadata with units
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

    /// Set display limits for UI rendering (min, max)
    pub fn display_range(mut self, low: f64, high: f64) -> Self {
        self.has_metadata = true;
        let display = DisplayMetadata {
            limit_low: low as i64,
            limit_high: high as i64,
            description: String::new(),
            units: String::new(),
            precision: 0,
        };
        self.metadata = self.metadata.display(display);
        self
    }

    /// Set the precision (number of decimal places) for display
    pub fn precision(mut self, precision: i32) -> Self {
        self.has_metadata = true;
        let display = DisplayMetadata {
            limit_low: 0,
            limit_high: 0,
            description: String::new(),
            units: String::new(),
            precision,
        };
        self.metadata = self.metadata.display(display);
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

    /// Set control limits for operator input (min, max, step)
    pub fn control_range(mut self, low: f64, high: f64, min_step: f64) -> Self {
        self.has_metadata = true;
        self.metadata = self.metadata.control(ControlMetadata {
            limit_low: low,
            limit_high: high,
            min_step,
        });
        self
    }

    /// Add this PV to the server
    ///
    /// Consumes the builder and returns a `Pv` handle for updating the value.
    pub fn add(self) -> Result<Pv> {
        // Check for duplicate
        if self.server.pv_names.contains(&self.name) {
            return Err(Error::ServerConfig {
                message: format!("PV '{}' already exists on this server", self.name),
            });
        }

        debug!("Adding double PV '{}' with value: {}", self.name, self.initial_value);

        // Create the PV using -sys API
        let metadata = if self.has_metadata {
            self.metadata
        } else {
            NTScalarMetadataBuilder::default()
        };

        let shared_pv = self.server.inner.create_pv_double(&self.name, self.initial_value, metadata)
            .map_err(|e| Error::ServerConfig {
                message: format!("Failed to create double PV '{}': {}", self.name, e),
            })?;

        self.server.pv_names.insert(self.name.clone());
        info!("Added double PV: {}", self.name);

        Ok(Pv {
            inner: shared_pv,
            name: self.name,
        })
    }
}

//! Error types and handling for PVXS operations

use thiserror::Error;

/// Result type for PVXS operations
pub type Result<T> = std::result::Result<T, Error>;

/// Comprehensive error type for PVXS operations
#[derive(Error, Debug)]
pub enum Error {
    /// Errors from the underlying PVXS library
    #[error("PVXS error: {message}")]
    PvxsError { message: String },

    /// Connection timeout or failure
    #[error("Connection error: {message}")]
    ConnectionError { message: String },

    /// Timeout during operation
    #[error("Operation timed out after {timeout}s")]
    Timeout { timeout: f64 },

    /// PV name not found
    #[error("PV not found: {pv_name}")]
    PvNotFound { pv_name: String },

    /// Invalid PV name format
    #[error("Invalid PV name: {pv_name}")]
    InvalidPvName { pv_name: String },

    /// Type conversion error
    #[error("Type conversion error: {message}")]
    TypeConversion { message: String },

    /// Field access error
    #[error("Field access error: field '{field}' not found or inaccessible")]
    FieldAccess { field: String },

    /// Server configuration error
    #[error("Server configuration error: {message}")]
    ServerConfig { message: String },

    /// Network error
    #[error("Network error: {message}")]
    Network { message: String },

    /// Invalid configuration
    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Error {
    /// Create a new PVXS error
    pub fn pvxs<S: Into<String>>(message: S) -> Self {
        Self::PvxsError { message: message.into() }
    }

    /// Create a new connection error
    pub fn connection<S: Into<String>>(message: S) -> Self {
        Self::ConnectionError { message: message.into() }
    }

    /// Create a new timeout error
    pub fn timedout(timeout: f64) -> Self {
        Self::Timeout { timeout }
    }

    /// Create a new PV not found error
    pub fn pv_not_found<S: Into<String>>(pv_name: S) -> Self {
        Self::PvNotFound { pv_name: pv_name.into() }
    }

    /// Create a new type conversion error
    pub fn type_conversion<S: Into<String>>(message: S) -> Self {
        Self::TypeConversion { message: message.into() }
    }

    /// Create a new field access error
    pub fn field_access<S: Into<String>>(field: S) -> Self {
        Self::FieldAccess { field: field.into() }
    }

    /// Check if this is a timeout error
    pub fn is_timeout(&self) -> bool {
        matches!(self, Error::Timeout { .. })
    }

    /// Check if this is a connection error
    pub fn is_connection_error(&self) -> bool {
        matches!(self, Error::ConnectionError { .. })
    }
}

/// Convert from epics-pvxs-sys errors to our error type
impl From<epics_pvxs_sys::PvxsError> for Error {
    fn from(err: epics_pvxs_sys::PvxsError) -> Self {
        Error::PvxsError { 
            message: err.to_string() 
        }
    }
}
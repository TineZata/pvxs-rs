// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
#![deny(missing_docs)]
//! Pure-Rust pvAccess implementation — same public API as pvxs-sys,
//! no C++ toolchain, no EPICS_BASE, no DLLs.
//!
//! ## Overview
//!
//! This crate provides the same surface as `pvxs-sys` backed entirely by Rust.
//! Consumers can swap the dependency without source changes.
//!
//! ## Client
//! - **GET**: [`Context::get`]
//! - **PUT**: [`Context::put_double`], [`Context::put_int32`], [`Context::put_string`],
//!   [`Context::put_enum`], and their `_array` variants
//! - **Monitor**: [`Context::monitor_builder`] → [`MonitorBuilder::exec`] → [`Monitor::pop`]
//! - **RPC**: [`Context::rpc`] → [`Rpc::execute`]
//!
//! ## Server
//! - **Start**: [`Server::start_from_env`] or [`Server::start_isolated`]
//! - **PV creation**: `create_pv_double`, `create_pv_int32`, `create_pv_string`,
//!   `create_pv_enum`, and their `_array` variants
//! - **POST**: `post_double`, `post_int32`, `post_string`, `post_enum`, and `_array` variants
//! - **Fetch**: `fetch_double`, `fetch_int32`, `fetch_string`, `fetch_enum`
//! - **Stop**: [`Server::stop_drop`]
//! - **Handle**: [`ServerHandle`]
//!
//! ## Metadata & Alarms
//! - [`NTScalarMetadataBuilder`] / [`NTEnumMetadataBuilder`]
//! - [`ControlMetadata`], [`AlarmMetadata`], [`DisplayMetadata`]
//! - [`AlarmSeverity`], [`AlarmStatus`]
//!
//! ## Network status
//! UDP discovery and TCP GET, PUT, Monitor, and server transport are implemented.
//! RPC and optional advanced transport features remain incomplete; see `TODO.md`.

/// Alarm computation and metadata types.
pub mod alarms;
/// PvAccess client API surface.
pub mod client;
/// NTScalar/NTEnum metadata builders.
pub mod metadata;
/// In-memory server implementation and compatibility types.
pub mod server;
/// Dynamic value container and field access helpers.
pub mod value;

pub(crate) mod proto;
pub(crate) mod pvdata;
pub(crate) mod net;

use std::fmt;

pub use alarms::{compute_alarm_for_scalar, AlarmConfig, AlarmResult, AlarmSeverity, AlarmStatus};
pub use client::{Context, Monitor, MonitorBuilder, MonitorEvent, Rpc};
pub use metadata::{AlarmMetadata, ControlMetadata, DisplayMetadata};
pub use server::{
    FetchedDouble, FetchedDoubleArray, FetchedEnum, FetchedInt32, FetchedInt32Array, FetchedString,
    FetchedStringArray, NTEnumMetadataBuilder, NTScalarMetadataBuilder, Server, ServerHandle,
    SharedPV, StaticSource,
};
pub use value::{FieldType, Value};
pub use std::sync::atomic::{AtomicUsize, Ordering};

/// Convenience type alias — every fallible operation in this crate returns this.
pub type Result<T> = std::result::Result<T, PvxsError>;

/// Error type for pvxs-rs operations.
#[derive(Debug, Clone)]
pub struct PvxsError {
    message: String,
}

impl PvxsError {
    /// Create a new error with a human-readable message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PvxsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pvxs error: {}", self.message)
    }
}

impl std::error::Error for PvxsError {}

/// Configure logging from environment variables.
///
/// The pure-Rust implementation currently does not require explicit logger setup,
/// so this is a compatibility no-op.
pub fn configure_logging_from_env() -> Result<()> {
    Ok(())
}

/// Set logger level for a specific logger name.
///
/// The pure-Rust implementation currently does not expose named loggers,
/// so this is a compatibility no-op.
pub fn set_logger_level(_name: &str, _level: &str) -> Result<()> {
    Ok(())
}

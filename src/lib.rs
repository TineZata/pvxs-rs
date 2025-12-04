//! # PVXS - High-level Rust bindings for EPICS PVXS (PVAccess)
//!
//! This crate provides idiomatic Rust bindings for the EPICS PVXS library,
//! with separate client and server APIs for cleaner architecture.
//!
//! ## Features
//!
//! - **Client API**: Connect to EPICS IOCs, get/put/monitor PVs
//! - **Server API**: Create PVXS servers and provide both local and remote PVs  
//! - **Async Support**: Tokio-based monitor and optional async operations
//! - **Type Safety**: Strong typing with comprehensive error handling
//! - **High Performance**: Zero-copy operations where possible
//!
//! ## Module Structure
//!
//! - [`client`] - Client API for connecting to EPICS PVs. This is optional and can be enabled with the `client` feature.
//! - [`server`] - Server API for providing EPICS PVs. This is optional and can be enabled with the `server` feature.
//! - [`types`] - Common types and value representations
//! - [`error`] - Error types and handling

pub mod error;
pub mod types;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "server")]
pub mod server;

// Re-export commonly used types
/*pub use error::{Error, Result};
pub use types::{Value, Timestamp};

#[cfg(feature = "client")]
pub use client::{Client, Monitor, MonitorBuilder};

#[cfg(feature = "server")]
pub use server::Server;*/

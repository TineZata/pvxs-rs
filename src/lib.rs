//! # PVXS - High-level Rust bindings for EPICS PVXS (PVAccess)
//!
//! This crate provides idiomatic Rust bindings for the EPICS PVXS library,
//! with separate client and server APIs for cleaner architecture.
//!
//! ## Features
//!
//! - **Client API**: Connect to EPICS IOCs, get/put/monitor PVs
//! - **Server API**: Create PVXS servers and provide PVs  
//! - **Async Support**: Optional tokio-based async operations
//! - **Type Safety**: Strong typing with comprehensive error handling
//! - **High Performance**: Zero-copy operations where possible
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use pvxs::client::Client;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new()?;
//!     let value = client.get("MY:PV:NAME", 5.0)?;
//!     println!("Value: {}", value);
//!     Ok(())
//! }
//! ```
//!
//! ## Module Structure
//!
//! - [`client`] - Client API for connecting to EPICS PVs
//! - [`server`] - Server API for providing EPICS PVs
//! - [`types`] - Common types and value representations
//! - [`error`] - Error types and handling

pub mod error;
pub mod types;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "server")]
pub mod server;

// Re-export commonly used types
pub use error::{Error, Result};
pub use types::{Value, Timestamp};

#[cfg(feature = "client")]
pub use client::{Client, Monitor, MonitorBuilder};

#[cfg(feature = "server")]
pub use server::Server;

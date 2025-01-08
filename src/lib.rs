// Enable Rust features or warnings (optional but helpful)
//#![warn(missing_docs)]
//#![warn(rust_2018_idioms)]

//! # PVXS Rust Wrapper
//! A Rust wrapper for the PVXS library, providing safe and ergonomic bindings for interacting
//! with the EPICS PVXS library.
//!

// Re-export modules
pub mod bindings; // Raw bindings to the PVXS library
pub mod wrapper;  // High-level abstractions over the bindings
pub mod config;   // Configuration settings for the wrapper
pub mod context;  // Client context for the wrapper
pub mod pvdata; // Data structures for PV data handling
pub mod getbuilder; // Builder pattern for getting PV data and info

// Publicly expose functions or types from submodules
pub use wrapper::get_version_str;
pub use config::Config;
pub use context::Context;

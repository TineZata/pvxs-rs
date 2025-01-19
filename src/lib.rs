// Enable Rust features or warnings (optional but helpful)
//#![warn(missing_docs)]
//#![warn(rust_2018_idioms)]

//! # PVXS Rust Wrapper
//! A Rust wrapper for the PVXS library, providing safe and ergonomic bindings for interacting
//! with the EPICS PVXS library.
//!

// Re-export modules
pub mod pvxs_library; // Raw bindings to the PVXS library
pub mod version; // Version information for the wrapper
pub mod storetype; // Enum for PVXS storage types
pub mod std_shared_ptr; // Shared pointer for PVXS objects
pub mod client_context;

// Publicly expose functions or types from submodules



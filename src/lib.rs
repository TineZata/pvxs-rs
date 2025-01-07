// Enable Rust features or warnings (optional but helpful)
//#![warn(missing_docs)]
//#![warn(rust_2018_idioms)]

//! # PVXS Rust Wrapper
//! A Rust wrapper for the PVXS library, providing safe and ergonomic bindings for interacting
//! with the EPICS PVXS library.
//!
//! ## Example
//! ```rust
//! let version = pvxs::get_version_str();
//! println!("PVXS Version: {}", version);
//! ```

// Re-export modules
pub mod bindings; // Raw bindings to the PVXS library
pub mod wrapper;  // High-level abstractions over the bindings

// Publicly expose functions or types from submodules
pub use wrapper::get_version_str;

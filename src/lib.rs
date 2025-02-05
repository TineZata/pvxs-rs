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
pub mod std_types; // Standard types for PVXS
pub mod client; // Client context for PVXS
pub mod wrapper; // Wrapper for the PVXS library


// Publicly exposed functions or types from submodules
pub use client::{
    Context,
    Config,
};
pub use std_types::{
    GetBuilder,
    StdBasicString,
};
pub use wrapper::{
    get_version_str,
    get_context_from_env,
    get_context_config,
    //get_context_info,
};


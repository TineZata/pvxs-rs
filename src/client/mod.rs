// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
//! Pure-Rust client for pvAccess.
//! 
//! # Network status
//!
//! UDP discovery and TCP GET, PUT, and Monitor operations are implemented.
//! RPC transport remains incomplete; see `TODO.md`.

pub(crate) mod config;
pub(crate) mod context;
pub(crate) mod monitor;
pub(crate) mod rpc;

pub use self::config::ClientConfig;
pub use self::context::Context;
pub use self::monitor::{Monitor, MonitorBuilder, MonitorEvent};
pub use self::rpc::Rpc;

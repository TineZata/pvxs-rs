// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
//! Pure-Rust client for pvAccess.
//!
//! The public API mirrors `pvxs-sys::Context`, `Monitor`, `MonitorBuilder`
//! and `Rpc` exactly so callers can swap the crate without source changes.
//!
//! # Network status
//!
//! The in-process state machine (value conversion, alarm, metadata) is fully
//! implemented in pure Rust.  The TCP/UDP pvAccess transport layer is a
//! TODO — see TODO.md for the implementation plan.  All network-bound
//! methods return `Err(PvxsError)` until that layer lands.

pub (crate) mod context;
pub (crate) mod monitor;
pub (crate) mod rpc;
pub (crate) mod config;

pub use self::context::Context;
pub use self::monitor::{Monitor, MonitorBuilder, MonitorEvent};
pub use self::rpc::Rpc;
pub use self::config::ClientConfig;

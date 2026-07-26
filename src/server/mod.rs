// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
//! Pure-Rust pvAccess server — same public API as `pvxs-sys::Server`.
//!
//! All state is held in a worker thread via crossbeam channels.
//! The pvAccess TCP/UDP transport is a TODO — see TODO.md.

use crossbeam_channel as channel;
use std::thread;

use crate::{AlarmMetadata, AlarmSeverity, AlarmStatus,
    ControlMetadata, DisplayMetadata, PvxsError, Result,
};
pub(crate) mod ntscalar;
pub(crate) mod ntenum;
pub (crate) mod manager;

pub use self::ntscalar::NTScalarMetadataBuilder;
pub use self::ntenum::NTEnumMetadataBuilder;
pub use self::manager::{ManagerCommand, run_worker};

// ============================================================================
// Fetched value types (mirror pvxs-sys exactly)
// ============================================================================

#[derive(Debug, Clone)]
pub struct FetchedDouble {
    pub value: f64,
    pub alarm_severity: AlarmSeverity,
    pub alarm_status: AlarmStatus,
    pub alarm_message: String,
    pub display_metadata: Option<DisplayMetadata>,
    pub control_metadata: Option<ControlMetadata>,
    pub alarm_metadata: Option<AlarmMetadata>,
}

#[derive(Debug, Clone)]
pub struct FetchedInt32 {
    pub value: i32,
    pub alarm_severity: AlarmSeverity,
    pub alarm_status: AlarmStatus,
    pub alarm_message: String,
    pub display_metadata: Option<DisplayMetadata>,
    pub control_metadata: Option<ControlMetadata>,
    pub alarm_metadata: Option<AlarmMetadata>,
}

#[derive(Debug, Clone)]
pub struct FetchedString {
    pub value: String,
    pub alarm_severity: AlarmSeverity,
    pub alarm_status: AlarmStatus,
    pub alarm_message: String,
}

#[derive(Debug, Clone)]
pub struct FetchedDoubleArray {
    pub value: Vec<f64>,
    pub alarm_severity: AlarmSeverity,
    pub alarm_status: AlarmStatus,
    pub alarm_message: String,
    pub display_metadata: Option<DisplayMetadata>,
    pub control_metadata: Option<ControlMetadata>,
    pub alarm_metadata: Option<AlarmMetadata>,
}

#[derive(Debug, Clone)]
pub struct FetchedInt32Array {
    pub value: Vec<i32>,
    pub alarm_severity: AlarmSeverity,
    pub alarm_status: AlarmStatus,
    pub alarm_message: String,
    pub display_metadata: Option<DisplayMetadata>,
    pub control_metadata: Option<ControlMetadata>,
    pub alarm_metadata: Option<AlarmMetadata>,
}

#[derive(Debug, Clone)]
pub struct FetchedStringArray {
    pub value: Vec<String>,
    pub alarm_severity: AlarmSeverity,
    pub alarm_status: AlarmStatus,
    pub alarm_message: String,
}

#[derive(Debug, Clone)]
pub struct FetchedEnum {
    pub value: i16,
    pub value_choices: Vec<String>,
    pub alarm_severity: AlarmSeverity,
    pub alarm_status: AlarmStatus,
    pub alarm_message: String,
}

// ============================================================================
// ServerHandle
// ============================================================================

/// Clone-able, thread-safe handle to a running server.
///
/// Mirrors `pvxs-sys::ServerHandle` exactly.
#[derive(Clone)]
pub struct ServerHandle {
    tx: channel::Sender<ManagerCommand>,
    /// TODO(network): will be the real TCP port once the transport layer lands.
    tcp_port: u16,
    /// TODO(network): will be the real UDP port once the transport layer lands.
    udp_port: u16,
}

impl ServerHandle {
    pub fn tcp_port(&self) -> u16 {
        self.tcp_port
    }

    pub fn udp_port(&self) -> u16 {
        self.udp_port
    }

    fn send<T>(&self, cmd: ManagerCommand, rx: channel::Receiver<T>) -> Result<T> {
        self.tx
            .send(cmd)
            .map_err(|_| PvxsError::new("server worker stopped"))?;
        rx.recv()
            .map_err(|_| PvxsError::new("server worker stopped"))
    }

    pub fn create_pv_double(
        &self,
        name: &str,
        initial: f64,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::CreateDouble {
                name: name.to_string(),
                initial,
                metadata,
                reply: tx,
            },
            rx,
        )?
    }

    pub fn create_pv_double_array(
        &self,
        name: &str,
        initial: Vec<f64>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::CreateDoubleArray {
                name: name.to_string(),
                initial,
                metadata,
                reply: tx,
            },
            rx,
        )?
    }

    pub fn create_pv_int32(
        &self,
        name: &str,
        initial: i32,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::CreateInt32 {
                name: name.to_string(),
                initial,
                metadata,
                reply: tx,
            },
            rx,
        )?
    }

    pub fn create_pv_int32_array(
        &self,
        name: &str,
        initial: Vec<i32>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::CreateInt32Array {
                name: name.to_string(),
                initial,
                metadata,
                reply: tx,
            },
            rx,
        )?
    }

    pub fn create_pv_string(
        &self,
        name: &str,
        initial: &str,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::CreateString {
                name: name.to_string(),
                initial: initial.to_string(),
                metadata,
                reply: tx,
            },
            rx,
        )?
    }

    pub fn create_pv_string_array(
        &self,
        name: &str,
        initial: Vec<String>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::CreateStringArray {
                name: name.to_string(),
                initial,
                metadata,
                reply: tx,
            },
            rx,
        )?
    }

    pub fn create_pv_enum(
        &self,
        name: &str,
        choices: Vec<&str>,
        selected_index: i16,
        metadata: NTEnumMetadataBuilder,
    ) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::CreateEnum {
                name: name.to_string(),
                choices: choices.iter().map(|s| s.to_string()).collect(),
                selected_index,
                metadata,
                reply: tx,
            },
            rx,
        )?
    }

    pub fn post_double(&self, name: &str, value: f64) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::PostDouble {
                name: name.to_string(),
                value,
                reply: tx,
            },
            rx,
        )?
    }

    pub fn post_double_array(&self, name: &str, value: Vec<f64>) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::PostDoubleArray {
                name: name.to_string(),
                value,
                reply: tx,
            },
            rx,
        )?
    }

    pub fn post_int32(&self, name: &str, value: i32) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::PostInt32 {
                name: name.to_string(),
                value,
                reply: tx,
            },
            rx,
        )?
    }

    pub fn post_int32_array(&self, name: &str, value: Vec<i32>) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::PostInt32Array {
                name: name.to_string(),
                value,
                reply: tx,
            },
            rx,
        )?
    }

    pub fn post_string(&self, name: &str, value: &str) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::PostString {
                name: name.to_string(),
                value: value.to_string(),
                reply: tx,
            },
            rx,
        )?
    }

    pub fn post_string_array(&self, name: &str, value: Vec<String>) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::PostStringArray {
                name: name.to_string(),
                value,
                reply: tx,
            },
            rx,
        )?
    }

    pub fn post_enum(&self, name: &str, value: i16) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::PostEnum {
                name: name.to_string(),
                value,
                reply: tx,
            },
            rx,
        )?
    }

    pub fn remove_pv(&self, name: &str) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::Remove {
                name: name.to_string(),
                reply: tx,
            },
            rx,
        )?
    }

    pub fn fetch_double(&self, name: &str) -> Result<FetchedDouble> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::FetchDouble {
                name: name.to_string(),
                reply: tx,
            },
            rx,
        )?
    }

    pub fn fetch_int32(&self, name: &str) -> Result<FetchedInt32> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::FetchInt32 {
                name: name.to_string(),
                reply: tx,
            },
            rx,
        )?
    }

    pub fn fetch_string(&self, name: &str) -> Result<FetchedString> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::FetchString {
                name: name.to_string(),
                reply: tx,
            },
            rx,
        )?
    }

    pub fn fetch_double_array(&self, name: &str) -> Result<FetchedDoubleArray> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::FetchDoubleArray {
                name: name.to_string(),
                reply: tx,
            },
            rx,
        )?
    }

    pub fn fetch_int32_array(&self, name: &str) -> Result<FetchedInt32Array> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::FetchInt32Array {
                name: name.to_string(),
                reply: tx,
            },
            rx,
        )?
    }

    pub fn fetch_string_array(&self, name: &str) -> Result<FetchedStringArray> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::FetchStringArray {
                name: name.to_string(),
                reply: tx,
            },
            rx,
        )?
    }

    pub fn fetch_enum(&self, name: &str) -> Result<FetchedEnum> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::FetchEnum {
                name: name.to_string(),
                reply: tx,
            },
            rx,
        )?
    }
}

// ============================================================================
// Server
// ============================================================================

/// Pure-Rust pvAccess server with automatic alarm management.
///
/// Mirrors `pvxs-sys::Server` exactly — same method names and signatures.
/// The in-process PV registry is fully functional.
/// The pvAccess TCP/UDP transport layer is a TODO — see TODO.md.
pub struct Server {
    handle: ServerHandle,
    join: Option<thread::JoinHandle<()>>,
}

impl Server {
    /// Start a server configured from environment variables.
    ///
    /// TODO(network): no TCP/UDP port is bound yet; `tcp_port()` returns 0.
    pub fn start_from_env() -> Result<Self> {
        Self::start_inner()
    }

    /// Start an isolated server (system-assigned ports, ideal for tests).
    ///
    /// TODO(network): no TCP/UDP port is bound yet; `tcp_port()` returns 0.
    pub fn start_isolated() -> Result<Self> {
        Self::start_inner()
    }

    fn start_inner() -> Result<Self> {
        let (tx, rx) = channel::unbounded::<ManagerCommand>();
        let join = thread::spawn(move || run_worker(rx));
        Ok(Self {
            handle: ServerHandle {
                tx,
                // TODO(network): replace with real bound port
                tcp_port: 0,
                udp_port: 0,
            },
            join: Some(join),
        })
    }

    /// Get a clone-able handle to this server for use from other threads.
    pub fn handle(&self) -> ServerHandle {
        self.handle.clone()
    }

    /// TCP port the server is listening on (0 until transport layer is implemented).
    pub fn tcp_port(&self) -> u16 {
        self.handle.tcp_port()
    }

    /// UDP port the server is using (0 until transport layer is implemented).
    pub fn udp_port(&self) -> u16 {
        self.handle.udp_port()
    }

    pub fn create_pv_double(
        &self,
        name: &str,
        initial: f64,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.handle.create_pv_double(name, initial, metadata)
    }

    pub fn create_pv_double_array(
        &self,
        name: &str,
        initial: Vec<f64>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.handle.create_pv_double_array(name, initial, metadata)
    }

    pub fn create_pv_int32(
        &self,
        name: &str,
        initial: i32,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.handle.create_pv_int32(name, initial, metadata)
    }

    pub fn create_pv_int32_array(
        &self,
        name: &str,
        initial: Vec<i32>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.handle.create_pv_int32_array(name, initial, metadata)
    }

    pub fn create_pv_string(
        &self,
        name: &str,
        initial: &str,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.handle.create_pv_string(name, initial, metadata)
    }

    pub fn create_pv_string_array(
        &self,
        name: &str,
        initial: Vec<String>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.handle.create_pv_string_array(name, initial, metadata)
    }

    pub fn create_pv_enum(
        &self,
        name: &str,
        choices: Vec<&str>,
        selected_index: i16,
        metadata: NTEnumMetadataBuilder,
    ) -> Result<()> {
        self.handle
            .create_pv_enum(name, choices, selected_index, metadata)
    }

    pub fn post_double(&self, name: &str, value: f64) -> Result<()> {
        self.handle.post_double(name, value)
    }

    pub fn post_double_array(&self, name: &str, value: Vec<f64>) -> Result<()> {
        self.handle.post_double_array(name, value)
    }

    pub fn post_int32(&self, name: &str, value: i32) -> Result<()> {
        self.handle.post_int32(name, value)
    }

    pub fn post_int32_array(&self, name: &str, value: Vec<i32>) -> Result<()> {
        self.handle.post_int32_array(name, value)
    }

    pub fn post_string(&self, name: &str, value: &str) -> Result<()> {
        self.handle.post_string(name, value)
    }

    pub fn post_string_array(&self, name: &str, value: Vec<String>) -> Result<()> {
        self.handle.post_string_array(name, value)
    }

    pub fn post_enum(&self, name: &str, value: i16) -> Result<()> {
        self.handle.post_enum(name, value)
    }

    pub fn remove_pv(&self, name: &str) -> Result<()> {
        self.handle.remove_pv(name)
    }

    pub fn fetch_double(&self, name: &str) -> Result<FetchedDouble> {
        self.handle.fetch_double(name)
    }

    pub fn fetch_int32(&self, name: &str) -> Result<FetchedInt32> {
        self.handle.fetch_int32(name)
    }

    pub fn fetch_string(&self, name: &str) -> Result<FetchedString> {
        self.handle.fetch_string(name)
    }

    pub fn fetch_double_array(&self, name: &str) -> Result<FetchedDoubleArray> {
        self.handle.fetch_double_array(name)
    }

    pub fn fetch_int32_array(&self, name: &str) -> Result<FetchedInt32Array> {
        self.handle.fetch_int32_array(name)
    }

    pub fn fetch_string_array(&self, name: &str) -> Result<FetchedStringArray> {
        self.handle.fetch_string_array(name)
    }

    pub fn fetch_enum(&self, name: &str) -> Result<FetchedEnum> {
        self.handle.fetch_enum(name)
    }

    /// Stop the server, consuming it and freeing all resources.
    pub fn stop_drop(mut self) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.handle
            .tx
            .send(ManagerCommand::Stop { reply: tx })
            .map_err(|_| PvxsError::new("server worker stopped"))?;
        let result = rx
            .recv()
            .map_err(|_| PvxsError::new("server worker stopped"))?;
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
        result
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        // If stop_drop was not called, send Stop anyway so the worker exits.
        if self.join.is_some() {
            let (tx, _rx) = channel::bounded(1);
            let _ = self.handle.tx.send(ManagerCommand::Stop { reply: tx });
            if let Some(join) = self.join.take() {
                let _ = join.join();
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn server() -> Server {
        Server::start_isolated().expect("server start")
    }

    #[test]
    fn create_and_fetch_double() {
        let s = server();
        s.create_pv_double("A", 3.14, NTScalarMetadataBuilder::new())
            .unwrap();
        let f = s.fetch_double("A").unwrap();
        assert!((f.value - 3.14).abs() < 1e-9);
        assert_eq!(f.alarm_severity, AlarmSeverity::NoAlarm);
        s.stop_drop().unwrap();
    }

    #[test]
    fn post_double_updates_value() {
        let s = server();
        s.create_pv_double("B", 0.0, NTScalarMetadataBuilder::new())
            .unwrap();
        s.post_double("B", 42.0).unwrap();
        let f = s.fetch_double("B").unwrap();
        assert!((f.value - 42.0).abs() < 1e-9);
        s.stop_drop().unwrap();
    }

    #[test]
    fn duplicate_pv_name_errors() {
        let s = server();
        s.create_pv_double("C", 0.0, NTScalarMetadataBuilder::new())
            .unwrap();
        assert!(s
            .create_pv_double("C", 1.0, NTScalarMetadataBuilder::new())
            .is_err());
        s.stop_drop().unwrap();
    }

    #[test]
    fn create_and_fetch_int32() {
        let s = server();
        s.create_pv_int32("D", 7, NTScalarMetadataBuilder::new())
            .unwrap();
        let f = s.fetch_int32("D").unwrap();
        assert_eq!(f.value, 7);
        s.stop_drop().unwrap();
    }

    #[test]
    fn create_and_fetch_string() {
        let s = server();
        s.create_pv_string("E", "hello", NTScalarMetadataBuilder::new())
            .unwrap();
        let f = s.fetch_string("E").unwrap();
        assert_eq!(f.value, "hello");
        s.stop_drop().unwrap();
    }

    #[test]
    fn create_and_fetch_enum() {
        let s = server();
        s.create_pv_enum(
            "F",
            vec!["OFF", "ON"],
            1,
            NTEnumMetadataBuilder::new(),
        )
        .unwrap();
        let f = s.fetch_enum("F").unwrap();
        assert_eq!(f.value, 1);
        assert_eq!(f.value_choices, vec!["OFF", "ON"]);
        s.stop_drop().unwrap();
    }

    #[test]
    fn create_and_fetch_double_array() {
        let s = server();
        s.create_pv_double_array("G", vec![1.0, 2.0, 3.0], NTScalarMetadataBuilder::new())
            .unwrap();
        let f = s.fetch_double_array("G").unwrap();
        assert_eq!(f.value, vec![1.0, 2.0, 3.0]);
        s.stop_drop().unwrap();
    }

    #[test]
    fn remove_pv() {
        let s = server();
        s.create_pv_double("H", 0.0, NTScalarMetadataBuilder::new())
            .unwrap();
        s.remove_pv("H").unwrap();
        assert!(s.fetch_double("H").is_err());
        s.stop_drop().unwrap();
    }

    #[test]
    fn control_limit_rejection() {
        use crate::ControlMetadata;
        let s = server();
        let meta = NTScalarMetadataBuilder::new().control(ControlMetadata {
            limit_low: 0.0,
            limit_high: 10.0,
            min_step: 0.0,
        });
        s.create_pv_double("I", 5.0, meta).unwrap();
        // Value outside control limits should be rejected
        assert!(s.post_double("I", 20.0).is_err());
        s.stop_drop().unwrap();
    }

    #[test]
    fn server_handle_clone() {
        let s = server();
        let h = s.handle();
        h.create_pv_double("J", 1.0, NTScalarMetadataBuilder::new())
            .unwrap();
        let f = h.fetch_double("J").unwrap();
        assert!((f.value - 1.0).abs() < 1e-9);
        s.stop_drop().unwrap();
    }
}

// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
//! Pure-Rust pvAccess server — same public API as `pvxs-sys::Server`.
//!
//! All state is held in a worker thread via crossbeam channels.
//! The pvAccess TCP/UDP transport is a TODO — see TODO.md.

use crossbeam_channel as channel;
use std::collections::HashMap;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    compute_alarm_for_scalar, AlarmConfig, AlarmMetadata, AlarmSeverity, AlarmStatus,
    ControlMetadata, DisplayMetadata, PvxsError, Result, Value,
};

// ============================================================================
// Public metadata builders (mirror pvxs-sys exactly)
// ============================================================================

/// Builder for NTScalar / NTScalarArray PV metadata.
///
/// Mirrors `pvxs-sys::NTScalarMetadataBuilder` exactly so call sites are
/// source-compatible when swapping crates.
pub struct NTScalarMetadataBuilder {
    pub(crate) alarm_severity: AlarmSeverity,
    pub(crate) alarm_status: AlarmStatus,
    pub(crate) alarm_message: String,
    pub(crate) timestamp_seconds: i64,
    pub(crate) timestamp_nanos: i32,
    pub(crate) timestamp_user_tag: i32,
    pub(crate) display: Option<DisplayMetadata>,
    pub(crate) control: Option<ControlMetadata>,
    pub(crate) alarm_metadata: Option<AlarmMetadata>,
}

impl NTScalarMetadataBuilder {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self {
            alarm_severity: AlarmSeverity::Invalid,
            alarm_status: AlarmStatus::UndefinedStatus,
            alarm_message: String::new(),
            timestamp_seconds: now.as_secs() as i64,
            timestamp_nanos: now.subsec_nanos() as i32,
            timestamp_user_tag: 0,
            display: None,
            control: None,
            alarm_metadata: None,
        }
    }

    pub fn alarm(
        mut self,
        severity: AlarmSeverity,
        status: AlarmStatus,
        message: impl Into<String>,
    ) -> Self {
        self.alarm_severity = severity;
        self.alarm_status = status;
        self.alarm_message = message.into();
        self
    }

    pub fn timestamp(mut self, seconds: i64, nanos: i32, user_tag: i32) -> Self {
        self.timestamp_seconds = seconds;
        self.timestamp_nanos = nanos;
        self.timestamp_user_tag = user_tag;
        self
    }

    pub fn display(mut self, meta: DisplayMetadata) -> Self {
        self.display = Some(meta);
        self
    }

    pub fn control(mut self, meta: ControlMetadata) -> Self {
        self.control = Some(meta);
        self
    }

    pub fn alarm_metadata(mut self, meta: AlarmMetadata) -> Self {
        self.alarm_metadata = Some(meta);
        self
    }
}

impl Default for NTScalarMetadataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for NTEnum PV metadata.
///
/// Mirrors `pvxs-sys::NTEnumMetadataBuilder` exactly.
pub struct NTEnumMetadataBuilder {
    pub(crate) alarm_severity: i32,
    pub(crate) alarm_status: i32,
    pub(crate) alarm_message: String,
    pub(crate) timestamp_seconds: i64,
    pub(crate) timestamp_nanos: i32,
    pub(crate) timestamp_user_tag: i32,
}

impl NTEnumMetadataBuilder {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self {
            alarm_severity: 0,
            alarm_status: 0,
            alarm_message: String::new(),
            timestamp_seconds: now.as_secs() as i64,
            timestamp_nanos: now.subsec_nanos() as i32,
            timestamp_user_tag: 0,
        }
    }

    pub fn alarm(mut self, severity: i32, status: i32, message: impl Into<String>) -> Self {
        self.alarm_severity = severity;
        self.alarm_status = status;
        self.alarm_message = message.into();
        self
    }

    pub fn timestamp(mut self, seconds: i64, nanos: i32, user_tag: i32) -> Self {
        self.timestamp_seconds = seconds;
        self.timestamp_nanos = nanos;
        self.timestamp_user_tag = user_tag;
        self
    }
}

impl Default for NTEnumMetadataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

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
// In-process PV store
// ============================================================================

/// In-memory state for a single managed PV.
enum ManagedPvState {
    Double {
        value: f64,
        alarm_config: AlarmConfig,
        alarm_severity: AlarmSeverity,
        alarm_status: AlarmStatus,
        alarm_message: String,
        display: Option<DisplayMetadata>,
        control: Option<ControlMetadata>,
        alarm_meta: Option<AlarmMetadata>,
    },
    DoubleArray {
        value: Vec<f64>,
        alarm_severity: AlarmSeverity,
        alarm_status: AlarmStatus,
        alarm_message: String,
        display: Option<DisplayMetadata>,
        control: Option<ControlMetadata>,
        alarm_meta: Option<AlarmMetadata>,
    },
    Int32 {
        value: i32,
        alarm_config: AlarmConfig,
        alarm_severity: AlarmSeverity,
        alarm_status: AlarmStatus,
        alarm_message: String,
        display: Option<DisplayMetadata>,
        control: Option<ControlMetadata>,
        alarm_meta: Option<AlarmMetadata>,
    },
    Int32Array {
        value: Vec<i32>,
        alarm_severity: AlarmSeverity,
        alarm_status: AlarmStatus,
        alarm_message: String,
        display: Option<DisplayMetadata>,
        control: Option<ControlMetadata>,
        alarm_meta: Option<AlarmMetadata>,
    },
    Str {
        value: String,
        alarm_severity: AlarmSeverity,
        alarm_status: AlarmStatus,
        alarm_message: String,
    },
    StrArray {
        value: Vec<String>,
        alarm_severity: AlarmSeverity,
        alarm_status: AlarmStatus,
        alarm_message: String,
    },
    Enum {
        value: i16,
        choices: Vec<String>,
        alarm_severity: AlarmSeverity,
        alarm_status: AlarmStatus,
        alarm_message: String,
    },
}

fn alarm_config_from_builder(b: &NTScalarMetadataBuilder) -> AlarmConfig {
    AlarmConfig {
        control: b.control.clone(),
        alarm_metadata: b.alarm_metadata.clone(),
    }
}

// ============================================================================
// Worker commands
// ============================================================================

enum ManagerCommand {
    CreateDouble {
        name: String,
        initial: f64,
        metadata: NTScalarMetadataBuilder,
        reply: channel::Sender<Result<()>>,
    },
    CreateDoubleArray {
        name: String,
        initial: Vec<f64>,
        metadata: NTScalarMetadataBuilder,
        reply: channel::Sender<Result<()>>,
    },
    CreateInt32 {
        name: String,
        initial: i32,
        metadata: NTScalarMetadataBuilder,
        reply: channel::Sender<Result<()>>,
    },
    CreateInt32Array {
        name: String,
        initial: Vec<i32>,
        metadata: NTScalarMetadataBuilder,
        reply: channel::Sender<Result<()>>,
    },
    CreateString {
        name: String,
        initial: String,
        metadata: NTScalarMetadataBuilder,
        reply: channel::Sender<Result<()>>,
    },
    CreateStringArray {
        name: String,
        initial: Vec<String>,
        metadata: NTScalarMetadataBuilder,
        reply: channel::Sender<Result<()>>,
    },
    CreateEnum {
        name: String,
        choices: Vec<String>,
        selected_index: i16,
        metadata: NTEnumMetadataBuilder,
        reply: channel::Sender<Result<()>>,
    },
    PostDouble {
        name: String,
        value: f64,
        reply: channel::Sender<Result<()>>,
    },
    PostDoubleArray {
        name: String,
        value: Vec<f64>,
        reply: channel::Sender<Result<()>>,
    },
    PostInt32 {
        name: String,
        value: i32,
        reply: channel::Sender<Result<()>>,
    },
    PostInt32Array {
        name: String,
        value: Vec<i32>,
        reply: channel::Sender<Result<()>>,
    },
    PostString {
        name: String,
        value: String,
        reply: channel::Sender<Result<()>>,
    },
    PostStringArray {
        name: String,
        value: Vec<String>,
        reply: channel::Sender<Result<()>>,
    },
    PostEnum {
        name: String,
        value: i16,
        reply: channel::Sender<Result<()>>,
    },
    Remove {
        name: String,
        reply: channel::Sender<Result<()>>,
    },
    FetchDouble {
        name: String,
        reply: channel::Sender<Result<FetchedDouble>>,
    },
    FetchInt32 {
        name: String,
        reply: channel::Sender<Result<FetchedInt32>>,
    },
    FetchString {
        name: String,
        reply: channel::Sender<Result<FetchedString>>,
    },
    FetchDoubleArray {
        name: String,
        reply: channel::Sender<Result<FetchedDoubleArray>>,
    },
    FetchInt32Array {
        name: String,
        reply: channel::Sender<Result<FetchedInt32Array>>,
    },
    FetchStringArray {
        name: String,
        reply: channel::Sender<Result<FetchedStringArray>>,
    },
    FetchEnum {
        name: String,
        reply: channel::Sender<Result<FetchedEnum>>,
    },
    Stop {
        reply: channel::Sender<Result<()>>,
    },
}

// ============================================================================
// Worker loop
// ============================================================================

fn run_worker(rx: channel::Receiver<ManagerCommand>) {
    let mut pvs: HashMap<String, ManagedPvState> = HashMap::new();

    while let Ok(cmd) = rx.recv() {
        match cmd {
            // ── Create ──────────────────────────────────────────────────────

            ManagerCommand::CreateDouble {
                name,
                initial,
                metadata,
                reply,
            } => {
                let result = if pvs.contains_key(&name) {
                    Err(PvxsError::new(format!("PV '{}' already exists", name)))
                } else {
                    let alarm_config = alarm_config_from_builder(&metadata);
                    let ar = compute_alarm_for_scalar(initial, &alarm_config);
                    pvs.insert(
                        name,
                        ManagedPvState::Double {
                            value: initial,
                            alarm_config,
                            alarm_severity: ar.severity,
                            alarm_status: ar.status,
                            alarm_message: ar.message,
                            display: metadata.display,
                            control: metadata.control,
                            alarm_meta: metadata.alarm_metadata,
                        },
                    );
                    Ok(())
                };
                let _ = reply.send(result);
            }

            ManagerCommand::CreateDoubleArray {
                name,
                initial,
                metadata,
                reply,
            } => {
                let result = if pvs.contains_key(&name) {
                    Err(PvxsError::new(format!("PV '{}' already exists", name)))
                } else if initial.is_empty() {
                    Err(PvxsError::new("Initial double array cannot be empty"))
                } else {
                    pvs.insert(
                        name,
                        ManagedPvState::DoubleArray {
                            value: initial,
                            alarm_severity: AlarmSeverity::NoAlarm,
                            alarm_status: AlarmStatus::NoAlarm,
                            alarm_message: "OK".to_string(),
                            display: metadata.display,
                            control: metadata.control,
                            alarm_meta: metadata.alarm_metadata,
                        },
                    );
                    Ok(())
                };
                let _ = reply.send(result);
            }

            ManagerCommand::CreateInt32 {
                name,
                initial,
                metadata,
                reply,
            } => {
                let result = if pvs.contains_key(&name) {
                    Err(PvxsError::new(format!("PV '{}' already exists", name)))
                } else {
                    let alarm_config = alarm_config_from_builder(&metadata);
                    let ar = compute_alarm_for_scalar(initial as f64, &alarm_config);
                    pvs.insert(
                        name,
                        ManagedPvState::Int32 {
                            value: initial,
                            alarm_config,
                            alarm_severity: ar.severity,
                            alarm_status: ar.status,
                            alarm_message: ar.message,
                            display: metadata.display,
                            control: metadata.control,
                            alarm_meta: metadata.alarm_metadata,
                        },
                    );
                    Ok(())
                };
                let _ = reply.send(result);
            }

            ManagerCommand::CreateInt32Array {
                name,
                initial,
                metadata,
                reply,
            } => {
                let result = if pvs.contains_key(&name) {
                    Err(PvxsError::new(format!("PV '{}' already exists", name)))
                } else if initial.is_empty() {
                    Err(PvxsError::new("Initial int32 array cannot be empty"))
                } else {
                    pvs.insert(
                        name,
                        ManagedPvState::Int32Array {
                            value: initial,
                            alarm_severity: AlarmSeverity::NoAlarm,
                            alarm_status: AlarmStatus::NoAlarm,
                            alarm_message: "OK".to_string(),
                            display: metadata.display,
                            control: metadata.control,
                            alarm_meta: metadata.alarm_metadata,
                        },
                    );
                    Ok(())
                };
                let _ = reply.send(result);
            }

            ManagerCommand::CreateString {
                name,
                initial,
                metadata: _,
                reply,
            } => {
                let result = if pvs.contains_key(&name) {
                    Err(PvxsError::new(format!("PV '{}' already exists", name)))
                } else {
                    pvs.insert(
                        name,
                        ManagedPvState::Str {
                            value: initial,
                            alarm_severity: AlarmSeverity::NoAlarm,
                            alarm_status: AlarmStatus::NoAlarm,
                            alarm_message: "OK".to_string(),
                        },
                    );
                    Ok(())
                };
                let _ = reply.send(result);
            }

            ManagerCommand::CreateStringArray {
                name,
                initial,
                metadata: _,
                reply,
            } => {
                let result = if pvs.contains_key(&name) {
                    Err(PvxsError::new(format!("PV '{}' already exists", name)))
                } else if initial.is_empty() {
                    Err(PvxsError::new("Initial string array cannot be empty"))
                } else {
                    pvs.insert(
                        name,
                        ManagedPvState::StrArray {
                            value: initial,
                            alarm_severity: AlarmSeverity::NoAlarm,
                            alarm_status: AlarmStatus::NoAlarm,
                            alarm_message: "OK".to_string(),
                        },
                    );
                    Ok(())
                };
                let _ = reply.send(result);
            }

            ManagerCommand::CreateEnum {
                name,
                choices,
                selected_index,
                metadata: _,
                reply,
            } => {
                let result = if pvs.contains_key(&name) {
                    Err(PvxsError::new(format!("PV '{}' already exists", name)))
                } else if choices.is_empty() {
                    Err(PvxsError::new("Enum choices cannot be empty"))
                } else if selected_index as usize >= choices.len() {
                    Err(PvxsError::new("selected_index out of range"))
                } else {
                    pvs.insert(
                        name,
                        ManagedPvState::Enum {
                            value: selected_index,
                            choices,
                            alarm_severity: AlarmSeverity::NoAlarm,
                            alarm_status: AlarmStatus::NoAlarm,
                            alarm_message: "OK".to_string(),
                        },
                    );
                    Ok(())
                };
                let _ = reply.send(result);
            }

            // ── Post ────────────────────────────────────────────────────────

            ManagerCommand::PostDouble { name, value, reply } => {
                let result = match pvs.get_mut(&name) {
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
                    Some(ManagedPvState::Double {
                        value: stored,
                        alarm_config,
                        alarm_severity,
                        alarm_status,
                        alarm_message,
                        ..
                    }) => {
                        let ar = compute_alarm_for_scalar(value, alarm_config);
                        if !ar.allow {
                            Err(PvxsError::new(ar.message))
                        } else {
                            *stored = value;
                            *alarm_severity = ar.severity;
                            *alarm_status = ar.status;
                            *alarm_message = ar.message;
                            Ok(())
                        }
                    }
                    Some(_) => Err(PvxsError::new(format!("PV '{}' is not a double", name))),
                };
                let _ = reply.send(result);
            }

            ManagerCommand::PostDoubleArray { name, value, reply } => {
                let result = match pvs.get_mut(&name) {
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
                    Some(ManagedPvState::DoubleArray { value: stored, .. }) => {
                        *stored = value;
                        Ok(())
                    }
                    Some(_) => Err(PvxsError::new(format!(
                        "PV '{}' is not a double array",
                        name
                    ))),
                };
                let _ = reply.send(result);
            }

            ManagerCommand::PostInt32 { name, value, reply } => {
                let result = match pvs.get_mut(&name) {
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
                    Some(ManagedPvState::Int32 {
                        value: stored,
                        alarm_config,
                        alarm_severity,
                        alarm_status,
                        alarm_message,
                        ..
                    }) => {
                        let ar = compute_alarm_for_scalar(value as f64, alarm_config);
                        if !ar.allow {
                            Err(PvxsError::new(ar.message))
                        } else {
                            *stored = value;
                            *alarm_severity = ar.severity;
                            *alarm_status = ar.status;
                            *alarm_message = ar.message;
                            Ok(())
                        }
                    }
                    Some(_) => Err(PvxsError::new(format!("PV '{}' is not an int32", name))),
                };
                let _ = reply.send(result);
            }

            ManagerCommand::PostInt32Array { name, value, reply } => {
                let result = match pvs.get_mut(&name) {
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
                    Some(ManagedPvState::Int32Array { value: stored, .. }) => {
                        *stored = value;
                        Ok(())
                    }
                    Some(_) => Err(PvxsError::new(format!(
                        "PV '{}' is not an int32 array",
                        name
                    ))),
                };
                let _ = reply.send(result);
            }

            ManagerCommand::PostString { name, value, reply } => {
                let result = match pvs.get_mut(&name) {
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
                    Some(ManagedPvState::Str { value: stored, .. }) => {
                        *stored = value;
                        Ok(())
                    }
                    Some(_) => Err(PvxsError::new(format!("PV '{}' is not a string", name))),
                };
                let _ = reply.send(result);
            }

            ManagerCommand::PostStringArray { name, value, reply } => {
                let result = match pvs.get_mut(&name) {
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
                    Some(ManagedPvState::StrArray { value: stored, .. }) => {
                        *stored = value;
                        Ok(())
                    }
                    Some(_) => Err(PvxsError::new(format!(
                        "PV '{}' is not a string array",
                        name
                    ))),
                };
                let _ = reply.send(result);
            }

            ManagerCommand::PostEnum { name, value, reply } => {
                let result = match pvs.get_mut(&name) {
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
                    Some(ManagedPvState::Enum {
                        value: stored,
                        choices,
                        ..
                    }) => {
                        if value as usize >= choices.len() {
                            Err(PvxsError::new("enum index out of range"))
                        } else {
                            *stored = value;
                            Ok(())
                        }
                    }
                    Some(_) => Err(PvxsError::new(format!("PV '{}' is not an enum", name))),
                };
                let _ = reply.send(result);
            }

            // ── Remove ──────────────────────────────────────────────────────

            ManagerCommand::Remove { name, reply } => {
                let result = if pvs.remove(&name).is_some() {
                    Ok(())
                } else {
                    Err(PvxsError::new(format!("PV '{}' not found", name)))
                };
                let _ = reply.send(result);
            }

            // ── Fetch ───────────────────────────────────────────────────────

            ManagerCommand::FetchDouble { name, reply } => {
                let result = match pvs.get(&name) {
                    Some(ManagedPvState::Double {
                        value,
                        alarm_severity,
                        alarm_status,
                        alarm_message,
                        display,
                        control,
                        alarm_meta,
                        ..
                    }) => Ok(FetchedDouble {
                        value: *value,
                        alarm_severity: *alarm_severity,
                        alarm_status: *alarm_status,
                        alarm_message: alarm_message.clone(),
                        display_metadata: display.clone(),
                        control_metadata: control.clone(),
                        alarm_metadata: alarm_meta.clone(),
                    }),
                    Some(_) => Err(PvxsError::new(format!("PV '{}' is not a double", name))),
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
                };
                let _ = reply.send(result);
            }

            ManagerCommand::FetchInt32 { name, reply } => {
                let result = match pvs.get(&name) {
                    Some(ManagedPvState::Int32 {
                        value,
                        alarm_severity,
                        alarm_status,
                        alarm_message,
                        display,
                        control,
                        alarm_meta,
                        ..
                    }) => Ok(FetchedInt32 {
                        value: *value,
                        alarm_severity: *alarm_severity,
                        alarm_status: *alarm_status,
                        alarm_message: alarm_message.clone(),
                        display_metadata: display.clone(),
                        control_metadata: control.clone(),
                        alarm_metadata: alarm_meta.clone(),
                    }),
                    Some(_) => Err(PvxsError::new(format!("PV '{}' is not an int32", name))),
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
                };
                let _ = reply.send(result);
            }

            ManagerCommand::FetchString { name, reply } => {
                let result = match pvs.get(&name) {
                    Some(ManagedPvState::Str {
                        value,
                        alarm_severity,
                        alarm_status,
                        alarm_message,
                    }) => Ok(FetchedString {
                        value: value.clone(),
                        alarm_severity: *alarm_severity,
                        alarm_status: *alarm_status,
                        alarm_message: alarm_message.clone(),
                    }),
                    Some(_) => Err(PvxsError::new(format!("PV '{}' is not a string", name))),
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
                };
                let _ = reply.send(result);
            }

            ManagerCommand::FetchDoubleArray { name, reply } => {
                let result = match pvs.get(&name) {
                    Some(ManagedPvState::DoubleArray {
                        value,
                        alarm_severity,
                        alarm_status,
                        alarm_message,
                        display,
                        control,
                        alarm_meta,
                    }) => Ok(FetchedDoubleArray {
                        value: value.clone(),
                        alarm_severity: *alarm_severity,
                        alarm_status: *alarm_status,
                        alarm_message: alarm_message.clone(),
                        display_metadata: display.clone(),
                        control_metadata: control.clone(),
                        alarm_metadata: alarm_meta.clone(),
                    }),
                    Some(_) => Err(PvxsError::new(format!(
                        "PV '{}' is not a double array",
                        name
                    ))),
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
                };
                let _ = reply.send(result);
            }

            ManagerCommand::FetchInt32Array { name, reply } => {
                let result = match pvs.get(&name) {
                    Some(ManagedPvState::Int32Array {
                        value,
                        alarm_severity,
                        alarm_status,
                        alarm_message,
                        display,
                        control,
                        alarm_meta,
                    }) => Ok(FetchedInt32Array {
                        value: value.clone(),
                        alarm_severity: *alarm_severity,
                        alarm_status: *alarm_status,
                        alarm_message: alarm_message.clone(),
                        display_metadata: display.clone(),
                        control_metadata: control.clone(),
                        alarm_metadata: alarm_meta.clone(),
                    }),
                    Some(_) => Err(PvxsError::new(format!(
                        "PV '{}' is not an int32 array",
                        name
                    ))),
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
                };
                let _ = reply.send(result);
            }

            ManagerCommand::FetchStringArray { name, reply } => {
                let result = match pvs.get(&name) {
                    Some(ManagedPvState::StrArray {
                        value,
                        alarm_severity,
                        alarm_status,
                        alarm_message,
                    }) => Ok(FetchedStringArray {
                        value: value.clone(),
                        alarm_severity: *alarm_severity,
                        alarm_status: *alarm_status,
                        alarm_message: alarm_message.clone(),
                    }),
                    Some(_) => Err(PvxsError::new(format!(
                        "PV '{}' is not a string array",
                        name
                    ))),
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
                };
                let _ = reply.send(result);
            }

            ManagerCommand::FetchEnum { name, reply } => {
                let result = match pvs.get(&name) {
                    Some(ManagedPvState::Enum {
                        value,
                        choices,
                        alarm_severity,
                        alarm_status,
                        alarm_message,
                    }) => Ok(FetchedEnum {
                        value: *value,
                        value_choices: choices.clone(),
                        alarm_severity: *alarm_severity,
                        alarm_status: *alarm_status,
                        alarm_message: alarm_message.clone(),
                    }),
                    Some(_) => Err(PvxsError::new(format!("PV '{}' is not an enum", name))),
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
                };
                let _ = reply.send(result);
            }

            // ── Stop ────────────────────────────────────────────────────────

            ManagerCommand::Stop { reply } => {
                pvs.clear();
                let _ = reply.send(Ok(()));
                return;
            }
        }
    }
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

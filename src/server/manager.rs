use crate::{PvxsError, Result};

use crate::alarms::{AlarmSeverity, AlarmStatus, AlarmConfig,
    compute_alarm_for_scalar};
use crate::metadata::{AlarmMetadata, ControlMetadata, DisplayMetadata};
use crate::{NTScalarMetadataBuilder, NTEnumMetadataBuilder, 
    FetchedDouble, FetchedInt32, FetchedString, FetchedDoubleArray, 
    FetchedInt32Array, FetchedStringArray, FetchedEnum};
use crossbeam_channel as channel;
use std::collections::HashMap;

/// In-memory state for a single managed PV.
pub(super) enum ManagedPvState {
    Double {
        readonly: bool,
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
        readonly: bool,
        value: Vec<f64>,
        alarm_severity: AlarmSeverity,
        alarm_status: AlarmStatus,
        alarm_message: String,
        display: Option<DisplayMetadata>,
        control: Option<ControlMetadata>,
        alarm_meta: Option<AlarmMetadata>,
    },
    Int32 {
        readonly: bool,
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
        readonly: bool,
        value: Vec<i32>,
        alarm_severity: AlarmSeverity,
        alarm_status: AlarmStatus,
        alarm_message: String,
        display: Option<DisplayMetadata>,
        control: Option<ControlMetadata>,
        alarm_meta: Option<AlarmMetadata>,
    },
    Str {
        readonly: bool,
        value: String,
        alarm_severity: AlarmSeverity,
        alarm_status: AlarmStatus,
        alarm_message: String,
    },
    StrArray {
        readonly: bool,
        value: Vec<String>,
        alarm_severity: AlarmSeverity,
        alarm_status: AlarmStatus,
        alarm_message: String,
    },
    Enum {
        readonly: bool,
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

/// Internal worker commands for the in-memory PV registry.
pub enum ManagerCommand {
    /// Create a double PV with the provided initial value.
    CreateDouble {
        /// PV name.
        name: String,
        /// Initial value.
        initial: f64,
        /// Scalar metadata builder.
        metadata: NTScalarMetadataBuilder,
        /// Worker reply channel.
        reply: channel::Sender<Result<()>>,
    },
    /// Create a double-array PV with the provided initial value.
    CreateDoubleArray {
        /// PV name.
        name: String,
        /// Initial value.
        initial: Vec<f64>,
        /// Scalar metadata builder.
        metadata: NTScalarMetadataBuilder,
        /// Worker reply channel.
        reply: channel::Sender<Result<()>>,
    },
    /// Create an int32 PV with the provided initial value.
    CreateInt32 {
        /// PV name.
        name: String,
        /// Initial value.
        initial: i32,
        /// Scalar metadata builder.
        metadata: NTScalarMetadataBuilder,
        /// Worker reply channel.
        reply: channel::Sender<Result<()>>,
    },
    /// Create an int32-array PV with the provided initial value.
    CreateInt32Array {
        /// PV name.
        name: String,
        /// Initial value.
        initial: Vec<i32>,
        /// Scalar metadata builder.
        metadata: NTScalarMetadataBuilder,
        /// Worker reply channel.
        reply: channel::Sender<Result<()>>,
    },
    /// Create a string PV with the provided initial value.
    CreateString {
        /// PV name.
        name: String,
        /// Initial value.
        initial: String,
        /// Scalar metadata builder.
        metadata: NTScalarMetadataBuilder,
        /// Worker reply channel.
        reply: channel::Sender<Result<()>>,
    },
    /// Create a string-array PV with the provided initial value.
    CreateStringArray {
        /// PV name.
        name: String,
        /// Initial value.
        initial: Vec<String>,
        /// Scalar metadata builder.
        metadata: NTScalarMetadataBuilder,
        /// Worker reply channel.
        reply: channel::Sender<Result<()>>,
    },
    /// Create an enum PV with the provided choices and selected index.
    CreateEnum {
        /// PV name.
        name: String,
        /// Available enum choices.
        choices: Vec<String>,
        /// Selected enum index.
        selected_index: i16,
        /// Enum metadata builder.
        metadata: NTEnumMetadataBuilder,
        /// Worker reply channel.
        reply: channel::Sender<Result<()>>,
    },
    /// Post a new value to an existing double PV.
    PostDouble {
        /// PV name.
        name: String,
        /// New double value.
        value: f64,
        /// Worker reply channel.
        reply: channel::Sender<Result<()>>,
    },
    /// Post a new value to an existing double-array PV.
    PostDoubleArray {
        /// PV name.
        name: String,
        /// New double-array value.
        value: Vec<f64>,
        /// Worker reply channel.
        reply: channel::Sender<Result<()>>,
    },
    /// Post a new value to an existing int32 PV.
    PostInt32 {
        /// PV name.
        name: String,
        /// New int32 value.
        value: i32,
        /// Worker reply channel.
        reply: channel::Sender<Result<()>>,
    },
    /// Post a new value to an existing int32-array PV.
    PostInt32Array {
        /// PV name.
        name: String,
        /// New int32-array value.
        value: Vec<i32>,
        /// Worker reply channel.
        reply: channel::Sender<Result<()>>,
    },
    /// Post a new value to an existing string PV.
    PostString {
        /// PV name.
        name: String,
        /// New string value.
        value: String,
        /// Worker reply channel.
        reply: channel::Sender<Result<()>>,
    },
    /// Post a new value to an existing string-array PV.
    PostStringArray {
        /// PV name.
        name: String,
        /// New string-array value.
        value: Vec<String>,
        /// Worker reply channel.
        reply: channel::Sender<Result<()>>,
    },
    /// Post a new value to an existing enum PV.
    PostEnum {
        /// PV name.
        name: String,
        /// New enum index.
        value: i16,
        /// Worker reply channel.
        reply: channel::Sender<Result<()>>,
    },
    /// Remove an existing PV from the registry.
    Remove {
        /// PV name.
        name: String,
        /// Worker reply channel.
        reply: channel::Sender<Result<()>>,
    },
    /// Toggle the readonly flag for an existing PV.
    SetReadonly {
        /// PV name.
        name: String,
        /// Whether the PV should become readonly.
        readonly: bool,
        /// Worker reply channel.
        reply: channel::Sender<Result<()>>,
    },
    /// Fetch the current value of a double PV.
    FetchDouble {
        /// PV name.
        name: String,
        /// Worker reply channel.
        reply: channel::Sender<Result<FetchedDouble>>,
    },
    /// Fetch the current value of an int32 PV.
    FetchInt32 {
        /// PV name.
        name: String,
        /// Worker reply channel.
        reply: channel::Sender<Result<FetchedInt32>>,
    },
    /// Fetch the current value of a string PV.
    FetchString {
        /// PV name.
        name: String,
        /// Worker reply channel.
        reply: channel::Sender<Result<FetchedString>>,
    },
    /// Fetch the current value of a double-array PV.
    FetchDoubleArray {
        /// PV name.
        name: String,
        /// Worker reply channel.
        reply: channel::Sender<Result<FetchedDoubleArray>>,
    },
    /// Fetch the current value of an int32-array PV.
    FetchInt32Array {
        /// PV name.
        name: String,
        /// Worker reply channel.
        reply: channel::Sender<Result<FetchedInt32Array>>,
    },
    /// Fetch the current value of a string-array PV.
    FetchStringArray {
        /// PV name.
        name: String,
        /// Worker reply channel.
        reply: channel::Sender<Result<FetchedStringArray>>,
    },
    /// Fetch the current value of an enum PV.
    FetchEnum {
        /// PV name.
        name: String,
        /// Worker reply channel.
        reply: channel::Sender<Result<FetchedEnum>>,
    },
    /// Stop the worker loop and clear the registry.
    Stop {
        /// Worker reply channel.
        reply: channel::Sender<Result<()>>,
    },
}

// ============================================================================
// Worker loop
// ============================================================================

/// Run the in-memory PV registry worker loop.
pub fn run_worker(rx: channel::Receiver<ManagerCommand>) {
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
                let result = match pvs.entry(name.clone()) {
                    std::collections::hash_map::Entry::Occupied(_) => {
                        Err(PvxsError::new(format!("PV '{}' already exists", name)))
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        let alarm_config = alarm_config_from_builder(&metadata);
                        let ar = compute_alarm_for_scalar(initial, &alarm_config);
                        entry.insert(ManagedPvState::Double {
                            readonly: false,
                            value: initial,
                            alarm_config,
                            alarm_severity: ar.severity,
                            alarm_status: ar.status,
                            alarm_message: ar.message,
                            display: metadata.display,
                            control: metadata.control,
                            alarm_meta: metadata.alarm_metadata,
                        });
                        Ok(())
                    }
                };
                let _ = reply.send(result);
            }

            ManagerCommand::CreateDoubleArray {
                name,
                initial,
                metadata,
                reply,
            } => {
                let result = match pvs.entry(name.clone()) {
                    std::collections::hash_map::Entry::Occupied(_) => {
                        Err(PvxsError::new(format!("PV '{}' already exists", name)))
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        if initial.is_empty() {
                            Err(PvxsError::new("Initial double array cannot be empty"))
                        } else {
                            entry.insert(ManagedPvState::DoubleArray {
                                readonly: false,
                                value: initial,
                                alarm_severity: AlarmSeverity::NoAlarm,
                                alarm_status: AlarmStatus::NoAlarm,
                                alarm_message: "OK".to_string(),
                                display: metadata.display,
                                control: metadata.control,
                                alarm_meta: metadata.alarm_metadata,
                            });
                            Ok(())
                        }
                    }
                };
                let _ = reply.send(result);
            }

            ManagerCommand::CreateInt32 {
                name,
                initial,
                metadata,
                reply,
            } => {
                let result = match pvs.entry(name.clone()) {
                    std::collections::hash_map::Entry::Occupied(_) => {
                        Err(PvxsError::new(format!("PV '{}' already exists", name)))
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        let alarm_config = alarm_config_from_builder(&metadata);
                        let ar = compute_alarm_for_scalar(initial as f64, &alarm_config);
                        entry.insert(ManagedPvState::Int32 {
                            readonly: false,
                            value: initial,
                            alarm_config,
                            alarm_severity: ar.severity,
                            alarm_status: ar.status,
                            alarm_message: ar.message,
                            display: metadata.display,
                            control: metadata.control,
                            alarm_meta: metadata.alarm_metadata,
                        });
                        Ok(())
                    }
                };
                let _ = reply.send(result);
            }

            ManagerCommand::CreateInt32Array {
                name,
                initial,
                metadata,
                reply,
            } => {
                let result = match pvs.entry(name.clone()) {
                    std::collections::hash_map::Entry::Occupied(_) => {
                        Err(PvxsError::new(format!("PV '{}' already exists", name)))
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        if initial.is_empty() {
                            Err(PvxsError::new("Initial int32 array cannot be empty"))
                        } else {
                            entry.insert(ManagedPvState::Int32Array {
                                readonly: false,
                                value: initial,
                                alarm_severity: AlarmSeverity::NoAlarm,
                                alarm_status: AlarmStatus::NoAlarm,
                                alarm_message: "OK".to_string(),
                                display: metadata.display,
                                control: metadata.control,
                                alarm_meta: metadata.alarm_metadata,
                            });
                            Ok(())
                        }
                    }
                };
                let _ = reply.send(result);
            }

            ManagerCommand::CreateString {
                name,
                initial,
                metadata: _,
                reply,
            } => {
                let result = match pvs.entry(name.clone()) {
                    std::collections::hash_map::Entry::Occupied(_) => {
                        Err(PvxsError::new(format!("PV '{}' already exists", name)))
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(ManagedPvState::Str {
                            readonly: false,
                            value: initial,
                            alarm_severity: AlarmSeverity::NoAlarm,
                            alarm_status: AlarmStatus::NoAlarm,
                            alarm_message: "OK".to_string(),
                        });
                        Ok(())
                    }
                };
                let _ = reply.send(result);
            }

            ManagerCommand::CreateStringArray {
                name,
                initial,
                metadata: _,
                reply,
            } => {
                let result = match pvs.entry(name.clone()) {
                    std::collections::hash_map::Entry::Occupied(_) => {
                        Err(PvxsError::new(format!("PV '{}' already exists", name)))
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        if initial.is_empty() {
                            Err(PvxsError::new("Initial string array cannot be empty"))
                        } else {
                            entry.insert(ManagedPvState::StrArray {
                                readonly: false,
                                value: initial,
                                alarm_severity: AlarmSeverity::NoAlarm,
                                alarm_status: AlarmStatus::NoAlarm,
                                alarm_message: "OK".to_string(),
                            });
                            Ok(())
                        }
                    }
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
                let result = match pvs.entry(name.clone()) {
                    std::collections::hash_map::Entry::Occupied(_) => {
                        Err(PvxsError::new(format!("PV '{}' already exists", name)))
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        if choices.is_empty() {
                            Err(PvxsError::new("Enum choices cannot be empty"))
                        } else if selected_index as usize >= choices.len() {
                            Err(PvxsError::new("selected_index out of range"))
                        } else {
                            entry.insert(ManagedPvState::Enum {
                                readonly: false,
                                value: selected_index,
                                choices,
                                alarm_severity: AlarmSeverity::NoAlarm,
                                alarm_status: AlarmStatus::NoAlarm,
                                alarm_message: "OK".to_string(),
                            });
                            Ok(())
                        }
                    }
                };
                let _ = reply.send(result);
            }

            // ── Post ────────────────────────────────────────────────────────

            ManagerCommand::PostDouble { name, value, reply } => {
                let result = match pvs.get_mut(&name) {
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
                    Some(ManagedPvState::Double {
                        readonly,
                        value: stored,
                        alarm_config,
                        alarm_severity,
                        alarm_status,
                        alarm_message,
                        ..
                    }) => {
                        if *readonly {
                            Err(PvxsError::new(format!("PV '{}' is readonly", name)))
                        } else {
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
                    }
                    Some(_) => Err(PvxsError::new(format!("PV '{}' is not a double", name))),
                };
                let _ = reply.send(result);
            }

            ManagerCommand::PostDoubleArray { name, value, reply } => {
                let result = match pvs.get_mut(&name) {
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
                    Some(ManagedPvState::DoubleArray {
                        readonly,
                        value: stored,
                        ..
                    }) => {
                        if *readonly {
                            Err(PvxsError::new(format!("PV '{}' is readonly", name)))
                        } else {
                            *stored = value;
                            Ok(())
                        }
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
                        readonly,
                        value: stored,
                        alarm_config,
                        alarm_severity,
                        alarm_status,
                        alarm_message,
                        ..
                    }) => {
                        if *readonly {
                            Err(PvxsError::new(format!("PV '{}' is readonly", name)))
                        } else {
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
                    }
                    Some(_) => Err(PvxsError::new(format!("PV '{}' is not an int32", name))),
                };
                let _ = reply.send(result);
            }

            ManagerCommand::PostInt32Array { name, value, reply } => {
                let result = match pvs.get_mut(&name) {
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
                    Some(ManagedPvState::Int32Array {
                        readonly,
                        value: stored,
                        ..
                    }) => {
                        if *readonly {
                            Err(PvxsError::new(format!("PV '{}' is readonly", name)))
                        } else {
                            *stored = value;
                            Ok(())
                        }
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
                    Some(ManagedPvState::Str {
                        readonly,
                        value: stored,
                        ..
                    }) => {
                        if *readonly {
                            Err(PvxsError::new(format!("PV '{}' is readonly", name)))
                        } else {
                            *stored = value;
                            Ok(())
                        }
                    }
                    Some(_) => Err(PvxsError::new(format!("PV '{}' is not a string", name))),
                };
                let _ = reply.send(result);
            }

            ManagerCommand::PostStringArray { name, value, reply } => {
                let result = match pvs.get_mut(&name) {
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
                    Some(ManagedPvState::StrArray {
                        readonly,
                        value: stored,
                        ..
                    }) => {
                        if *readonly {
                            Err(PvxsError::new(format!("PV '{}' is readonly", name)))
                        } else {
                            *stored = value;
                            Ok(())
                        }
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
                        readonly,
                        value: stored,
                        choices,
                        ..
                    }) => {
                        if *readonly {
                            Err(PvxsError::new(format!("PV '{}' is readonly", name)))
                        } else if value as usize >= choices.len() {
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

            ManagerCommand::SetReadonly {
                name,
                readonly,
                reply,
            } => {
                let result = match pvs.get_mut(&name) {
                    Some(ManagedPvState::Double { readonly: r, .. }) => {
                        *r = readonly;
                        Ok(())
                    }
                    Some(ManagedPvState::DoubleArray { readonly: r, .. }) => {
                        *r = readonly;
                        Ok(())
                    }
                    Some(ManagedPvState::Int32 { readonly: r, .. }) => {
                        *r = readonly;
                        Ok(())
                    }
                    Some(ManagedPvState::Int32Array { readonly: r, .. }) => {
                        *r = readonly;
                        Ok(())
                    }
                    Some(ManagedPvState::Str { readonly: r, .. }) => {
                        *r = readonly;
                        Ok(())
                    }
                    Some(ManagedPvState::StrArray { readonly: r, .. }) => {
                        *r = readonly;
                        Ok(())
                    }
                    Some(ManagedPvState::Enum { readonly: r, .. }) => {
                        *r = readonly;
                        Ok(())
                    }
                    None => Err(PvxsError::new(format!("PV '{}' not found", name))),
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
                        ..
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
                        ..
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
                        ..
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
                        ..
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
                        ..
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

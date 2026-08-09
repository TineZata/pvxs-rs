// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use crate::alarms::{AlarmSeverity, AlarmStatus};
use crate::metadata::{AlarmMetadata, ControlMetadata, DisplayMetadata};
use std::time::{SystemTime, UNIX_EPOCH};

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
    /// Create a new metadata builder with default values.
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

    /// Set the alarm metadata for the scalar.
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

    /// Set the timestamp metadata for the scalar.
    pub fn timestamp(mut self, seconds: i64, nanos: i32, user_tag: i32) -> Self {
        self.timestamp_seconds = seconds;
        self.timestamp_nanos = nanos;
        self.timestamp_user_tag = user_tag;
        self
    }

    /// Set the display metadata for the scalar.
    pub fn display(mut self, meta: DisplayMetadata) -> Self {
        self.display = Some(meta);
        self
    }

    /// Set the control metadata for the scalar.
    pub fn control(mut self, meta: ControlMetadata) -> Self {
        self.control = Some(meta);
        self
    }

    /// Set the alarm metadata object for the scalar.
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

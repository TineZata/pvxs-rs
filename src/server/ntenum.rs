// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use std::time::{SystemTime, UNIX_EPOCH};
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
    /// Create a new enum metadata builder with default values.
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

    /// Set the alarm metadata for the enum.
    pub fn alarm(mut self, severity: i32, status: i32, message: impl Into<String>) -> Self {
        self.alarm_severity = severity;
        self.alarm_status = status;
        self.alarm_message = message.into();
        self
    }

    /// Set the timestamp metadata for the enum.
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

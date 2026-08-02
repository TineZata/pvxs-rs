// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use crate::AlarmSeverity;

/// Alarm metadata for NTScalar
#[derive(Clone, Debug, Default)]
pub struct AlarmMetadata {
    pub active: bool,
    pub low_alarm_limit: f64,
    pub low_warning_limit: f64,
    pub high_warning_limit: f64,
    pub high_alarm_limit: f64,
    pub low_alarm_severity: AlarmSeverity,
    pub low_warning_severity: AlarmSeverity,
    pub high_warning_severity: AlarmSeverity,
    pub high_alarm_severity: AlarmSeverity,
    pub hysteresis: u8,
}

impl AlarmMetadata {
    /// Create a new empty alarm metadata value.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn low_alarm_limit(mut self, limit: f64) -> Self {
        self.low_alarm_limit = limit;
        self
    }

    pub fn low_warning_limit(mut self, limit: f64) -> Self {
        self.low_warning_limit = limit;
        self
    }

    pub fn high_warning_limit(mut self, limit: f64) -> Self {
        self.high_warning_limit = limit;
        self
    }

    pub fn high_alarm_limit(mut self, limit: f64) -> Self {
        self.high_alarm_limit = limit;
        self
    }

    pub fn low_alarm_severity(mut self, severity: AlarmSeverity) -> Self {
        self.low_alarm_severity = severity;
        self
    }

    pub fn low_warning_severity(mut self, severity: AlarmSeverity) -> Self {
        self.low_warning_severity = severity;
        self
    }

    pub fn high_warning_severity(mut self, severity: AlarmSeverity) -> Self {
        self.high_warning_severity = severity;
        self
    }

    pub fn high_alarm_severity(mut self, severity: AlarmSeverity) -> Self {
        self.high_alarm_severity = severity;
        self
    }

    pub fn hysteresis(mut self, hysteresis: u8) -> Self {
        self.hysteresis = hysteresis;
        self
    }
}

/// Display metadata for NTScalar
#[derive(Clone, Debug, Default)]
pub struct DisplayMetadata {
    pub limit_low: i64,
    pub limit_high: i64,
    pub description: String,
    pub units: String,
    pub precision: i32,
}

/// Control metadata for NTScalar
#[derive(Clone, Debug, Default)]
pub struct ControlMetadata {
    pub limit_low: f64,
    pub limit_high: f64,
    pub min_step: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alarm_metadata_builder_sets_all_fields() {
        let m = AlarmMetadata::new()
            .active(true)
            .low_alarm_limit(1.0)
            .low_warning_limit(2.0)
            .high_warning_limit(8.0)
            .high_alarm_limit(9.0)
            .low_alarm_severity(AlarmSeverity::Major)
            .low_warning_severity(AlarmSeverity::Minor)
            .high_warning_severity(AlarmSeverity::Minor)
            .high_alarm_severity(AlarmSeverity::Invalid)
            .hysteresis(3);

        assert!(m.active);
        assert_eq!(m.low_alarm_limit, 1.0);
        assert_eq!(m.low_warning_limit, 2.0);
        assert_eq!(m.high_warning_limit, 8.0);
        assert_eq!(m.high_alarm_limit, 9.0);
        assert_eq!(m.low_alarm_severity, AlarmSeverity::Major);
        assert_eq!(m.low_warning_severity, AlarmSeverity::Minor);
        assert_eq!(m.high_warning_severity, AlarmSeverity::Minor);
        assert_eq!(m.high_alarm_severity, AlarmSeverity::Invalid);
        assert_eq!(m.hysteresis, 3);
    }

    #[test]
    fn alarm_metadata_builder_last_write_wins() {
        let m = AlarmMetadata::new()
            .active(false)
            .active(true)
            .hysteresis(1)
            .hysteresis(7)
            .low_alarm_severity(AlarmSeverity::Minor)
            .low_alarm_severity(AlarmSeverity::Major);

        assert!(m.active);
        assert_eq!(m.hysteresis, 7);
        assert_eq!(m.low_alarm_severity, AlarmSeverity::Major);
    }

    #[test]
    fn alarm_metadata_new_matches_default() {
        let a = AlarmMetadata::new();
        let b = AlarmMetadata::default();

        assert_eq!(a.active, b.active);
        assert_eq!(a.low_alarm_limit, b.low_alarm_limit);
        assert_eq!(a.low_warning_limit, b.low_warning_limit);
        assert_eq!(a.high_warning_limit, b.high_warning_limit);
        assert_eq!(a.high_alarm_limit, b.high_alarm_limit);
        assert_eq!(a.low_alarm_severity, b.low_alarm_severity);
        assert_eq!(a.low_warning_severity, b.low_warning_severity);
        assert_eq!(a.high_warning_severity, b.high_warning_severity);
        assert_eq!(a.high_alarm_severity, b.high_alarm_severity);
        assert_eq!(a.hysteresis, b.hysteresis);
    }
}

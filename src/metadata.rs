// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use crate::AlarmSeverity;

/// Alarm metadata for NTScalar.
#[derive(Clone, Debug, Default)]
pub struct AlarmMetadata {
    /// Whether the alarm calculation is active.
    pub active: bool,
    /// Low alarm threshold.
    pub low_alarm_limit: f64,
    /// Low warning threshold.
    pub low_warning_limit: f64,
    /// High warning threshold.
    pub high_warning_limit: f64,
    /// High alarm threshold.
    pub high_alarm_limit: f64,
    /// Severity used when crossing the low alarm threshold.
    pub low_alarm_severity: AlarmSeverity,
    /// Severity used when crossing the low warning threshold.
    pub low_warning_severity: AlarmSeverity,
    /// Severity used when crossing the high warning threshold.
    pub high_warning_severity: AlarmSeverity,
    /// Severity used when crossing the high alarm threshold.
    pub high_alarm_severity: AlarmSeverity,
    /// Hysteresis applied to alarm comparisons.
    pub hysteresis: u8,
}

impl AlarmMetadata {
    /// Create a new empty alarm metadata value.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable the alarm calculation.
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Set the low alarm threshold.
    pub fn low_alarm_limit(mut self, limit: f64) -> Self {
        self.low_alarm_limit = limit;
        self
    }

    /// Set the low warning threshold.
    pub fn low_warning_limit(mut self, limit: f64) -> Self {
        self.low_warning_limit = limit;
        self
    }

    /// Set the high warning threshold.
    pub fn high_warning_limit(mut self, limit: f64) -> Self {
        self.high_warning_limit = limit;
        self
    }

    /// Set the high alarm threshold.
    pub fn high_alarm_limit(mut self, limit: f64) -> Self {
        self.high_alarm_limit = limit;
        self
    }

    /// Set the severity for low alarm transitions.
    pub fn low_alarm_severity(mut self, severity: AlarmSeverity) -> Self {
        self.low_alarm_severity = severity;
        self
    }

    /// Set the severity for low warning transitions.
    pub fn low_warning_severity(mut self, severity: AlarmSeverity) -> Self {
        self.low_warning_severity = severity;
        self
    }

    /// Set the severity for high warning transitions.
    pub fn high_warning_severity(mut self, severity: AlarmSeverity) -> Self {
        self.high_warning_severity = severity;
        self
    }

    /// Set the severity for high alarm transitions.
    pub fn high_alarm_severity(mut self, severity: AlarmSeverity) -> Self {
        self.high_alarm_severity = severity;
        self
    }

    /// Set the hysteresis value.
    pub fn hysteresis(mut self, hysteresis: u8) -> Self {
        self.hysteresis = hysteresis;
        self
    }
}

/// Display metadata for NTScalar.
#[derive(Clone, Debug, Default)]
pub struct DisplayMetadata {
    /// Lower display limit.
    pub limit_low: i64,
    /// Upper display limit.
    pub limit_high: i64,
    /// Human-readable description.
    pub description: String,
    /// Engineering units.
    pub units: String,
    /// Display precision.
    pub precision: i32,
}

/// Control metadata for NTScalar.
#[derive(Clone, Debug, Default)]
pub struct ControlMetadata {
    /// Lower control limit.
    pub limit_low: f64,
    /// Upper control limit.
    pub limit_high: f64,
    /// Minimum step size.
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

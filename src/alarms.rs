// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use crate::{AlarmMetadata, ControlMetadata};

/// Severity reported for alarm conditions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlarmSeverity {
    /// No active alarm.
    #[default]
    NoAlarm = 0,
    /// Minor alarm severity.
    Minor = 1,
    /// Major alarm severity.
    Major = 2,
    /// Invalid or out-of-range condition.
    Invalid = 3,
    /// Undefined alarm severity.
    UndefinedAlarm = 4,
}

impl From<i32> for AlarmSeverity {
    fn from(value: i32) -> Self {
        match value {
            0 => AlarmSeverity::NoAlarm,
            1 => AlarmSeverity::Minor,
            2 => AlarmSeverity::Major,
            3 => AlarmSeverity::Invalid,
            4 => AlarmSeverity::UndefinedAlarm,
            _ => AlarmSeverity::UndefinedAlarm,
        }
    }
}

/// Status reported for alarm conditions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlarmStatus {
    /// No active alarm status.
    #[default]
    NoAlarm = 0,
    /// Device-related status.
    DeviceStatus = 1,
    /// Driver-related status.
    DriverStatus = 2,
    /// Record-related status.
    RecordStatus = 3,
    /// Database-related status.
    DbStatus = 4,
    /// Configuration-related status.
    ConfigStatus = 5,
    /// Undefined status.
    UndefinedStatus = 6,
    /// Client-related status.
    ClientStatus = 7,
}

impl From<i32> for AlarmStatus {
    fn from(value: i32) -> Self {
        match value {
            0 => AlarmStatus::NoAlarm,
            1 => AlarmStatus::DeviceStatus,
            2 => AlarmStatus::DriverStatus,
            3 => AlarmStatus::RecordStatus,
            4 => AlarmStatus::DbStatus,
            5 => AlarmStatus::ConfigStatus,
            6 => AlarmStatus::UndefinedStatus,
            7 => AlarmStatus::ClientStatus,
            _ => AlarmStatus::UndefinedStatus,
        }
    }
}

/// Configuration used to evaluate scalar alarms.
#[derive(Clone, Debug, Default)]
pub struct AlarmConfig {
    pub(crate) control: Option<ControlMetadata>,
    pub(crate) alarm_metadata: Option<AlarmMetadata>,
}

/// Result of evaluating a scalar alarm against a configuration.
#[derive(Clone, Debug)]
pub struct AlarmResult {
    pub(crate) allow: bool,
    pub(crate) severity: AlarmSeverity,
    pub(crate) status: AlarmStatus,
    pub(crate) message: String,
}

/// Compute the alarm state for a scalar value using the provided configuration.
pub fn compute_alarm_for_scalar(value: f64, config: &AlarmConfig) -> AlarmResult {
    // Control limits: if present, reject updates outside limits
    if let Some(control) = &config.control {
        let hysteresis = config
            .alarm_metadata
            .as_ref()
            .map_or(0.0, |m| m.hysteresis as f64);
        if value < control.limit_low + hysteresis || value > control.limit_high - hysteresis {
            return AlarmResult {
                allow: false,
                severity: AlarmSeverity::Invalid,
                status: AlarmStatus::RecordStatus,
                message: "OUT_OF_CONTROL_LIMITS".to_string(),
            };
        }
    }

    if let Some(value_alarm) = &config.alarm_metadata {
        if value_alarm.active {
            if value <= value_alarm.low_alarm_limit + value_alarm.hysteresis as f64 {
                return AlarmResult {
                    allow: true,
                    severity: value_alarm.low_alarm_severity,
                    status: AlarmStatus::DeviceStatus,
                    message: "LOW_ALARM".to_string(),
                };
            }
            if value <= value_alarm.low_warning_limit + value_alarm.hysteresis as f64 {
                return AlarmResult {
                    allow: true,
                    severity: value_alarm.low_warning_severity,
                    status: AlarmStatus::DeviceStatus,
                    message: "LOW_WARNING".to_string(),
                };
            }
            if value >= value_alarm.high_alarm_limit - value_alarm.hysteresis as f64 {
                return AlarmResult {
                    allow: true,
                    severity: value_alarm.high_alarm_severity,
                    status: AlarmStatus::DeviceStatus,
                    message: "HIGH_ALARM".to_string(),
                };
            }
            if value >= value_alarm.high_warning_limit - value_alarm.hysteresis as f64 {
                return AlarmResult {
                    allow: true,
                    severity: value_alarm.high_warning_severity,
                    status: AlarmStatus::DeviceStatus,
                    message: "HIGH_WARNING".to_string(),
                };
            }
        }
    }

    AlarmResult {
        allow: true,
        severity: AlarmSeverity::NoAlarm,
        status: AlarmStatus::NoAlarm,
        message: "OK".to_string(),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── AlarmSeverity From<i32> ─────────────────────────────────────────────

    #[test]
    fn severity_from_i32_known() {
        assert_eq!(AlarmSeverity::from(0), AlarmSeverity::NoAlarm);
        assert_eq!(AlarmSeverity::from(1), AlarmSeverity::Minor);
        assert_eq!(AlarmSeverity::from(2), AlarmSeverity::Major);
        assert_eq!(AlarmSeverity::from(3), AlarmSeverity::Invalid);
        assert_eq!(AlarmSeverity::from(4), AlarmSeverity::UndefinedAlarm);
    }

    #[test]
    fn severity_from_i32_unknown_maps_to_undefined() {
        assert_eq!(AlarmSeverity::from(99), AlarmSeverity::UndefinedAlarm);
        assert_eq!(AlarmSeverity::from(-1), AlarmSeverity::UndefinedAlarm);
    }

    // ── AlarmStatus From<i32> ───────────────────────────────────────────────

    #[test]
    fn status_from_i32_known() {
        assert_eq!(AlarmStatus::from(0), AlarmStatus::NoAlarm);
        assert_eq!(AlarmStatus::from(7), AlarmStatus::ClientStatus);
    }

    #[test]
    fn status_from_i32_unknown_maps_to_undefined() {
        assert_eq!(AlarmStatus::from(99), AlarmStatus::UndefinedStatus);
    }

    // ── compute_alarm_for_scalar ────────────────────────────────────────────

    fn alarm_meta(
        low_alarm: f64,
        low_warn: f64,
        high_warn: f64,
        high_alarm: f64,
    ) -> AlarmMetadata {
        AlarmMetadata {
            active: true,
            low_alarm_limit: low_alarm,
            low_warning_limit: low_warn,
            high_warning_limit: high_warn,
            high_alarm_limit: high_alarm,
            low_alarm_severity: AlarmSeverity::Major,
            low_warning_severity: AlarmSeverity::Minor,
            high_warning_severity: AlarmSeverity::Minor,
            high_alarm_severity: AlarmSeverity::Major,
            hysteresis: 0,
        }
    }

    #[test]
    fn no_alarm_in_range() {
        let config = AlarmConfig {
            control: None,
            alarm_metadata: Some(alarm_meta(10.0, 20.0, 80.0, 90.0)),
        };
        let r = compute_alarm_for_scalar(50.0, &config);
        assert!(r.allow);
        assert_eq!(r.severity, AlarmSeverity::NoAlarm);
    }

    #[test]
    fn high_warning_boundary() {
        let config = AlarmConfig {
            control: None,
            alarm_metadata: Some(alarm_meta(10.0, 20.0, 80.0, 90.0)),
        };
        let r = compute_alarm_for_scalar(80.0, &config);
        assert!(r.allow);
        assert_eq!(r.severity, AlarmSeverity::Minor);
    }

    #[test]
    fn high_alarm_boundary() {
        let config = AlarmConfig {
            control: None,
            alarm_metadata: Some(alarm_meta(10.0, 20.0, 80.0, 90.0)),
        };
        let r = compute_alarm_for_scalar(90.0, &config);
        assert!(r.allow);
        assert_eq!(r.severity, AlarmSeverity::Major);
    }

    #[test]
    fn low_alarm_boundary() {
        let config = AlarmConfig {
            control: None,
            alarm_metadata: Some(alarm_meta(10.0, 20.0, 80.0, 90.0)),
        };
        let r = compute_alarm_for_scalar(10.0, &config);
        assert!(r.allow);
        assert_eq!(r.severity, AlarmSeverity::Major);
    }

    #[test]
    fn control_limit_rejection() {
        let config = AlarmConfig {
            control: Some(ControlMetadata {
                limit_low: 0.0,
                limit_high: 100.0,
                min_step: 0.0,
            }),
            alarm_metadata: None,
        };
        let r = compute_alarm_for_scalar(150.0, &config);
        assert!(!r.allow);
        assert_eq!(r.severity, AlarmSeverity::Invalid);
    }

    #[test]
    fn inactive_alarm_metadata_no_alarm() {
        let mut meta = alarm_meta(10.0, 20.0, 80.0, 90.0);
        meta.active = false;
        let config = AlarmConfig {
            control: None,
            alarm_metadata: Some(meta),
        };
        // Even though value is outside limits, inactive means no alarm
        let r = compute_alarm_for_scalar(5.0, &config);
        assert!(r.allow);
        assert_eq!(r.severity, AlarmSeverity::NoAlarm);
    }
}

// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use std::collections::HashMap;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// A PVAccess value container — pure-Rust replacement for the cxx-backed ValueWrapper.
///
/// Represents a structured data value returned from pvAccess operations.
/// Values have a hierarchical structure with named fields, accessed by
/// dot-separated paths (e.g. `"alarm.severity"`).
///
/// # Example
///
/// ```
/// use pvxs::Value;
///
/// let mut v = Value::new();
/// v.set_display_units("A");
/// v.set_timestamp_seconds(1_724_000_000);
/// assert_eq!(v.get_display_units().unwrap(), "A");
/// assert_eq!(v.get_timestamp_seconds().unwrap(), 1_724_000_000);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Value {
    fields: HashMap<String, FieldValue>,
}

/// The type of a field stored in a [`Value`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Double,
    Int32,
    Int64,
    Bool,
    String,
    Enum,
    DoubleArray,
    Int32Array,
    StringArray,
}

#[derive(Debug, Clone)]
pub(crate) enum FieldValue {
    Double(f64),
    Int32(i32),
    Int64(i64),
    Bool(bool),
    String(String),
    Enum(i16),
    DoubleArray(Vec<f64>),
    Int32Array(Vec<i32>),
    StringArray(Vec<String>),
}

impl Value {
    /// Create an empty Value.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if this value is valid (non-empty).
    pub fn is_valid(&self) -> bool {
        !self.fields.is_empty()
    }

    /// Build an NTScalar value with standard alarm/timestamp fields populated.
    pub fn nt_scalar_double(value: f64) -> Self {
        let mut out = Self::new();
        out.set_field_double("value", value);
        out.populate_nt_common();
        out
    }

    /// Build an NTScalar value with standard alarm/timestamp fields populated.
    pub fn nt_scalar_int32(value: i32) -> Self {
        let mut out = Self::new();
        out.set_field_int32("value", value);
        out.populate_nt_common();
        out
    }

    /// Build an NTScalar value with standard alarm/timestamp fields populated.
    pub fn nt_scalar_string(value: impl Into<String>) -> Self {
        let mut out = Self::new();
        out.set_field_string("value", value.into());
        out.populate_nt_common();
        out
    }

    /// Build an NTScalarArray value with standard alarm/timestamp fields populated.
    pub fn nt_scalar_array_double(value: Vec<f64>) -> Self {
        let mut out = Self::new();
        out.set_field_double_array("value", value);
        out.populate_nt_common();
        out
    }

    /// Build an NTScalarArray value with standard alarm/timestamp fields populated.
    pub fn nt_scalar_array_int32(value: Vec<i32>) -> Self {
        let mut out = Self::new();
        out.set_field_int32_array("value", value);
        out.populate_nt_common();
        out
    }

    /// Build an NTScalarArray value with standard alarm/timestamp fields populated.
    pub fn nt_scalar_array_string(value: Vec<String>) -> Self {
        let mut out = Self::new();
        out.set_field_string_array("value", value);
        out.populate_nt_common();
        out
    }

    /// Build an NTEnum value with standard alarm/timestamp fields populated.
    ///
    /// The selected enum index is stored in `value`, and the list of choice
    /// labels is stored in `value.choices`.
    pub fn nt_enum(value: i16, choices: Vec<String>) -> Self {
        let mut out = Self::new();
        out.set_field_enum("value", value);
        out.set_field_string_array("value.choices", choices);
        out.populate_nt_common();
        out
    }

    fn populate_nt_common(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        self.set_field_int32("alarm.severity", 0);
        self.set_field_int32("alarm.status", 0);
        self.set_field_string("alarm.message", "OK".to_string());
        self.set_field_int64("timeStamp.secondsPastEpoch", now.as_secs() as i64);
        self.set_field_int32("timeStamp.nanoseconds", now.subsec_nanos() as i32);
        self.set_field_int32("timeStamp.userTag", 0);
    }

    // ── setters (pub(crate) — used by the client/server impls) ──────────────

    pub(crate) fn set_field_double(&mut self, field: &str, v: f64) {
        self.fields.insert(field.to_string(), FieldValue::Double(v));
    }

    pub(crate) fn set_field_int32(&mut self, field: &str, v: i32) {
        self.fields.insert(field.to_string(), FieldValue::Int32(v));
    }

    pub(crate) fn set_field_string(&mut self, field: &str, v: String) {
        self.fields.insert(field.to_string(), FieldValue::String(v));
    }

    pub(crate) fn set_field_enum(&mut self, field: &str, v: i16) {
        self.fields.insert(field.to_string(), FieldValue::Enum(v));
    }

    pub(crate) fn set_field_double_array(&mut self, field: &str, v: Vec<f64>) {
        self.fields
            .insert(field.to_string(), FieldValue::DoubleArray(v));
    }

    pub(crate) fn set_field_int32_array(&mut self, field: &str, v: Vec<i32>) {
        self.fields
            .insert(field.to_string(), FieldValue::Int32Array(v));
    }

    pub(crate) fn set_field_string_array(&mut self, field: &str, v: Vec<String>) {
        self.fields
            .insert(field.to_string(), FieldValue::StringArray(v));
    }

    // ── getters ──────────────────────────────────────────────────────────────

    /// Get a field value as a double.
    pub fn get_field_double(&self, field_name: &str) -> crate::Result<f64> {
        match self.fields.get(field_name) {
            Some(FieldValue::Double(v)) => Ok(*v),
            Some(FieldValue::Int32(v)) => Ok(*v as f64),
            Some(FieldValue::Enum(v)) => Ok(*v as f64),
            Some(_) => Err(crate::PvxsError::new(format!(
                "field '{}' is not a double",
                field_name
            ))),
            None => Err(crate::PvxsError::new(format!(
                "field '{}' not found",
                field_name
            ))),
        }
    }

    /// Get a field value as an i32.
    pub fn get_field_int32(&self, field_name: &str) -> crate::Result<i32> {
        match self.fields.get(field_name) {
            Some(FieldValue::Int32(v)) => Ok(*v),
            Some(FieldValue::Double(v)) => Ok(*v as i32),
            Some(FieldValue::Enum(v)) => Ok(*v as i32),
            Some(_) => Err(crate::PvxsError::new(format!(
                "field '{}' is not an int32",
                field_name
            ))),
            None => Err(crate::PvxsError::new(format!(
                "field '{}' not found",
                field_name
            ))),
        }
    }

    /// Get a field value as a String.
    pub fn get_field_string(&self, field_name: &str) -> crate::Result<String> {
        match self.fields.get(field_name) {
            Some(FieldValue::String(v)) => Ok(v.clone()),
            Some(FieldValue::Double(v)) => Ok(v.to_string()),
            Some(FieldValue::Int32(v)) => Ok(v.to_string()),
            Some(FieldValue::Enum(v)) => Ok(v.to_string()),
            Some(_) => Err(crate::PvxsError::new(format!(
                "field '{}' is not a string",
                field_name
            ))),
            None => Err(crate::PvxsError::new(format!(
                "field '{}' not found",
                field_name
            ))),
        }
    }

    /// Get a field value as an enum index (i16).
    pub fn get_field_enum(&self, field_name: &str) -> crate::Result<i16> {
        match self.fields.get(field_name) {
            Some(FieldValue::Enum(v)) => Ok(*v),
            Some(FieldValue::Int32(v)) => Ok(*v as i16),
            Some(_) => Err(crate::PvxsError::new(format!(
                "field '{}' is not an enum",
                field_name
            ))),
            None => Err(crate::PvxsError::new(format!(
                "field '{}' not found",
                field_name
            ))),
        }
    }

    /// Get a field value as an array of doubles.
    pub fn get_field_double_array(&self, field_name: &str) -> crate::Result<Vec<f64>> {
        match self.fields.get(field_name) {
            Some(FieldValue::DoubleArray(v)) => Ok(v.clone()),
            Some(_) => Err(crate::PvxsError::new(format!(
                "field '{}' is not a double array",
                field_name
            ))),
            None => Err(crate::PvxsError::new(format!(
                "field '{}' not found",
                field_name
            ))),
        }
    }

    /// Get a field value as an array of i32.
    pub fn get_field_int32_array(&self, field_name: &str) -> crate::Result<Vec<i32>> {
        match self.fields.get(field_name) {
            Some(FieldValue::Int32Array(v)) => Ok(v.clone()),
            Some(_) => Err(crate::PvxsError::new(format!(
                "field '{}' is not an int32 array",
                field_name
            ))),
            None => Err(crate::PvxsError::new(format!(
                "field '{}' not found",
                field_name
            ))),
        }
    }

    /// Get a field value as an array of strings.
    pub fn get_field_string_array(&self, field_name: &str) -> crate::Result<Vec<String>> {
        match self.fields.get(field_name) {
            Some(FieldValue::StringArray(v)) => Ok(v.clone()),
            Some(_) => Err(crate::PvxsError::new(format!(
                "field '{}' is not a string array",
                field_name
            ))),
            None => Err(crate::PvxsError::new(format!(
                "field '{}' not found",
                field_name
            ))),
        }
    }

    // ── bool ─────────────────────────────────────────────────────────────

    pub(crate) fn set_field_bool(&mut self, field: &str, v: bool) {
        self.fields.insert(field.to_string(), FieldValue::Bool(v));
    }

    pub fn get_field_bool(&self, field_name: &str) -> crate::Result<bool> {
        match self.fields.get(field_name) {
            Some(FieldValue::Bool(v)) => Ok(*v),
            Some(FieldValue::Int32(v)) => Ok(*v != 0),
            Some(FieldValue::Int64(v)) => Ok(*v != 0),
            Some(_) => Err(crate::PvxsError::new(format!(
                "field '{}' is not a bool",
                field_name
            ))),
            None => Err(crate::PvxsError::new(format!(
                "field '{}' not found",
                field_name
            ))),
        }
    }

    // ── i64 ──────────────────────────────────────────────────────────────

    pub(crate) fn set_field_int64(&mut self, field: &str, v: i64) {
        self.fields.insert(field.to_string(), FieldValue::Int64(v));
    }

    pub fn get_field_int64(&self, field_name: &str) -> crate::Result<i64> {
        match self.fields.get(field_name) {
            Some(FieldValue::Int64(v)) => Ok(*v),
            Some(FieldValue::Int32(v)) => Ok(*v as i64),
            Some(FieldValue::Double(v)) => Ok(*v as i64),
            Some(_) => Err(crate::PvxsError::new(format!(
                "field '{}' is not an int64",
                field_name
            ))),
            None => Err(crate::PvxsError::new(format!(
                "field '{}' not found",
                field_name
            ))),
        }
    }

    // ── introspection ─────────────────────────────────────────────────────

    /// Return the names of all fields currently set in this value.
    pub fn field_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.fields.keys().cloned().collect();
        names.sort();
        names
    }

    /// Return the [`FieldType`] of a field, or `None` if the field is not set.
    pub fn type_of(&self, field_name: &str) -> Option<FieldType> {
        self.fields.get(field_name).map(|v| match v {
            FieldValue::Double(_) => FieldType::Double,
            FieldValue::Int32(_) => FieldType::Int32,
            FieldValue::Int64(_) => FieldType::Int64,
            FieldValue::Bool(_) => FieldType::Bool,
            FieldValue::String(_) => FieldType::String,
            FieldValue::Enum(_) => FieldType::Enum,
            FieldValue::DoubleArray(_) => FieldType::DoubleArray,
            FieldValue::Int32Array(_) => FieldType::Int32Array,
            FieldValue::StringArray(_) => FieldType::StringArray,
        })
    }

    // ── display.* convenience setters/getters ────────────────────────────

    pub fn set_display_limit_low(&mut self, v: i64) {
        self.set_field_int64("display.limitLow", v);
    }
    pub fn get_display_limit_low(&self) -> crate::Result<i64> {
        self.get_field_int64("display.limitLow")
    }

    pub fn set_display_limit_high(&mut self, v: i64) {
        self.set_field_int64("display.limitHigh", v);
    }
    pub fn get_display_limit_high(&self) -> crate::Result<i64> {
        self.get_field_int64("display.limitHigh")
    }

    pub fn set_display_units(&mut self, v: impl Into<String>) {
        self.set_field_string("display.units", v.into());
    }
    pub fn get_display_units(&self) -> crate::Result<String> {
        self.get_field_string("display.units")
    }

    pub fn set_display_precision(&mut self, v: i32) {
        self.set_field_int32("display.precision", v);
    }
    pub fn get_display_precision(&self) -> crate::Result<i32> {
        self.get_field_int32("display.precision")
    }

    pub fn set_display_description(&mut self, v: impl Into<String>) {
        self.set_field_string("display.description", v.into());
    }
    pub fn get_display_description(&self) -> crate::Result<String> {
        self.get_field_string("display.description")
    }

    // ── timeStamp.* convenience setters/getters ──────────────────────────

    pub fn set_timestamp_seconds(&mut self, v: i64) {
        self.set_field_int64("timeStamp.secondsPastEpoch", v);
    }
    pub fn get_timestamp_seconds(&self) -> crate::Result<i64> {
        self.get_field_int64("timeStamp.secondsPastEpoch")
    }

    pub fn set_timestamp_nanos(&mut self, v: i32) {
        self.set_field_int32("timeStamp.nanoseconds", v);
    }
    pub fn get_timestamp_nanos(&self) -> crate::Result<i32> {
        self.get_field_int32("timeStamp.nanoseconds")
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut pairs: Vec<_> = self.fields.iter().collect();
        pairs.sort_by_key(|(k, _)| k.as_str());
        write!(f, "{{")? ;
        for (i, (k, v)) in pairs.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?
            }
            match v {
                FieldValue::Double(x) => write!(f, "{k}: {x}")?,
                FieldValue::Int32(x) => write!(f, "{k}: {x}")?,
                FieldValue::Int64(x) => write!(f, "{k}: {x}i64")?,
                FieldValue::Bool(x) => write!(f, "{k}: {x}")?,
                FieldValue::String(x) => write!(f, "{k}: \"{x}\"")?,
                FieldValue::Enum(x) => write!(f, "{k}: enum({x})")?,
                FieldValue::DoubleArray(x) => write!(f, "{k}: {x:?}")?,
                FieldValue::Int32Array(x) => write!(f, "{k}: {x:?}")?,
                FieldValue::StringArray(x) => write!(f, "{k}: {x:?}")?,
            }
        }
        write!(f, "}}")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_round_trips() {
        let mut v = Value::new();
        v.set_field_double("d", 1.5);
        v.set_field_int32("i", -7);
        v.set_field_string("s", "hi".to_string());
        v.set_field_enum("e", 3);
        v.set_field_bool("b", true);
        v.set_field_int64("l", i64::MAX);

        assert!((v.get_field_double("d").unwrap() - 1.5).abs() < 1e-12);
        assert_eq!(v.get_field_int32("i").unwrap(), -7);
        assert_eq!(v.get_field_string("s").unwrap(), "hi");
        assert_eq!(v.get_field_enum("e").unwrap(), 3);
        assert!(v.get_field_bool("b").unwrap());
        assert_eq!(v.get_field_int64("l").unwrap(), i64::MAX);
    }

    #[test]
    fn array_round_trips() {
        let mut v = Value::new();
        v.set_field_double_array("da", vec![1.0, 2.0]);
        v.set_field_int32_array("ia", vec![10, 20]);
        v.set_field_string_array("sa", vec!["a".to_string(), "b".to_string()]);

        assert_eq!(v.get_field_double_array("da").unwrap(), vec![1.0, 2.0]);
        assert_eq!(v.get_field_int32_array("ia").unwrap(), vec![10, 20]);
        assert_eq!(
            v.get_field_string_array("sa").unwrap(),
            vec!["a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn is_valid() {
        let mut v = Value::new();
        assert!(!v.is_valid());
        v.set_field_double("x", 0.0);
        assert!(v.is_valid());
    }

    #[test]
    fn field_names_sorted() {
        let mut v = Value::new();
        v.set_field_double("z", 1.0);
        v.set_field_double("a", 2.0);
        v.set_field_double("m", 3.0);
        assert_eq!(v.field_names(), vec!["a", "m", "z"]);
    }

    #[test]
    fn type_of_correct() {
        let mut v = Value::new();
        v.set_field_double("d", 0.0);
        v.set_field_int32("i", 0);
        v.set_field_bool("b", false);
        assert_eq!(v.type_of("d"), Some(FieldType::Double));
        assert_eq!(v.type_of("i"), Some(FieldType::Int32));
        assert_eq!(v.type_of("b"), Some(FieldType::Bool));
        assert_eq!(v.type_of("missing"), None);
    }

    #[test]
    fn display_timestamp_helpers() {
        let mut v = Value::new();
        v.set_display_limit_low(-100);
        v.set_display_limit_high(100);
        v.set_display_units("mm");
        v.set_display_precision(3);
        v.set_display_description("position");
        v.set_timestamp_seconds(1_000_000_000);
        v.set_timestamp_nanos(500_000_000);

        assert_eq!(v.get_display_limit_low().unwrap(), -100);
        assert_eq!(v.get_display_limit_high().unwrap(), 100);
        assert_eq!(v.get_display_units().unwrap(), "mm");
        assert_eq!(v.get_display_precision().unwrap(), 3);
        assert_eq!(v.get_display_description().unwrap(), "position");
        assert_eq!(v.get_timestamp_seconds().unwrap(), 1_000_000_000);
        assert_eq!(v.get_timestamp_nanos().unwrap(), 500_000_000);
    }

    #[test]
    fn missing_field_errors() {
        let v = Value::new();
        assert!(v.get_field_double("x").is_err());
        assert!(v.get_field_int32("x").is_err());
        assert!(v.get_field_string("x").is_err());
    }

    #[test]
    fn type_mismatch_errors() {
        let mut v = Value::new();
        v.set_field_double_array("da", vec![1.0, 2.0]);
        // Scalar string accessor on an array field should error
        assert!(v.get_field_string("da").is_err());
        // Bool accessor on a double-array field should error
        assert!(v.get_field_bool("da").is_err());
    }

    #[test]
    fn nt_scalar_builder_populates_common_fields() {
        let v = Value::nt_scalar_double(3.25);

        assert!((v.get_field_double("value").unwrap() - 3.25).abs() < 1e-12);
        assert_eq!(v.get_field_int32("alarm.severity").unwrap(), 0);
        assert_eq!(v.get_field_int32("alarm.status").unwrap(), 0);
        assert_eq!(v.get_field_string("alarm.message").unwrap(), "OK");
        assert!(v.get_timestamp_seconds().unwrap() > 0);
        assert!(v.get_timestamp_nanos().unwrap() >= 0);
        assert_eq!(v.get_field_int32("timeStamp.userTag").unwrap(), 0);
    }

    #[test]
    fn nt_scalar_array_builder_populates_value_and_common_fields() {
        let v = Value::nt_scalar_array_int32(vec![1, 2, 3]);

        assert_eq!(v.get_field_int32_array("value").unwrap(), vec![1, 2, 3]);
        assert_eq!(v.get_field_int32("alarm.severity").unwrap(), 0);
        assert_eq!(v.get_field_int32("alarm.status").unwrap(), 0);
        assert_eq!(v.get_field_string("alarm.message").unwrap(), "OK");
        assert!(v.get_timestamp_seconds().unwrap() > 0);
    }

    #[test]
    fn nt_enum_builder_populates_choices_and_common_fields() {
        let v = Value::nt_enum(1, vec!["OFF".to_string(), "ON".to_string()]);

        assert_eq!(v.get_field_enum("value").unwrap(), 1);
        assert_eq!(
            v.get_field_string_array("value.choices").unwrap(),
            vec!["OFF".to_string(), "ON".to_string()]
        );
        assert_eq!(v.get_field_int32("alarm.severity").unwrap(), 0);
        assert_eq!(v.get_field_int32("alarm.status").unwrap(), 0);
        assert_eq!(v.get_field_string("alarm.message").unwrap(), "OK");
    }
}

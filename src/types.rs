//! Common types and value representations for PVXS

use crate::error::{Error, Result};
use std::fmt;

/// Re-export the underlying Value type from epics-pvxs-sys
pub use epics_pvxs_sys::Value as PvxsValue;

/// High-level wrapper around PVXS Value with convenient methods
#[derive(Debug)]
pub struct Value {
    inner: PvxsValue,
}

impl Value {
    /// Create a new Value from the underlying PVXS value
    pub fn from_pvxs(inner: PvxsValue) -> Self {
        Self { inner }
    }

    /// Get the underlying PVXS value
    pub fn into_pvxs(self) -> PvxsValue {
        self.inner
    }

    /// Get a reference to the underlying PVXS value
    pub fn as_pvxs(&self) -> &PvxsValue {
        &self.inner
    }

    /// Get a string field by name
    pub fn get_string(&self, field: &str) -> Result<String> {
        self.inner
            .get_field_string(field)
            .map_err(|_| Error::field_access(field))
    }

    /// Get a double field by name
    pub fn get_double(&self, field: &str) -> Result<f64> {
        self.inner
            .get_field_double(field)
            .map_err(|_| Error::field_access(field))
    }

    /// Get an integer field by name
    pub fn get_int(&self, field: &str) -> Result<i32> {
        self.inner
            .get_field_int32(field)
            .map_err(|_| Error::field_access(field))
    }

    /// Get a long field by name (using int32 for now)
    pub fn get_long(&self, field: &str) -> Result<i64> {
        self.inner
            .get_field_int32(field)
            .map(|v| v as i64)
            .map_err(|_| Error::field_access(field))
    }

    /// Get an enum field by name (enum values are i16 indices)
    pub fn get_enum(&self, field: &str) -> Result<i16> {
        self.inner
            .get_field_enum(field)
            .map_err(|_| Error::field_access(field))
    }

    /// Get a double array field by name
    pub fn get_double_array(&self, field: &str) -> Result<Vec<f64>> {
        self.inner
            .get_field_double_array(field)
            .map_err(|_| Error::field_access(field))
    }

    /// Get an integer array field by name
    pub fn get_int_array(&self, field: &str) -> Result<Vec<i32>> {
        self.inner
            .get_field_int32_array(field)
            .map_err(|_| Error::field_access(field))
    }

    /// Get a string array field by name
    pub fn get_string_array(&self, field: &str) -> Result<Vec<String>> {
        self.inner
            .get_field_string_array(field)
            .map_err(|_| Error::field_access(field))
    }

    /// Get enum choices for a field
    ///
    /// For NTEnum types, this retrieves the string choices array.
    /// Typically used with paths like "value.choices" for the main enum field.
    ///
    /// # Arguments
    ///
    /// * `field` - The field path to the choices array (e.g., "value.choices")
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Client;
    /// # let mut client = Client::new().unwrap();
    /// let value = client.get("MY:ENUM:PV", 5.0)?;
    /// let choices = value.get_enum_choices("value.choices")?;
    /// let index = value.get_enum("value.index")?;
    /// println!("Current choice: '{}'", choices[index as usize]);
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn get_enum_choices(&self, field: &str) -> Result<Vec<String>> {
        self.get_string_array(field)
    }

    /// Attempt to get the main "value" field as a double
    /// This is a convenience method for scalar PVs
    pub fn as_double(&self) -> Result<f64> {
        self.get_double("value")
    }

    /// Attempt to get the main "value" field as a string
    /// This is a convenience method for string PVs
    pub fn as_string(&self) -> Result<String> {
        self.get_string("value")
    }

    /// Attempt to get the main "value" field as an integer
    /// This is a convenience method for integer PVs
    pub fn as_int(&self) -> Result<i32> {
        self.get_int("value")
    }

    /// Attempt to get the main "value" field as a long
    /// This is a convenience method for long integer PVs
    /// Currently uses int32 for long values
    pub fn as_long(&self) -> Result<i64> {
        self.get_long("value")
    }

    /// Attempt to get the main "value.index" field as an enum index
    /// This is a convenience method for enum PVs (returns i16 index)
    pub fn as_enum_index(&self) -> Result<i16> {
        self.get_enum("value.index")
    }

    /// Attempt to get the main "value.choices" field as a string array
    /// This is a convenience method for enum PVs
    pub fn as_enum_choices(&self) -> Result<Vec<String>> {
        self.get_string_array("value.choices")
    }

    /// Attempt to get the main "value" field as a double array
    /// This is a convenience method for array PVs
    pub fn as_double_array(&self) -> Result<Vec<f64>> {
        self.get_double_array("value")
    }

    /// Attempt to get the main "value" field as an integer array
    /// This is a convenience method for array PVs
    pub fn as_int_array(&self) -> Result<Vec<i32>> {
        self.get_int_array("value")
    }

    /// Attempt to get the main "value" field as a string array
    /// This is a convenience method for array PVs
    pub fn as_string_array(&self) -> Result<Vec<String>> {
        self.get_string_array("value")
    }

    /// Get alarm information if available
    pub fn alarm_info(&self) -> AlarmInfo {
        let severity = self.get_int("alarm.severity").unwrap_or(0);
        let status = self.get_int("alarm.status").unwrap_or(0);
        let message = self.get_string("alarm.message").unwrap_or_default();
        
        AlarmInfo {
            severity: AlarmSeverity::from_int(severity),
            status,
            message,
        }
    }

    /// Get timestamp information if available
    pub fn timestamp(&self) -> Option<Timestamp> {
        // Try to get timestamp fields (using int32 fields for now)
        if let (Ok(seconds), Ok(nanoseconds)) = (
            self.get_int("timeStamp.secondsPastEpoch"),
            self.get_int("timeStamp.nanoseconds")
        ) {
            Some(Timestamp {
                seconds_past_epoch: seconds as i64,
                nanoseconds: nanoseconds as u32,
            })
        } else {
            None
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Try to display the most common value types
        if let Ok(val) = self.as_double() {
            write!(f, "{}", val)
        } else if let Ok(val) = self.as_string() {
            write!(f, "{}", val)
        } else if let Ok(val) = self.as_int() {
            write!(f, "{}", val)
        } else {
            // Fallback to the underlying display
            write!(f, "{}", self.inner)
        }
    }
}

/// Timestamp information for PV values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timestamp {
    /// Seconds since EPICS epoch (1990-01-01 00:00:00 UTC)
    pub seconds_past_epoch: i64,
    /// Nanoseconds within the second
    pub nanoseconds: u32,
}

impl Timestamp {
    /// Create a new timestamp
    pub fn new(seconds_past_epoch: i64, nanoseconds: u32) -> Self {
        Self {
            seconds_past_epoch,
            nanoseconds,
        }
    }

    /// Get the current time as a timestamp
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // EPICS epoch is 1990-01-01, UNIX epoch is 1970-01-01
        // Difference is 20 years = 631152000 seconds
        const EPICS_EPOCH_OFFSET: i64 = 631152000;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        
        let epics_seconds = now.as_secs() as i64 - EPICS_EPOCH_OFFSET;
        let nanoseconds = now.subsec_nanos();
        
        Self::new(epics_seconds, nanoseconds)
    }

    /// Convert to seconds as a floating-point number
    pub fn as_f64(&self) -> f64 {
        self.seconds_past_epoch as f64 + (self.nanoseconds as f64 / 1_000_000_000.0)
    }

    /// Convert to human-readable string
    pub fn to_string(&self) -> String {
        // Convert EPICS epoch to UNIX epoch for display
        const EPICS_EPOCH_OFFSET: i64 = 631152000;
        let unix_seconds = self.seconds_past_epoch + EPICS_EPOCH_OFFSET;
        let datatime = chrono::DateTime::<chrono::Utc>::from_timestamp(unix_seconds, self.nanoseconds).unwrap_or_default();
        let time_stamp_str = datatime.format("%Y-%m-%d %H:%M:%S").to_string();
        format!("{}.{}", time_stamp_str, self.nanoseconds)
    }
}

/// Alarm severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmSeverity {
    /// No alarm
    None = 0,
    /// Minor alarm
    Minor = 1,
    /// Major alarm
    Major = 2,
    /// Invalid alarm
    Invalid = 3,
}

impl AlarmSeverity {
    /// Convert from integer value
    pub fn from_int(value: i32) -> Self {
        match value {
            0 => AlarmSeverity::None,
            1 => AlarmSeverity::Minor,
            2 => AlarmSeverity::Major,
            3 => AlarmSeverity::Invalid,
            _ => AlarmSeverity::Invalid,
        }
    }

    /// Convert to integer value
    pub fn as_int(self) -> i32 {
        self as i32
    }

    /// Check if this represents an alarm condition
    pub fn is_alarm(self) -> bool {
        !matches!(self, AlarmSeverity::None)
    }
}

impl fmt::Display for AlarmSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlarmSeverity::None => write!(f, "NONE"),
            AlarmSeverity::Minor => write!(f, "MINOR"),
            AlarmSeverity::Major => write!(f, "MAJOR"),
            AlarmSeverity::Invalid => write!(f, "INVALID"),
        }
    }
}

/// Alarm information for a PV value
#[derive(Debug, Clone)]
pub struct AlarmInfo {
    /// Alarm severity
    pub severity: AlarmSeverity,
    /// Alarm status code
    pub status: i32,
    /// Alarm message
    pub message: String,
}

impl AlarmInfo {
    /// Check if there is an active alarm
    pub fn has_alarm(&self) -> bool {
        self.severity.is_alarm()
    }
}

impl fmt::Display for AlarmInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.has_alarm() {
            if self.message.is_empty() {
                write!(f, "{} ({})", self.severity, self.status)
            } else {
                write!(f, "{} ({}): {}", self.severity, self.status, self.message)
            }
        } else {
            write!(f, "OK")
        }
    }
}

/// PV connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// PV is disconnected
    Disconnected,
    /// PV is connected
    Connected,
    /// PV connection state is unknown
    Unknown,
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionState::Disconnected => write!(f, "DISCONNECTED"),
            ConnectionState::Connected => write!(f, "CONNECTED"),
            ConnectionState::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// PV access rights
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessRights {
    /// Can read from this PV
    pub read: bool,
    /// Can write to this PV
    pub write: bool,
}

impl AccessRights {
    /// Create new access rights
    pub fn new(read: bool, write: bool) -> Self {
        Self { read, write }
    }

    /// Read-only access
    pub fn read_only() -> Self {
        Self::new(true, false)
    }

    /// Read-write access
    pub fn read_write() -> Self {
        Self::new(true, true)
    }

    /// No access
    pub fn none() -> Self {
        Self::new(false, false)
    }
}

impl fmt::Display for AccessRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.read, self.write) {
            (true, true) => write!(f, "READ/WRITE"),
            (true, false) => write!(f, "READ ONLY"),
            (false, true) => write!(f, "WRITE ONLY"),
            (false, false) => write!(f, "NO ACCESS"),
        }
    }
}
use std::collections::HashMap;

/// Flexible recursive data structure to represent various types of PV data
#[derive(Debug, Clone)]
pub enum PVData {
    Double(f64),
    Int(i64),
    String(String),
    Structure(HashMap<String, PVData>),
    Invalid, // To represent missing or invalid fields
}

impl PVData {
    /// Helper to extract a value as a string for display
    pub fn to_string(&self) -> String {
        match self {
            PVData::Double(v) => format!("Double({})", v),
            PVData::Int(v) => format!("Int({})", v),
            PVData::String(v) => format!("String({})", v),
            PVData::Structure(v) => format!("Structure({:?})", v),
            PVData::Invalid => "Invalid".to_string(),
        }
    }
}


use crate::{PvxsError, Result, Value};


/// RPC (Remote Procedure Call) builder.
///
/// Mirrors `pvxs-sys::Rpc` exactly.
///
/// TODO(network): pvAccess TCP transport not yet implemented.
pub struct Rpc {
    _name: String,
    _args: Value,
}

impl Rpc {
    pub(crate) fn new(name: String) -> Self {
        Self {
            _name: name,
            _args: Value::new(),
        }
    }

    /// Add a string argument.
    pub fn arg_string(&mut self, field: &str, value: &str) {
        self._args.set_field_string(field, value.to_string());
    }

    /// Add a double argument.
    pub fn arg_double(&mut self, field: &str, value: f64) {
        self._args.set_field_double(field, value);
    }

    /// Add an int32 argument.
    pub fn arg_int32(&mut self, field: &str, value: i32) {
        self._args.set_field_int32(field, value);
    }

    /// Add a boolean argument (stored as int32 0/1).
    pub fn arg_bool(&mut self, field: &str, value: bool) {
        self._args.set_field_int32(field, value as i32);
    }

    /// Execute the RPC synchronously.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn execute(&mut self, _timeout: f64) -> Result<Value> {
        Err(PvxsError::new(
            "pvAccess network transport not yet implemented — see TODO.md",
        ))
    }
}
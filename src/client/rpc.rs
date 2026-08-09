// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0

use crate::{PvxsError, Result, Value};

/// RPC (Remote Procedure Call) builder.
///
/// Mirrors `pvxs-sys::Rpc` exactly.
///
/// RPC execution transport is not yet implemented.
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
    pub fn arg_string(&mut self, field: &str, value: &str) -> Result<&mut Self> {
        self._args.set_field_string(field, value.to_string());
        Ok(self)
    }

    /// Add a double argument.
    pub fn arg_double(&mut self, field: &str, value: f64) -> Result<&mut Self> {
        self._args.set_field_double(field, value);
        Ok(self)
    }

    /// Add an int32 argument.
    pub fn arg_int32(&mut self, field: &str, value: i32) -> Result<&mut Self> {
        self._args.set_field_int32(field, value);
        Ok(self)
    }

    /// Add a boolean argument (stored as int32 0/1).
    pub fn arg_bool(&mut self, field: &str, value: bool) -> Result<&mut Self> {
        self._args.set_field_int32(field, value as i32);
        Ok(self)
    }

    /// Execute the RPC synchronously.
    ///
    /// RPC execution transport is not yet implemented.
    pub fn execute(self, _timeout: f64) -> Result<Value> {
        Err(PvxsError::new(
            "pvAccess RPC transport not yet implemented — see TODO.md",
        ))
    }
}

#[cfg(feature = "async")]
impl Rpc {
    /// Execute the RPC asynchronously.
    pub async fn execute_async(self, timeout: f64) -> Result<Value> {
        self.execute(timeout)
    }
}

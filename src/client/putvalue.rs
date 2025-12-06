use crate::client::Client;
use crate::error::{Error, Result};
/// Trait for types that can be put to a PV
pub trait PutValue {
    fn put(self, client: &mut Client, pv_name: &str, timeout: f64) -> Result<()>;
}

impl PutValue for f64 {
    fn put(self, client: &mut Client, pv_name: &str, timeout: f64) -> Result<()> {
        client.put_double(pv_name, self, timeout)
    }
}

impl PutValue for i32 {
    fn put(self, client: &mut Client, pv_name: &str, timeout: f64) -> Result<()> {
        client.put_int32(pv_name, self, timeout)
    }
}

impl PutValue for &str {
    fn put(self, _client: &mut Client, _pv_name: &str, _timeout: f64) -> Result<()> {
        Err(Error::TypeConversion { message: "put_string not yet implemented in epics-pvxs-sys - only put_double and put_int32 are available".to_string() })
    }
}

impl PutValue for String {
    fn put(self, client: &mut Client, pv_name: &str, timeout: f64) -> Result<()> {
        self.as_str().put(client, pv_name, timeout)
    }
}

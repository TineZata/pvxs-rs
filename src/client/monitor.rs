use std::collections::{HashMap, VecDeque};
use crate::{PvxsError, Result, Value};
use std::fmt;
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::{Duration, Instant};

/// Internal shared queue between the network driver and the consumer.
struct MonitorInner {
    name: String,
    running: bool,
    connected: bool,
    queue: VecDeque<Value>,
    connect_exception: bool,
    disconnect_exception: bool,
}

type MonitorSubscribers = Vec<Weak<Mutex<MonitorInner>>>;
static MONITOR_REGISTRY: OnceLock<Mutex<HashMap<String, MonitorSubscribers>>> = OnceLock::new();

fn monitor_registry() -> &'static Mutex<HashMap<String, MonitorSubscribers>> {
    MONITOR_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn register_monitor(name: &str, inner: &Arc<Mutex<MonitorInner>>) {
    let mut guard = monitor_registry().lock().unwrap();
    let subscribers = guard.entry(name.to_string()).or_default();
    subscribers.retain(|entry| entry.upgrade().is_some());
    subscribers.push(Arc::downgrade(inner));
}

pub(crate) fn publish_value(name: &str, value: Value) -> usize {
    let mut guard = monitor_registry().lock().unwrap();
    let subscribers = guard.entry(name.to_string()).or_default();
    subscribers.retain(|entry| entry.upgrade().is_some());

    let mut delivered = 0;
    let mut stale_indices = Vec::new();
    for (idx, entry) in subscribers.iter().enumerate() {
        if let Some(inner) = entry.upgrade() {
            let mut state = inner.lock().unwrap();
            state.queue.push_back(value.clone());
            state.connected = true;
            delivered += 1;
        } else {
            stale_indices.push(idx);
        }
    }

    for idx in stale_indices.into_iter().rev() {
        subscribers.remove(idx);
    }

    delivered
}

/// A subscription to value changes for a process variable.
///
/// Mirrors the `pvxs-sys::Monitor` API exactly.
///
/// TODO(network): pop() / try_get_update() will block/return None until the
/// pvAccess transport layer delivers real data.
pub struct Monitor {
    inner: Arc<Mutex<MonitorInner>>,
}

impl Monitor {
    pub(crate) fn new(name: String) -> Self {
        let inner = Arc::new(Mutex::new(MonitorInner {
            name: name.clone(),
            running: false,
            connected: false,
            queue: VecDeque::new(),
            connect_exception: false,
            disconnect_exception: true,
        }));
        register_monitor(&name, &inner);
        Self { inner }
    }

    pub(crate) fn push_update(&mut self, value: Value) {
        self.inner.lock().unwrap().queue.push_back(value);
    }

    pub(crate) fn set_connected(&mut self, connected: bool) {
        self.inner.lock().unwrap().connected = connected;
    }

    /// Start monitoring.
    pub fn start(&mut self) -> Result<()> {
        // TODO(network): open a pvAccess subscription channel.
        self.inner.lock().unwrap().running = true;
        Ok(())
    }

    /// Stop monitoring.
    pub fn stop(&mut self) -> Result<()> {
        // TODO(network): close the subscription channel.
        self.inner.lock().unwrap().running = false;
        Ok(())
    }

    /// Returns `true` if monitoring is active.
    pub fn is_running(&self) -> bool {
        self.inner.lock().unwrap().running
    }

    /// Returns `true` if updates are available in the queue.
    pub fn has_update(&self) -> bool {
        !self.inner.lock().unwrap().queue.is_empty()
    }

    /// Returns `true` if connected to the remote PV.
    pub fn is_connected(&self) -> bool {
        self.inner.lock().unwrap().connected
    }

    /// The PV name being monitored.
    pub fn name(&self) -> String {
        self.inner.lock().unwrap().name.clone()
    }

    /// Get the next update, blocking until one arrives or the timeout elapses.
    ///
    /// TODO(network): will block forever until the transport layer pushes data.
    pub fn get_update(&mut self, timeout: f64) -> Result<Value> {
        let deadline = Instant::now() + Duration::from_secs_f64(timeout);
        loop {
            {
                let mut guard = self.inner.lock().unwrap();
                if let Some(v) = guard.queue.pop_front() {
                    return Ok(v);
                }
            }
            if Instant::now() >= deadline {
                return Err(PvxsError::new("monitor get_update timed out"));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Try to get the next update without blocking.
    pub fn try_get_update(&mut self) -> Result<Option<Value>> {
        Ok(self.inner.lock().unwrap().queue.pop_front())
    }

    /// Pop the next item from the subscription queue (PVXS-style).
    ///
    /// Returns:
    /// - `Ok(Some(Value))` — a new value is available.
    /// - `Ok(None)` — the queue is empty.
    /// - `Err(MonitorEvent::Connected)` — connection event (when `connect_exception` is set).
    /// - `Err(MonitorEvent::Disconnected)` — disconnection event.
    /// - `Err(MonitorEvent::Finished)` — subscription ended.
    pub fn pop(&mut self) -> std::result::Result<Option<Value>, MonitorEvent> {
        Ok(self.inner.lock().unwrap().queue.pop_front())
    }
}


/// Builder for creating monitors with advanced configuration.
///
/// Mirrors `pvxs-sys::MonitorBuilder` exactly.
pub struct MonitorBuilder {
    name: String,
    connect_exception: bool,
    disconnect_exception: bool,
}

impl MonitorBuilder {
    pub (crate) fn new(name: String) -> Self {
        Self {
            name,
            connect_exception: false,
            disconnect_exception: true,
        }
    }

    /// Enable or disable connection exceptions in the monitor queue.
    ///
    /// `true` = throw `MonitorEvent::Connected` on connect.
    /// `false` = suppress connection events (default).
    pub fn connect_exception(mut self, enable: bool) -> Self {
        self.connect_exception = enable;
        self
    }

    /// Enable or disable disconnection exceptions in the monitor queue.
    ///
    /// `true` = throw `MonitorEvent::Disconnected` on disconnect (default).
    /// `false` = suppress disconnection events.
    pub fn disconnect_exception(mut self, enable: bool) -> Self {
        self.disconnect_exception = enable;
        self
    }

    /// Finalise the builder and start the subscription.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn exec(self) -> Result<Monitor> {
        let mut m = Monitor::new(self.name);
        {
            let mut guard = m.inner.lock().unwrap();
            guard.connect_exception = self.connect_exception;
            guard.disconnect_exception = self.disconnect_exception;
        }
        Ok(m)
    }
}

/// Events that can be returned by [`Monitor::pop`].
#[derive(Debug, Clone, PartialEq)]
pub enum MonitorEvent {
    /// Connection event (maskConnected(false)).
    Connected(String),
    /// Disconnection event (maskDisconnected(false)).
    Disconnected(String),
    /// Subscription completed — no more events will arrive.
    Finished(String),
    /// Remote error from the server.
    RemoteError(String),
    /// Client-side error.
    ClientError(String),
}

impl fmt::Display for MonitorEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MonitorEvent::Connected(msg) => write!(f, "Monitor connected: {}", msg),
            MonitorEvent::Disconnected(msg) => write!(f, "Monitor disconnected: {}", msg),
            MonitorEvent::Finished(msg) => write!(f, "Monitor finished: {}", msg),
            MonitorEvent::RemoteError(msg) => write!(f, "Monitor remote error: {}", msg),
            MonitorEvent::ClientError(msg) => write!(f, "Monitor client error: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitor_queue_preserves_fifo_order() {
        let mut monitor = Monitor::new("demo".to_string());

        let mut first = Value::new();
        first.set_field_double("value", 1.0);
        let mut second = Value::new();
        second.set_field_double("value", 2.0);

        monitor.push_update(first);
        monitor.push_update(second);

        let first_out = monitor.try_get_update().unwrap().unwrap();
        assert_eq!(first_out.get_field_double("value").unwrap(), 1.0);

        let second_out = monitor.try_get_update().unwrap().unwrap();
        assert_eq!(second_out.get_field_double("value").unwrap(), 2.0);
    }

    #[test]
    fn monitor_receives_published_updates() {
        let mut monitor = Monitor::new("demo".to_string());
        monitor.start().unwrap();

        let mut value = Value::new();
        value.set_field_double("value", 3.5);

        publish_value("demo", value);

        let received = monitor.try_get_update().unwrap().unwrap();
        assert_eq!(received.get_field_double("value").unwrap(), 3.5);
    }
}

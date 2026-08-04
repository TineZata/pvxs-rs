use std::collections::{HashMap, VecDeque};
use crate::client::ClientConfig;
use crate::{PvxsError, Result, Value};
use std::fmt;
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::Duration;
use epics_libcom_rs::runtime::task;

/// Internal shared queue between the network driver and the consumer.
struct MonitorInner {
    name: String,
    running: bool,
    connected: bool,
    queue: VecDeque<Value>,
    connect_exception: bool,
    disconnect_exception: bool,
    events: VecDeque<MonitorEvent>,
    /// Wakes any thread blocked in `get_update` as soon as a value is pushed.
    notify: Arc<tokio::sync::Notify>,
    /// Optional callback fired when queue transitions from empty to not-empty.
    event_callback: Option<extern "C" fn()>,
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

#[allow(dead_code)]
pub(crate) fn publish_value(name: &str, value: Value) -> usize {
    let mut guard = monitor_registry().lock().unwrap();
    let subscribers = guard.entry(name.to_string()).or_default();
    subscribers.retain(|entry| entry.upgrade().is_some());

    let mut delivered = 0;
    let mut stale_indices = Vec::new();
    let mut notifiers: Vec<Arc<tokio::sync::Notify>> = Vec::new();
    for (idx, entry) in subscribers.iter().enumerate() {
        if let Some(inner) = entry.upgrade() {
            let mut state = inner.lock().unwrap();
            state.queue.push_back(value.clone());
            state.connected = true;
            notifiers.push(Arc::clone(&state.notify));
            delivered += 1;
        } else {
            stale_indices.push(idx);
        }
    }

    for idx in stale_indices.into_iter().rev() {
        subscribers.remove(idx);
    }
    drop(guard);

    // Wake all `get_update` waiters after releasing the registry lock.
    for notify in notifiers {
        notify.notify_waiters();
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
    /// Handle to the tokio runtime used to drive `get_update`'s async wait.
    rt: tokio::runtime::Handle,
    config: ClientConfig,
    session: Option<crate::net::MonitorSession>,
    event_task: Option<tokio::task::JoinHandle<()>>,
}

impl Monitor {
    pub(crate) fn new(name: String, rt: tokio::runtime::Handle, config: ClientConfig) -> Self {
        let notify = Arc::new(tokio::sync::Notify::new());
        let inner = Arc::new(Mutex::new(MonitorInner {
            name: name.clone(),
            running: false,
            connected: false,
            queue: VecDeque::new(),
            connect_exception: false,
            disconnect_exception: true,
            events: VecDeque::new(),
            notify: Arc::clone(&notify),
            event_callback: None,
        }));
        register_monitor(&name, &inner);
        Self {
            inner,
            rt,
            config,
            session: None,
            event_task: None,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn push_update(&mut self, value: Value) {
        let (notify, callback) = {
            let mut guard = self.inner.lock().unwrap();
            let was_empty = guard.queue.is_empty() && guard.events.is_empty();
            guard.queue.push_back(value);
            let callback = if was_empty { guard.event_callback } else { None };
            (guard.notify.clone(), callback)
        };
        if let Some(cb) = callback {
            cb();
        }
        notify.notify_waiters();
    }

    #[allow(dead_code)]
    pub(crate) fn set_connected(&mut self, connected: bool) {
        self.inner.lock().unwrap().connected = connected;
    }

    /// Start monitoring.
    pub fn start(&mut self) -> Result<()> {
        if self.is_running() {
            return Ok(());
        }

        let name = self.name();
        let (session, mut rx) = crate::net::start_monitor(
            self.config.clone(),
            self.rt.clone(),
            name,
            5.0,
        )?;

        let inner = Arc::clone(&self.inner);
        let task = self.rt.spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    crate::net::MonitorNetEvent::Connected => {
                        let (notify, callback) = {
                            let mut guard = inner.lock().unwrap();
                            guard.connected = true;
                            let was_empty = guard.queue.is_empty() && guard.events.is_empty();
                            if guard.connect_exception {
                                guard
                                    .events
                                    .push_back(MonitorEvent::Connected("connected".to_string()));
                            }
                            let callback = if was_empty && !guard.events.is_empty() {
                                guard.event_callback
                            } else {
                                None
                            };
                            (guard.notify.clone(), callback)
                        };
                        if let Some(cb) = callback {
                            cb();
                        }
                        notify.notify_waiters();
                    }
                    crate::net::MonitorNetEvent::Disconnected(msg) => {
                        let (notify, callback) = {
                            let mut guard = inner.lock().unwrap();
                            guard.connected = false;
                            let was_empty = guard.queue.is_empty() && guard.events.is_empty();
                            if guard.disconnect_exception {
                                guard.events.push_back(MonitorEvent::Disconnected(msg));
                            }
                            let callback = if was_empty && !guard.events.is_empty() {
                                guard.event_callback
                            } else {
                                None
                            };
                            (guard.notify.clone(), callback)
                        };
                        if let Some(cb) = callback {
                            cb();
                        }
                        notify.notify_waiters();
                    }
                    crate::net::MonitorNetEvent::Value(value) => {
                        let (notify, callback) = {
                            let mut guard = inner.lock().unwrap();
                            let was_empty = guard.queue.is_empty() && guard.events.is_empty();
                            guard.queue.push_back(value);
                            guard.connected = true;
                            let callback = if was_empty { guard.event_callback } else { None };
                            (guard.notify.clone(), callback)
                        };
                        if let Some(cb) = callback {
                            cb();
                        }
                        notify.notify_waiters();
                    }
                    crate::net::MonitorNetEvent::RemoteError(msg) => {
                        let (notify, callback) = {
                            let mut guard = inner.lock().unwrap();
                            let was_empty = guard.queue.is_empty() && guard.events.is_empty();
                            guard.events.push_back(MonitorEvent::RemoteError(msg));
                            let callback = if was_empty { guard.event_callback } else { None };
                            (guard.notify.clone(), callback)
                        };
                        if let Some(cb) = callback {
                            cb();
                        }
                        notify.notify_waiters();
                    }
                    crate::net::MonitorNetEvent::ClientError(msg) => {
                        let (notify, callback) = {
                            let mut guard = inner.lock().unwrap();
                            let was_empty = guard.queue.is_empty() && guard.events.is_empty();
                            guard.events.push_back(MonitorEvent::ClientError(msg));
                            let callback = if was_empty { guard.event_callback } else { None };
                            (guard.notify.clone(), callback)
                        };
                        if let Some(cb) = callback {
                            cb();
                        }
                        notify.notify_waiters();
                    }
                    crate::net::MonitorNetEvent::Finished => {
                        let (notify, callback) = {
                            let mut guard = inner.lock().unwrap();
                            let was_empty = guard.queue.is_empty() && guard.events.is_empty();
                            guard.connected = false;
                            guard.running = false;
                            guard.events.push_back(MonitorEvent::Finished("finished".to_string()));
                            let callback = if was_empty { guard.event_callback } else { None };
                            (guard.notify.clone(), callback)
                        };
                        if let Some(cb) = callback {
                            cb();
                        }
                        notify.notify_waiters();
                        break;
                    }
                }
            }
        });

        self.session = Some(session);
        self.event_task = Some(task);
        self.inner.lock().unwrap().running = true;
        Ok(())
    }

    /// Stop monitoring.
    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut session) = self.session.take() {
            session.stop();
        }
        if let Some(task) = self.event_task.take() {
            task.abort();
        }
        let mut guard = self.inner.lock().unwrap();
        guard.running = false;
        guard.connected = false;
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
    /// Uses `epics_libcom_rs::runtime::task::sleep_until` so the wait is an
    /// async sleep (tokio timer) rather than a busy-polling thread sleep.
    /// `Notify` wakes the caller immediately when a value is pushed, so
    /// latency is bounded by the tokio task scheduler, not a 10 ms poll interval.
    pub fn get_update(&mut self, timeout: f64) -> Result<Value> {
        let inner = Arc::clone(&self.inner);
        let notify = inner.lock().unwrap().notify.clone();
        let duration = Duration::from_secs_f64(timeout);

        let fut = async move {
            let deadline = task::Instant::now() + duration;
            loop {
                // Subscribe to the next notification *before* checking the
                // queue: a push that fires between the queue check and the
                // select! will still wake us because we registered first.
                let notified = notify.notified();
                {
                    let mut guard = inner.lock().unwrap();
                    if let Some(v) = guard.queue.pop_front() {
                        return Ok(v);
                    }
                }
                epics_libcom_rs::runtime::select! {
                    _ = notified => {}
                    _ = task::sleep_until(deadline) => {
                        return Err(PvxsError::new("monitor get_update timed out"));
                    }
                }
            }
        };

        // Drive the future on the stored runtime.  When called from inside a
        // tokio worker (e.g. spawn_blocking), block_in_place hands off this
        // thread's tasks before parking so the scheduler does not deadlock.
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => tokio::task::block_in_place(|| handle.block_on(fut)),
            Err(_) => self.rt.block_on(fut),
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
        let mut guard = self.inner.lock().unwrap();
        if let Some(event) = guard.events.pop_front() {
            return Err(event);
        }
        Ok(guard.queue.pop_front())
    }
}

impl Drop for Monitor {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}


/// Builder for creating monitors with advanced configuration.
///
/// Mirrors `pvxs-sys::MonitorBuilder` exactly.
pub struct MonitorBuilder {
    name: String,
    connect_exception: bool,
    disconnect_exception: bool,
    event_callback: Option<extern "C" fn()>,
    rt: tokio::runtime::Handle,
    config: ClientConfig,
}

impl MonitorBuilder {
    pub(crate) fn new(name: String, rt: tokio::runtime::Handle, config: ClientConfig) -> Self {
        Self {
            name,
            connect_exception: false,
            disconnect_exception: true,
            event_callback: None,
            rt,
            config,
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

    /// Register an event callback fired when the monitor queue transitions
    /// from empty to not-empty.
    pub fn event(mut self, callback: extern "C" fn()) -> Self {
        self.event_callback = Some(callback);
        self
    }

    /// Finalise the builder and start the subscription.
    ///
    /// TODO(network): pvAccess TCP transport not yet implemented.
    pub fn exec(self) -> Result<Monitor> {
        let m = Monitor::new(self.name, self.rt, self.config);
        {
            let mut guard = m.inner.lock().unwrap();
            guard.connect_exception = self.connect_exception;
            guard.disconnect_exception = self.disconnect_exception;
            guard.event_callback = self.event_callback;
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
        let rt = tokio::runtime::Runtime::new().expect("test runtime");
        let mut monitor = Monitor::new(
            "fifo_test".to_string(),
            rt.handle().clone(),
            ClientConfig::default(),
        );

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
        let rt = tokio::runtime::Runtime::new().expect("test runtime");
        let mut monitor = Monitor::new(
            "pub_test".to_string(),
            rt.handle().clone(),
            ClientConfig::default(),
        );

        let mut value = Value::new();
        value.set_field_double("value", 3.5);

        publish_value("pub_test", value);

        let received = monitor.try_get_update().unwrap().unwrap();
        assert_eq!(received.get_field_double("value").unwrap(), 3.5);
    }

    #[test]
    fn get_update_wakes_on_publish() {
        let rt = tokio::runtime::Runtime::new().expect("test runtime");
        let mut monitor = Monitor::new(
            "wake_test".to_string(),
            rt.handle().clone(),
            ClientConfig::default(),
        );

        let mut val = Value::new();
        val.set_field_double("value", 7.0);

        // Publish from a separate thread while get_update blocks.
        let val_clone = val.clone();
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            publish_value("wake_test", val_clone);
        });

        let received = monitor.get_update(5.0).unwrap();
        assert_eq!(received.get_field_double("value").unwrap(), 7.0);
        handle.join().unwrap();
    }
}

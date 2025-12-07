use crate::error::{Error, Result};
use crate::types::Value;
use epics_pvxs_sys::{Monitor as PvxsMonitor, MonitorBuilder as PvxsMonitorBuilder};
use tracing::{debug, info, warn};
use std::sync::{Arc, Mutex};

/// Event types that can occur during monitoring
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonitorEvent {
    /// Monitor connected to PV
    Connected,
    /// Monitor disconnected from PV
    Disconnected,
    /// Monitor finished normally (no more events will be received)
    Finished,
    /// Remote error from server
    RemoteError(String),
    /// Client-side error
    ClientError(String),
}

/// Callback type for monitor events
///
/// Called when connection state changes or errors occur.
pub type MonitorEventCallback = Box<dyn FnMut(&str, MonitorEvent) + Send>;

/// High-level monitor for receiving PV updates
///
/// Provides ergonomic APIs beyond the raw -sys bindings:
/// - Automatic start/stop lifecycle management
/// - Iterator support for draining updates
/// - Event callbacks for connection state changes
/// - Convenience methods for common patterns
/// - Better error handling
///
/// # Examples
///
/// ```rust,no_run
/// use pvxs::Client;
///
/// let mut client = Client::new()?;
/// let mut monitor = client.monitor("MY:PV")?;
///
/// // Iterator pattern - drains all pending updates
/// for value in monitor.drain() {
///     println!("Value: {}", value);
/// }
///
/// // With event callbacks
/// let mut monitor = client.monitor_builder("MY:PV")?
///     .on_connected(|name| println!("{} connected!", name))
///     .on_disconnected(|name| println!("{} disconnected!", name))
///     .build()?;
/// # Ok::<(), pvxs::Error>(())
/// ```
pub struct Monitor {
    pub(super) inner: PvxsMonitor,
    pub(super) pv_name: String,
    started: bool,
    event_callback: Option<Arc<Mutex<MonitorEventCallback>>>,
}

impl Monitor {
    pub(super) fn new(inner: PvxsMonitor, pv_name: String) -> Self {
        Self { 
            inner, 
            pv_name,
            started: false,
            event_callback: None,
        }
    }

    pub(super) fn with_callback(
        inner: PvxsMonitor, 
        pv_name: String, 
        callback: Option<Arc<Mutex<MonitorEventCallback>>>
    ) -> Self {
        Self { 
            inner, 
            pv_name,
            started: false,
            event_callback: callback,
        }
    }

    /// Start the monitor
    ///
    /// Begins receiving updates from the PV. Idempotent - safe to call multiple times.
    pub fn start(&mut self) {
        if !self.started {
            debug!("Starting monitor for PV: {}", self.pv_name);
            self.inner.start();
            self.started = true;
        }
    }

    /// Stop the monitor
    ///
    /// Stops receiving updates. Idempotent - safe to call multiple times.
    pub fn stop(&mut self) {
        if self.started {
            debug!("Stopping monitor for PV: {}", self.pv_name);
            self.inner.stop();
            self.started = false;
        }
    }

    /// Check if the monitor is currently started
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// Get the next update from the queue
    ///
    /// Returns `None` if the queue is empty. Processes monitor events and triggers callbacks.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Client;
    /// # let mut client = Client::new().unwrap();
    /// let mut monitor = client.monitor("MY:PV")?;
    /// 
    /// loop {
    ///     match monitor.next_update()? {
    ///         Some(value) => println!("Value: {}", value),
    ///         None => break, // Queue empty
    ///     }
    /// }
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn next_update(&mut self) -> Result<Option<Value>> {
        match self.inner.pop() {
            Ok(Some(pvxs_value)) => Ok(Some(Value::from_pvxs(pvxs_value))),
            Ok(None) => Ok(None),
            Err(sys_event) => {
                // Convert sys event to our event type and trigger callback
                let event = Self::convert_event(&sys_event);
                let event_name = format!("{:?}", event);
                
                match &event {
                    MonitorEvent::Connected => {
                        info!("Monitor '{}' connected", self.pv_name);
                    }
                    MonitorEvent::Disconnected => {
                        warn!("Monitor '{}' disconnected", self.pv_name);
                    }
                    MonitorEvent::Finished => {
                        info!("Monitor '{}' finished", self.pv_name);
                    }
                    MonitorEvent::RemoteError(msg) => {
                        warn!("Monitor '{}' remote error: {}", self.pv_name, msg);
                    }
                    MonitorEvent::ClientError(msg) => {
                        warn!("Monitor '{}' client error: {}", self.pv_name, msg);
                    }
                }

                // Trigger user callback if registered
                if let Some(callback) = &self.event_callback {
                    if let Ok(mut cb) = callback.lock() {
                        cb(&self.pv_name, event);
                    }
                }

                debug!("Monitor event for {}: {}", self.pv_name, event_name);
                Ok(None)
            }
        }
    }

    /// Convert sys-level MonitorEvent to our high-level type
    fn convert_event(sys_event: &epics_pvxs_sys::MonitorEvent) -> MonitorEvent {
        use epics_pvxs_sys::MonitorEvent as SysEvent;
        
        match sys_event {
            SysEvent::Connected(_) => MonitorEvent::Connected,
            SysEvent::Disconnected(_) => MonitorEvent::Disconnected,
            SysEvent::Finished(_) => MonitorEvent::Finished,
            SysEvent::RemoteError(msg) => MonitorEvent::RemoteError(msg.clone()),
            SysEvent::ClientError(msg) => MonitorEvent::ClientError(msg.clone()),
        }
    }

    /// Drain all pending updates from the queue
    ///
    /// Returns an iterator that yields all currently queued updates.
    /// Stops when the queue is empty.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Client;
    /// # let mut client = Client::new().unwrap();
    /// let mut monitor = client.monitor("MY:PV")?;
    ///
    /// // Process all pending updates
    /// for value in monitor.drain() {
    ///     println!("Value: {}", value);
    /// }
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn drain(&mut self) -> MonitorDrain<'_> {
        MonitorDrain { monitor: self }
    }

    /// Wait for and return the next update
    ///
    /// Blocks until an update is available or the timeout expires.
    /// This is more efficient than polling `next_update()` in a loop.
    ///
    /// # Arguments
    ///
    /// * `timeout_ms` - Maximum time to wait in milliseconds
    ///
    /// # Returns
    ///
    /// * `Ok(Some(value))` - Update received
    /// * `Ok(None)` - Timeout expired with no update
    /// * `Err` - Error occurred
    pub fn wait_for_update(&mut self, timeout_ms: u64) -> Result<Option<Value>> {
        use std::thread;
        use std::time::{Duration, Instant};
        
        let timeout = Duration::from_millis(timeout_ms);
        let start = Instant::now();
        
        loop {
            if let Some(value) = self.next_update()? {
                return Ok(Some(value));
            }
            
            if start.elapsed() >= timeout {
                return Ok(None);
            }
            
            // Small sleep to avoid busy-waiting
            thread::sleep(Duration::from_millis(10));
        }
    }

    /// Check if the monitor is connected to the PV
    pub fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    /// Check if there are updates available
    ///
    /// Returns `true` if calling `next_update()` would return data immediately.
    pub fn has_update(&self) -> bool {
        self.inner.has_update()
    }

    /// Get the PV name being monitored
    pub fn name(&self) -> &str {
        &self.pv_name
    }

    /// Collect all pending updates into a Vec
    ///
    /// Convenience method that drains the queue and collects into a vector.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Client;
    /// # let mut client = Client::new().unwrap();
    /// let mut monitor = client.monitor("MY:PV")?;
    /// let updates = monitor.collect_updates();
    /// println!("Received {} updates", updates.len());
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn collect_updates(&mut self) -> Vec<Value> {
        self.drain().collect()
    }

    /// Get the latest update, discarding intermediate values
    ///
    /// Returns the most recent value from the queue, skipping over older updates.
    /// Useful when you only care about the current state.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Client;
    /// # let mut client = Client::new().unwrap();
    /// let mut monitor = client.monitor("MY:PV")?;
    ///
    /// // Only care about the latest value
    /// if let Some(latest) = monitor.latest_update()? {
    ///     println!("Latest value: {}", latest);
    /// }
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn latest_update(&mut self) -> Result<Option<Value>> {
        let mut latest = None;
        while let Some(value) = self.next_update()? {
            latest = Some(value);
        }
        Ok(latest)
    }
}

impl Drop for Monitor {
    fn drop(&mut self) {
        self.stop();
    }
}

impl std::fmt::Debug for Monitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Monitor")
            .field("pv_name", &self.pv_name)
            .field("started", &self.started)
            .field("connected", &self.is_connected())
            .field("has_update", &self.has_update())
            .finish()
    }
}

/// Iterator that drains pending updates from a monitor
///
/// Created by calling [`Monitor::drain()`].
pub struct MonitorDrain<'a> {
    monitor: &'a mut Monitor,
}

impl<'a> Iterator for MonitorDrain<'a> {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        self.monitor.next_update().ok().flatten()
    }
}

/// Builder for configuring a monitor
///
/// Allows configuration of event handling and callbacks before creating the monitor.
///
/// # Example
///
/// ```rust,no_run
/// use pvxs::Client;
///
/// extern "C" fn my_callback() {
///     println!("New data!");
/// }
///
/// let mut client = Client::new()?;
/// let monitor = client.monitor_builder("MY:PV")?
///     .event(my_callback)
///     .build()?;
/// # Ok::<(), pvxs::Error>(())
/// ```
pub struct MonitorBuilder {
    pub(super) inner: PvxsMonitorBuilder,
    pub(super) pv_name: String,
    auto_start: bool,
    event_callback: Option<Arc<Mutex<MonitorEventCallback>>>,
}

impl MonitorBuilder {
    pub(super) fn new(inner: PvxsMonitorBuilder, pv_name: String) -> Self {
        Self { 
            inner, 
            pv_name,
            auto_start: true, // Auto-start by default for convenience
            event_callback: None,
        }
    }

    /// Control whether the monitor starts automatically when built
    ///
    /// By default, monitors start automatically. Set to `false` if you want
    /// to manually control when the monitor starts.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Client;
    /// # let mut client = Client::new().unwrap();
    /// let mut monitor = client.monitor_builder("MY:PV")?
    ///     .auto_start(false)  // Don't start automatically
    ///     .build()?;
    ///
    /// // Manually start later
    /// monitor.start();
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn auto_start(mut self, enable: bool) -> Self {
        self.auto_start = enable;
        self
    }

    /// Register a callback for when the monitor connects to the PV
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Client;
    /// # let mut client = Client::new().unwrap();
    /// let monitor = client.monitor_builder("MY:PV")?
    ///     .on_connected(|pv_name| {
    ///         println!("Monitor connected to {}", pv_name);
    ///     })
    ///     .build()?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn on_connected<F>(self, mut callback: F) -> Self
    where
        F: FnMut(&str) + Send + 'static,
    {
        self.on_event(move |name, event| {
            if matches!(event, MonitorEvent::Connected) {
                callback(name);
            }
        })
    }

    /// Register a callback for when the monitor disconnects from the PV
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Client;
    /// # let mut client = Client::new().unwrap();
    /// let monitor = client.monitor_builder("MY:PV")?
    ///     .on_disconnected(|pv_name| {
    ///         eprintln!("Monitor disconnected from {}", pv_name);
    ///     })
    ///     .build()?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn on_disconnected<F>(self, mut callback: F) -> Self
    where
        F: FnMut(&str) + Send + 'static,
    {
        self.on_event(move |name, event| {
            if matches!(event, MonitorEvent::Disconnected) {
                callback(name);
            }
        })
    }

    /// Register a callback for when the monitor finishes normally
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Client;
    /// # let mut client = Client::new().unwrap();
    /// let monitor = client.monitor_builder("MY:PV")?
    ///     .on_finished(|pv_name| {
    ///         println!("Monitor {} finished", pv_name);
    ///     })
    ///     .build()?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn on_finished<F>(self, mut callback: F) -> Self
    where
        F: FnMut(&str) + Send + 'static,
    {
        self.on_event(move |name, event| {
            if matches!(event, MonitorEvent::Finished) {
                callback(name);
            }
        })
    }

    /// Register a callback for remote errors
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Client;
    /// # let mut client = Client::new().unwrap();
    /// let monitor = client.monitor_builder("MY:PV")?
    ///     .on_error(|pv_name, error_msg| {
    ///         eprintln!("Error on {}: {}", pv_name, error_msg);
    ///     })
    ///     .build()?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn on_error<F>(self, mut callback: F) -> Self
    where
        F: FnMut(&str, &str) + Send + 'static,
    {
        self.on_event(move |name, event| {
            match event {
                MonitorEvent::RemoteError(ref msg) | MonitorEvent::ClientError(ref msg) => {
                    callback(name, msg);
                }
                _ => {}
            }
        })
    }

    /// Register a callback for all monitor events
    ///
    /// This is the most flexible option, allowing you to handle all event types.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Client;
    /// # use pvxs::MonitorEvent;
    /// let mut client = Client::new().unwrap();
    /// let monitor = client.monitor_builder("MY:PV")?
    ///     .on_event(|pv_name, event| {
    ///         match event {
    ///             MonitorEvent::Connected => println!("Connected to {}", pv_name),
    ///             MonitorEvent::Disconnected => println!("Disconnected from {}", pv_name),
    ///             MonitorEvent::RemoteError(msg) => eprintln!("Error: {}", msg),
    ///             _ => {}
    ///         }
    ///     })
    ///     .build()?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn on_event<F>(mut self, mut callback: F) -> Self
    where
        F: FnMut(&str, MonitorEvent) + Send + 'static,
    {
        let existing = self.event_callback.take();
        
        self.event_callback = Some(Arc::new(Mutex::new(Box::new(move |name: &str, event: MonitorEvent| {
            // Call new callback
            callback(name, event.clone());
            
            // Call existing callback if any
            if let Some(ref existing_cb) = existing {
                if let Ok(mut cb) = existing_cb.lock() {
                    cb(name, event);
                }
            }
        }))));
        
        self
    }

    /// Register a callback function to be invoked when the queue goes from empty to not-empty
    ///
    /// The callback should be an `extern "C" fn()` function. It will be called when new
    /// data arrives in an empty queue. Note: The callback fires on queue state transitions
    /// (empty -> not-empty), so you should drain the queue with `drain()` to reset the state.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Client;
    ///
    /// extern "C" fn my_callback() {
    ///     println!("New data available!");
    /// }
    ///
    /// let mut client = Client::new()?;
    /// let monitor = client.monitor_builder("MY:PV")?
    ///     .event(my_callback)
    ///     .build()?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn event(mut self, callback: extern "C" fn()) -> Self {
        self.inner = self.inner.event(callback);
        self
    }

    /// Build and create the monitor
    ///
    /// This consumes the builder and returns the configured `Monitor`.
    /// By default, the monitor is started automatically unless `auto_start(false)` was called.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pvxs::Client;
    /// # let mut client = Client::new().unwrap();
    /// let mut monitor = client.monitor_builder("MY:PV")?
    ///     .on_connected(|name| println!("{} connected", name))
    ///     .on_disconnected(|name| println!("{} disconnected", name))
    ///     .build()?;
    ///
    /// // Monitor is already started and receiving updates
    /// for value in monitor.drain() {
    ///     println!("Value: {}", value);
    /// }
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn build(self) -> Result<Monitor> {
        debug!("Building monitor for PV: {} (auto_start: {})", self.pv_name, self.auto_start);
        let pvxs_monitor = self.inner
            .exec()
            .map_err(|e| Error::from(e))?;
        
        let mut monitor = Monitor::with_callback(
            pvxs_monitor,
            self.pv_name,
            self.event_callback,
        );

        if self.auto_start {
            monitor.start();
        }

        Ok(monitor)
    }

    /// Alias for `build()` for backward compatibility
    #[deprecated(since = "0.2.0", note = "Use `build()` instead")]
    pub fn exec(self) -> Result<Monitor> {
        self.build()
    }
}


use crate::error::{Error, Result};
use crate::types::Value;
use epics_pvxs_sys::{Monitor as PvxsMonitor, MonitorBuilder as PvxsMonitorBuilder};
use tracing::debug;

/// High-level monitor for receiving PV updates
///
/// A monitor watches a PV and queues updates. Updates can be retrieved using
/// `pop()`. The monitor must be started with `start()` before it will receive updates.
pub struct Monitor {
    pub(super) inner: PvxsMonitor,
    pub(super) pv_name: String,
}

impl Monitor {
    pub(super) fn new(inner: PvxsMonitor, pv_name: String) -> Self {
        Self { inner, pv_name }
    }

    /// Start the monitor
    ///
    /// The monitor will begin receiving updates after this is called.
    pub fn start(&mut self) {
        debug!("Starting monitor for PV: {}", self.pv_name);
        self.inner.start();
    }

    /// Stop the monitor
    ///
    /// The monitor will stop receiving updates after this is called.
    pub fn stop(&mut self) {
        debug!("Stopping monitor for PV: {}", self.pv_name);
        self.inner.stop();
    }

    /// Pop the next update from the queue
    ///
    /// This retrieves the next update from the monitor's internal queue.
    /// Returns `Ok(Some(value))` if an update is available, `Ok(None)` if the
    /// queue is empty, or `Err` for connection/disconnection events.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pvxs::Client;
    ///
    /// let mut client = Client::new()?;
    /// let mut monitor = client.monitor("MY:PV:NAME")?;
    /// monitor.start();
    ///
    /// loop {
    ///     match monitor.pop() {
    ///         Ok(Some(value)) => println!("Value: {}", value),
    ///         Ok(None) => break, // Queue empty
    ///         Err(e) => println!("Event: {}", e),
    ///     }
    /// }
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn pop(&mut self) -> Result<Option<Value>> {
        match self.inner.pop() {
            Ok(Some(pvxs_value)) => Ok(Some(Value::from_pvxs(pvxs_value))),
            Ok(None) => Ok(None),
            Err(e) => Err(Error::from(e)),
        }
    }

    /// Check if the monitor is connected to the PV
    pub fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    /// Check if there are updates available
    ///
    /// Returns `true` if calling `pop()` would return data immediately.
    pub fn has_update(&self) -> bool {
        self.inner.has_update()
    }

    /// Get the PV name being monitored
    pub fn name(&self) -> &str {
        &self.pv_name
    }
}

impl std::fmt::Debug for Monitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Monitor")
            .field("pv_name", &self.pv_name)
            .field("connected", &self.is_connected())
            .finish()
    }
}

/// Builder for configuring a monitor
///
/// Allows configuration of event handling and callbacks before creating the monitor.
pub struct MonitorBuilder {
    pub(super) inner: PvxsMonitorBuilder,
    pub(super) pv_name: String,
}

impl MonitorBuilder {
    pub(super) fn new(inner: PvxsMonitorBuilder, pv_name: String) -> Self {
        Self { inner, pv_name }
    }

    /// Enable or disable connection event notifications
    ///
    /// When enabled, connection events will be added to the queue and can be
    /// retrieved with `pop()` (they will return `Err`).
    pub fn connection_events(mut self, enable: bool) -> Self {
        self.inner = self.inner.mask_connected(enable);
        self
    }

    /// Enable or disable disconnection event notifications
    ///
    /// When enabled, disconnection events will be added to the queue and can be
    /// retrieved with `pop()` (they will return `Err`).
    pub fn disconnection_events(mut self, enable: bool) -> Self {
        self.inner = self.inner.mask_disconnected(enable);
        self
    }

    /// Register a callback function to be invoked when the queue goes from empty to not-empty
    ///
    /// The callback should be an `extern "C" fn()` function. It will be called when new
    /// data arrives in an empty queue. Note: The callback fires on queue state transitions
    /// (empty -> not-empty), so you should drain the queue with `pop()` to reset the state.
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
    /// let mut monitor = client.monitor_builder("MY:PV:NAME")?
    ///     .event(my_callback)
    ///     .exec()?;
    /// # Ok::<(), pvxs::Error>(())
    /// ```
    pub fn event(mut self, callback: extern "C" fn()) -> Self {
        self.inner = self.inner.event(callback);
        self
    }

    /// Execute the builder and create the monitor
    ///
    /// This consumes the builder and returns the configured `Monitor`.
    pub fn exec(self) -> Result<Monitor> {
        debug!("Executing monitor builder for PV: {}", self.pv_name);
        let pvxs_monitor = self.inner
            .exec()
            .map_err(|e| Error::from(e))?;
        
        Ok(Monitor {
            inner: pvxs_monitor,
            pv_name: self.pv_name,
        })
    }
}


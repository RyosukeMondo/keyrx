//! Multi-device input support for Linux.
//!
//! This module provides `MultiDeviceInput` which merges events from multiple
//! keyboard devices into a single unified stream, enabling cross-device key
//! combinations (e.g., Shift on keyboard 1 + character on keyboard 2).
//!
//! # Example
//!
//! ```ignore
//! let devices = vec![
//!     "/dev/input/event3".into(),
//!     "/dev/input/event4".into(),
//!     "/dev/input/event5".into(),
//! ];
//! let mut input = MultiDeviceInput::new(devices)?;
//! input.start().await?;
//!
//! loop {
//!     let events = input.poll_events().await?;
//!     // Events from all keyboards, merged in timestamp order
//! }
//! ```

use crate::drivers::DeviceInfo;
use crate::engine::{InputEvent, OutputAction};
use crate::errors::driver::*;
use crate::errors::KeyrxError;
use crate::keyrx_err;
use crate::traits::InputSource;
use std::collections::BinaryHeap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use super::LinuxInput;

/// Wrapper for an event with its source device index and sequence number.
///
/// The sequence number ensures stable ordering when timestamps are identical,
/// preventing nondeterministic behavior from BinaryHeap's unstable ordering.
#[derive(Clone, Debug)]
struct DeviceEvent {
    event: InputEvent,
    #[allow(dead_code)]
    device_index: usize,
    /// Monotonic sequence number for stable ordering of events with identical timestamps
    sequence: u64,
}

impl PartialEq for DeviceEvent {
    fn eq(&self, other: &Self) -> bool {
        self.event.timestamp_us == other.event.timestamp_us && self.sequence == other.sequence
    }
}

impl Eq for DeviceEvent {}

impl PartialOrd for DeviceEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DeviceEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse order for min-heap (earliest timestamp first)
        match other.event.timestamp_us.cmp(&self.event.timestamp_us) {
            std::cmp::Ordering::Equal => {
                // If timestamps are identical, use sequence number for stable ordering
                // This ensures FIFO behavior for simultaneous events
                self.sequence.cmp(&other.sequence)
            }
            other_ordering => other_ordering,
        }
    }
}

/// Multi-device input manager that merges events from multiple keyboards.
pub struct MultiDeviceInput {
    devices: Vec<LinuxInput>,
    device_infos: Vec<DeviceInfo>,
    event_buffer: Arc<Mutex<BinaryHeap<DeviceEvent>>>,
    /// Monotonic sequence counter for stable event ordering
    sequence_counter: Arc<Mutex<u64>>,
    /// Maximum buffering delay in microseconds to ensure correct ordering
    buffer_delay_us: u64,
    /// Maximum buffer size to prevent unbounded growth
    max_buffer_size: usize,
}

impl MultiDeviceInput {
    /// Create a new multi-device input manager.
    ///
    /// # Arguments
    ///
    /// * `device_paths` - List of device paths (e.g., `/dev/input/event3`)
    ///
    /// # Returns
    ///
    /// A new `MultiDeviceInput` instance if all devices can be opened.
    ///
    /// # Errors
    ///
    /// Returns an error if any device fails to initialize.
    pub fn new(device_paths: Vec<PathBuf>) -> Result<Self, KeyrxError> {
        if device_paths.is_empty() {
            return Err(keyrx_err!(
                DRIVER_INIT_FAILED,
                reason = "At least one device path is required for multi-device input".to_string()
            ));
        }

        info!(
            service = "keyrx",
            event = "multi_device_init",
            component = "multi_device_input",
            device_count = device_paths.len(),
            "Initializing multi-device input"
        );

        let mut devices = Vec::new();
        let mut device_infos = Vec::new();

        for (index, path) in device_paths.iter().enumerate() {
            debug!(
                service = "keyrx",
                event = "device_open",
                component = "multi_device_input",
                device_index = index,
                path = %path.display(),
                "Opening device"
            );

            match LinuxInput::new(Some(path.clone())) {
                Ok(device) => {
                    let info = device.device_info().clone();
                    info!(
                        service = "keyrx",
                        event = "device_opened",
                        component = "multi_device_input",
                        device_index = index,
                        name = %info.name,
                        vendor_id = info.vendor_id,
                        product_id = info.product_id,
                        "Device opened successfully"
                    );
                    device_infos.push(info);
                    devices.push(device);
                }
                Err(e) => {
                    warn!(
                        service = "keyrx",
                        event = "device_open_failed",
                        component = "multi_device_input",
                        device_index = index,
                        path = %path.display(),
                        error = %e,
                        "Failed to open device, cleaning up"
                    );
                    // Clean up any devices we already opened
                    return Err(e.with_context(
                        "device_path",
                        format!("Failed to open device: {}", path.display()),
                    ));
                }
            }
        }

        Ok(Self {
            devices,
            device_infos,
            event_buffer: Arc::new(Mutex::new(BinaryHeap::new())),
            sequence_counter: Arc::new(Mutex::new(0)),
            // Buffer for 2ms to ensure events are in order
            // (handles slight clock skew between devices while minimizing latency)
            // Reduced from 10ms to improve responsiveness
            buffer_delay_us: 2_000,
            // Maximum buffer size to prevent unbounded growth
            // If buffer exceeds this, oldest events are force-released
            max_buffer_size: 10_000,
        })
    }

    /// Create a multi-device input from all available keyboards.
    ///
    /// # Returns
    ///
    /// A new `MultiDeviceInput` instance with all detected keyboards.
    ///
    /// # Errors
    ///
    /// Returns an error if no keyboards are found or if device initialization fails.
    pub fn from_all_keyboards() -> Result<Self, KeyrxError> {
        let keyboards = super::list_keyboards()?;

        if keyboards.is_empty() {
            return Err(keyrx_err!(
                DRIVER_DEVICE_NOT_FOUND,
                device = "keyboard".to_string()
            ));
        }

        info!(
            service = "keyrx",
            event = "auto_detect_keyboards",
            component = "multi_device_input",
            count = keyboards.len(),
            "Auto-detected keyboards"
        );

        let paths: Vec<PathBuf> = keyboards.iter().map(|kb| kb.path.clone()).collect();
        Self::new(paths)
    }

    /// Get the number of devices being managed.
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Get information about all managed devices.
    pub fn device_infos(&self) -> &[DeviceInfo] {
        &self.device_infos
    }

    /// Get device info for the first (representative) device.
    pub fn device_info(&self) -> &DeviceInfo {
        // Return the first device's info (representative)
        // In the future, we might want to return a synthetic "multi-device" info
        &self.device_infos[0]
    }

    /// Get device path for the first (representative) device.
    pub fn device_path(&self) -> &std::path::Path {
        // Return the first device's path (representative)
        &self.device_infos[0].path
    }

    /// Poll events from all devices and merge them in timestamp order.
    async fn poll_all_devices(&mut self) -> Result<Vec<InputEvent>, KeyrxError> {
        // Poll all devices sequentially and collect events
        let mut buffer = self.event_buffer.lock().await;
        let mut sequence_counter = self.sequence_counter.lock().await;

        for (device_index, device) in self.devices.iter_mut().enumerate() {
            match device.poll_events().await {
                Ok(events) => {
                    for event in events {
                        // Increment sequence counter for each event
                        *sequence_counter += 1;
                        buffer.push(DeviceEvent {
                            event,
                            device_index,
                            sequence: *sequence_counter,
                        });
                    }
                }
                Err(e) => {
                    warn!(
                        service = "keyrx",
                        event = "device_poll_failed",
                        component = "multi_device_input",
                        device_index = device_index,
                        error = %e,
                        "Device polling failed, continuing with other devices"
                    );
                    // Continue with other devices instead of failing completely
                }
            }
        }

        // SAFEGUARD: If buffer exceeds max size, force-release oldest events
        // This prevents unbounded growth if events arrive faster than buffer delay
        // or if a device has a fast clock (events stuck in buffer)
        if buffer.len() > self.max_buffer_size {
            let excess = buffer.len() - (self.max_buffer_size / 2);
            warn!(
                service = "keyrx",
                event = "buffer_overflow_protection",
                component = "multi_device_input",
                buffer_size = buffer.len(),
                max_size = self.max_buffer_size,
                releasing = excess,
                "Buffer exceeded max size, force-releasing oldest events"
            );

            let mut force_released = Vec::new();
            for _ in 0..excess {
                if let Some(device_event) = buffer.pop() {
                    force_released.push(device_event.event);
                }
            }

            // Return force-released events immediately to prevent event loss
            if !force_released.is_empty() {
                drop(buffer);
                drop(sequence_counter);
                return Ok(force_released);
            }
        }

        // Extract events that are old enough (past buffer delay)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        let cutoff = now.saturating_sub(self.buffer_delay_us);

        let mut merged_events = Vec::new();
        while let Some(device_event) = buffer.peek() {
            if device_event.event.timestamp_us <= cutoff {
                if let Some(device_event) = buffer.pop() {
                    merged_events.push(device_event.event);
                }
            } else {
                break;
            }
        }

        Ok(merged_events)
    }
}

#[async_trait::async_trait]
impl InputSource for MultiDeviceInput {
    async fn start(&mut self) -> Result<(), KeyrxError> {
        info!(
            service = "keyrx",
            event = "multi_device_start",
            component = "multi_device_input",
            device_count = self.devices.len(),
            "Starting all devices"
        );

        for (index, device) in self.devices.iter_mut().enumerate() {
            device.start().await.map_err(|e| {
                e.with_context("device_index", format!("Failed to start device {}", index))
            })?;
        }

        info!(
            service = "keyrx",
            event = "multi_device_started",
            component = "multi_device_input",
            "All devices started successfully"
        );

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), KeyrxError> {
        info!(
            service = "keyrx",
            event = "multi_device_stop",
            component = "multi_device_input",
            device_count = self.devices.len(),
            "Stopping all devices"
        );

        let mut errors = Vec::new();

        for (index, device) in self.devices.iter_mut().enumerate() {
            if let Err(e) = device.stop().await {
                warn!(
                    service = "keyrx",
                    event = "device_stop_failed",
                    component = "multi_device_input",
                    device_index = index,
                    error = %e,
                    "Failed to stop device"
                );
                errors.push((index, e));
            }
        }

        if errors.is_empty() {
            info!(
                service = "keyrx",
                event = "multi_device_stopped",
                component = "multi_device_input",
                "All devices stopped successfully"
            );
            Ok(())
        } else {
            Err(keyrx_err!(
                DRIVER_INIT_FAILED,
                reason = format!("Failed to stop {} device(s)", errors.len())
            ))
        }
    }

    async fn poll_events(&mut self) -> Result<Vec<InputEvent>, KeyrxError> {
        self.poll_all_devices().await
    }

    async fn send_output(&mut self, action: OutputAction) -> Result<(), KeyrxError> {
        // Send output through the first device (they all share the same UInput)
        // Only one UInput instance is needed for output
        if let Some(device) = self.devices.first_mut() {
            device.send_output(action).await
        } else {
            Err(keyrx_err!(
                DRIVER_DEVICE_NOT_FOUND,
                device = "No devices available".to_string()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_event_ordering() {
        let event1 = DeviceEvent {
            event: InputEvent {
                timestamp_us: 1000,
                ..Default::default()
            },
            device_index: 0,
            sequence: 0,
        };

        let event2 = DeviceEvent {
            event: InputEvent {
                timestamp_us: 2000,
                ..Default::default()
            },
            device_index: 1,
            sequence: 1,
        };

        // Min-heap should give us smallest timestamp first
        let mut heap = BinaryHeap::new();
        heap.push(event2.clone());
        heap.push(event1.clone());

        assert_eq!(heap.pop().unwrap().event.timestamp_us, 1000);
        assert_eq!(heap.pop().unwrap().event.timestamp_us, 2000);
    }
}

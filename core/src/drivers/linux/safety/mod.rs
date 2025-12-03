//! Safety wrappers for Linux driver operations.
//!
//! This module contains safe abstractions over Linux evdev and uinput operations used
//! by the keyboard driver. Each wrapper encapsulates error-prone operations with proper
//! RAII cleanup and comprehensive error handling.
//!
//! # Safety Architecture
//!
//! The Linux driver interacts with kernel interfaces that can fail in various ways:
//! - **evdev devices** (`/dev/input/eventX`): Reading raw input events
//! - **uinput device** (`/dev/uinput`): Creating virtual keyboard for injection
//! - **Device grabbing**: Exclusive access to input devices
//! - **Permissions**: Access to device files requires proper groups/capabilities
//!
//! Each operation is isolated into a dedicated wrapper type that:
//! 1. Validates permissions and capabilities upfront
//! 2. Provides RAII cleanup (e.g., ungrab devices, close file descriptors)
//! 3. Handles disconnection gracefully (hot-plug scenarios)
//! 4. Returns actionable errors with recovery hints
//!
//! # Module Organization
//!
//! - `device`: SafeDevice wrapper for evdev device operations
//! - `uinput`: SafeUinput wrapper for virtual device creation
//! - `permissions`: Permission checking with helpful error messages
//!
//! # Permission Model
//!
//! Linux input devices require specific permissions:
//! - **Read access** to `/dev/input/eventX` (usually via `input` group)
//! - **Write access** to `/dev/uinput` (for virtual device creation)
//! - **Capabilities**: CAP_SYS_ADMIN or group membership
//!
//! The safety layer checks these upfront and provides actionable error messages:
//! - "Add user to 'input' group: `sudo usermod -aG input $USER`"
//! - "Configure udev rules for uinput access"
//! - "Run with elevated privileges (not recommended)"
//!
//! # Error Handling
//!
//! All operations return `Result<T, DriverError>` with context:
//! - Device path for identification
//! - Errno codes for debugging
//! - Recovery suggestions for users
//! - Retryability flags for temporary errors
//!
//! # Device Lifecycle
//!
//! Devices can disconnect at any time (USB keyboards unplugged, bluetooth disconnects).
//! Safety wrappers handle this gracefully:
//! 1. Detect disconnection via ENODEV errors
//! 2. Return DriverError::DeviceDisconnected
//! 3. Engine layer handles reconnection logic
//! 4. No panics, no undefined behavior
//!
//! # Example Usage
//!
//! ```no_run
//! use keyrx_core::drivers::linux::safety::device::SafeDevice;
//! use std::time::Duration;
//!
//! // Open and grab device with automatic ungrab on drop
//! let mut device = SafeDevice::open("/dev/input/event3")?;
//!
//! // Read events with timeout
//! while let Some(event) = device.read_event(Duration::from_millis(100))? {
//!     // Process input event
//! }
//!
//! // Device is automatically ungrabbed and closed when dropped
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Performance Considerations
//!
//! Safety wrappers minimize overhead:
//! - No buffering beyond what evdev provides
//! - Direct syscall wrappers (no extra layers)
//! - Efficient permission checks (once at open time)
//! - Target: < 10μs overhead per operation

// Module declarations (components will be implemented in subsequent tasks)
pub mod device;
pub mod permissions;
pub mod uinput;

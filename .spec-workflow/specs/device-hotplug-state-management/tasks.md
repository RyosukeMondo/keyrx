# Tasks Document

## Phase 1: Core Watcher

- [x] 1. Create DeviceWatcher trait
  - File: `core/src/discovery/watcher.rs`
  - Define device event interface
  - Add DeviceState enum
  - Purpose: Platform-agnostic interface
  - _Requirements: 1.1, 1.2_

- [x] 2. Implement Linux device watcher
  - File: `core/src/discovery/watcher_linux.rs`
  - Use inotify on /dev/input
  - Detect add/remove events
  - Purpose: Linux hotplug support
  - _Requirements: 1.1_

- [x] 3. Implement Windows device watcher
  - File: `core/src/discovery/watcher_windows.rs`
  - Handle WM_DEVICECHANGE
  - Detect HID changes
  - Purpose: Windows hotplug support
  - _Requirements: 1.1_

## Phase 2: State Management

- [ ] 4. Add session pause/resume
  - File: `core/src/engine/session.rs`
  - Pause on device removal
  - Resume on reconnection
  - Purpose: Graceful transitions
  - _Requirements: 2.1, 2.2, 2.3_

- [ ] 5. Add multi-device coordination
  - File: `core/src/engine/multi_device.rs`
  - Independent device handling
  - Partial failure support
  - Purpose: Multi-device support
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

# Requirements: UI/UX Unification & Rebuild

## Introduction

This initiative aims to unify the User Experience (UX) and Developer Experience (DX) of the KeyRx application. Currently, disparate logic handles device listing, configuration, and file paths across different UI pages (Dashboard, Devices, Wiring), leading to inconsistencies (e.g., offline devices missing). We will centralize state management and establish Rust as the Single Source of Truth (SSOT).

## Alignment with Product Vision

To provide a professional-grade tool, KeyRx must be consistent and reliable. Users should not be confused by "ghost" devices or conflicting settings. This rebuild enables a seamless flow between plugging in a device, configuring it, and monitoring it.

## Requirements

### Requirement 1: Unified Device & Context Management

**User Story:** As a user, I want a "Unified Workspace" where I select a device ONCE, and that selection persists as I move between Wiring, Mapping, and Testing.

#### Acceptance Criteria

1.  **Global Context:** A global `ActiveDeviceContext` MUST track the currently selected device. Changing it in the Dashboard changes it for Wiring/Mapping/Layouts.
2.  **Unified Dropdown:** The device selector MUST be available and identical in all relevant views (Wiring, Devices, Dashboard), showing both Connected (Green) and Offline (Grey) devices.
3.  **Five-Pillar Consistency:** The unification MUST cover all 5 core views:
    *   **Dashboard:** High-level status, quick actions, "Command Center".
    *   **Devices:** Hardware inventory, connection status, firmware details.
    *   **Layouts:** Virtual matrix management (Layers, Grid configuration).
    *   **Wiring:** Physical-to-Virtual mapping (The "Bridge" between Device and Layout).
    *   **Mapping:** Logical key assignment (The "Brain" of the keyboard).

### Requirement 2: Configuration Single Source of Truth (SSOT)

**User Story:** As a developer/user, I want the application to respect a single configuration file location managed by the Core, so that I don't encounter "split brain" issues where the UI reads one file and the Engine reads another.

#### Acceptance Criteria

1.  **Rust Authority:** The UI SHALL NOT read `~/.keyrx` (or any config file) directly from Dart/Flutter file I/O.
2.  **API Driven:** All configuration (paths, validation options, recent files) MUST be fetched via FFI calls to the Rust Core.
3.  **Path Consistency:** The application MUST use platform-standard paths (e.g., `%APPDATA%` on Windows, XDG on Linux) unless overridden by an implementation-specific environment variable, determined solely by Rust.

### Requirement 3: "Next Level" Visual & Navigational Flow

**User Story:** As a user, I want a modern, fluid interface ("Reborn UI") that guides me through the configuration process intuitively.

#### Acceptance Criteria

1.  **Seamless Transitions:** Navigation between Wiring and Mapping should feel like changing tabs in a single workspace, not loading separate pages.
2.  **Visual Unification:** Use a consistent Design System (Colors, Typography, Spacing, Cards) across all 5 views.
3.  **Contextual Actions:**
    *   From **Devices**: Click "Configure" -> Go to Wiring.
    *   From **Wiring**: Click "Map Keys" -> Go to Mapping.
    *   From **Dashboard**: Click "Troubleshoot" -> Go to Monitor/Devices.

## Non-Functional Requirements

### Code Architecture
-   **Service Abstraction:** UI components must depend on high-level Facade services, not low-level FFI bridges or raw streams.
-   **State Management:** Use a centralized Store (e.g., Provider/Riverpod) that holds the `ActiveDevice`, `ActiveProfile`, and `ActiveLayout` state, distinct from the UI widgets.

### Usability
-   **Visual Feedback:** Offline states should be visually distinct (e.g., desaturated/greyed out) but fully interactive for configuration purposes.
-   **"Alive" UI:** The UI should react immediately to device connection events without manual refresh.

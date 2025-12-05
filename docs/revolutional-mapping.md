# Revolutional Mapping Architecture

## 1. Executive Summary & Vision

This document outlines a paradigm shift in how KeyRx handles device mapping and configuration. The core philosophy moves away from simple key-to-key remapping on a generic surface towards a powerful, professional-grade system where **Physical Devices** are decoupled from **Logical Profiles**.

This architecture solves the scalability problem of managing multiple identical devices (e.g., two of the same macro pad) and enables complex, purpose-specific configurations that can be swapped instantly. It transforms KeyRx from a simple remapper into a comprehensive Input Management System.

## 2. Core Concepts

### 2.1. The Great Decoupling
Currently, configurations are often tightly bound to a generic "keyboard" concept. The new architecture introduces a strict separation:
- **Device:** A physical piece of hardware connected to the computer, identified uniquely.
- **Profile:** A logical set of behaviors, mappings, and layouts, independent of any specific hardware.

### 2.2. Device Identity
Every connected device is treated as a unique individual, not just a generic class.
- **Identification:** Devices are tracked via `Product ID` + `Vendor ID` + `Serial Number`.
- **User Aliasing:** Users can rename devices (e.g., "Work Macro Pad", "Streaming Deck") to give them semantic meaning.
- **State:** A device can be "Active" (Remapping Enabled) or "Passthrough" (No Remap).

### 2.3. Profile Definition
Profiles are the "soul" that can be inhabited by any compatible "body" (Device).
- **Independence:** Profiles exist whether a device is connected or not.
- **Layout Definition:** Profiles are defined by a Row-Column (Row-Col) matrix, allowing support for non-standard layouts (e.g., 5x5 Stream Deck, split keyboards, custom pads) alongside standard ANSI/ISO layouts.
- **One-to-One Mapping:** A single device can host exactly one active profile at a time.

## 3. User Experience (UX) Redesign

### 3.1. Navigation Structure
The hierarchy of the application reflects the workflow: Hardware -> Configuration.
- **Change:** The **Devices** icon moves **ABOVE** the **Editor** icon in the sidebar.
- **Reasoning:** Users first establish *what* is connected (Devices), then define *how* it works (Editor/Profiles).

### 3.2. Devices Tab (The Connection Hub)
The Devices tab becomes the command center for hardware management.
- **List View:** Displays all connected devices with their unique IDs or User Aliases.
- **Toggle Controls:**
    - **Remap Switch:** A toggle to Enable/Disable remapping for that specific device.
- **Profile Assignment:**
    - A dropdown menu for each device allows selecting which **Profile** is currently active on it.
    - Example: `Device: Stream Deck (Serial #123)` -> `Profile: Stream-Deck-OBS-Setup`.

### 3.3. Visual Editor (The Profile Workspace)
The Editor tab is reimagined. It no longer just swaps keys on a generic keyboard; it designs the behavior of a Profile.
- **Scope:** The editor edits a specific **Profile**, not a connected device directly.
- **Layout Visualization:**
    - The editor renders the specific Row-Col layout defined by the profile (e.g., a 5x5 grid for a macro pad, a full layout for a keyboard).
- **Mapping Workflow (Drag & Drop):**
    - **Source (Profile):** The physical layout representation (Row/Col grid).
    - **Target (Soft Keys):** A full virtual keyboard palette representing all available logical keycodes (the "Output").
    - **Interaction:** Users drag a key from the **Profile** (Physical Location) to the **Soft Keys** (Logical Output), or vice-versa, to establish the link: "When I press Row 0, Col 0, output Key 'A'".
    *Note: We will reuse and expand the existing drag-and-drop infrastructure to support this flexible mapping.*

## 4. Technical Architecture

### 4.1. Data Models

#### Device Registry
```rust
struct DeviceIdentity {
    vendor_id: u16,
    product_id: u16,
    serial_number: String,
    user_label: Option<String>,
}

struct DeviceState {
    identity: DeviceIdentity,
    is_remapping_enabled: bool,
    active_profile_id: Option<String>, // Link to Profile
}
```

#### Profile Registry
```rust
struct Profile {
    id: String,
    name: String,
    layout_type: LayoutType, // e.g., Matrix(rows, cols), Standard(ANSI)
    mappings: HashMap<(u8, u8), KeyAction>, // (Row, Col) -> Action
}
```

### 4.2. Mapping Logic
The runtime engine will perform a multi-stage lookup:
1. **Input Event:** Received from OS (contains Device Handle).
2. **Device Resolution:** Resolve Handle -> `DeviceIdentity`.
3. **State Check:** Is `is_remapping_enabled` true? If no, passthrough.
4. **Profile Resolution:** Get `active_profile_id`. Load `Profile`.
5. **Coordinate Translation:** Map Physical Input (ScanCode/Usage) -> logical (Row, Col).
6. **Action Resolution:** Look up (Row, Col) in Profile Mappings.
7. **Execution:** Execute the mapped `KeyAction`.

## 5. Roadmap
1. **Phase 1:** Implement Device Identity & Serial Number tracking in Rust Core.
2. **Phase 2:** Create Profile Data Structure and separate it from existing Device logic.
3. **Phase 3:** UI Overhaul - Devices Tab updates and Profile Assignment UI.
4. **Phase 4:** UI Overhaul - Visual Editor Row-Col support and Soft Key palette.

## 6. Technical Feasibility: Serial Number Acquisition

A critical component of this architecture is the ability to uniquely identify devices beyond their Vendor ID (VID) and Product ID (PID). This section outlines the strategy for Windows and Linux.

### 6.1. Windows Implementation
On Windows, we rely on the **Raw Input API**, which provides a "Device Name" that is actually a **PnP Device Interface Path**.

- **Format:** `\\?\HID#VID_vvvv&PID_pppp&MI_ii#<InstanceID>#{<ClassGUID>}`
- **Strategy:**
    1. Parse the `<InstanceID>` segment from the device path.
    2. **Scenario A (Device has Serial):** If the USB device has a valid iSerial string descriptor, Windows uses it as the Instance ID. This ID is persistent across ports and reboots.
    3. **Scenario B (No Serial):** If the device lacks a serial number (common in cheap keyboards), Windows generates a unique Instance ID based on the parent USB port topology (e.g., `7&3a2b4c5&0&0000`).
        - **Implication:** The profile binding becomes **port-dependent**. If the user moves the keyboard to a different USB port, it may be seen as a "new" device. This is an acceptable trade-off for generic hardware.

### 6.2. Linux Implementation
On Linux, we utilize the `evdev` interface and `udev` properties.

- **Primary Method (evdev):**
    - The `EVIOCGUNIQ` ioctl (exposed via `evdev` crate as `.unique_name()`) retrieves the device's unique identifier (serial number) if reported by the kernel driver.
- **Secondary Method (udev):**
    - If `evdev` yields no result, we can cross-reference the `/sys/class/input/eventX` path to read udev properties like `ID_SERIAL` or `ID_SERIAL_SHORT`.
- **Challenges:**
    - Similar to Windows, generic devices may return an empty string for the unique ID. In this case, we fall back to generating a synthetic ID based on the `phys` topology (physical port path), making it port-dependent.

### 6.3. Fallback Strategy
For devices where a true hardware serial number cannot be obtained on either OS:
1. **Generate a "Port-Bound ID":** Create a hash of the physical port location + VID + PID.
2. **UX Handling:** Warn the user that this specific device configuration is tied to the USB port.

## 7. Layout Definitions & Geometry

To support the "Profile defined by Row-Col" concept across diverse hardware (from standard keyboards to 5x5 macro pads), we need a translation layer that sits between the OS Input and the Logical Profile.

### 7.1. The Problem
A Profile defines *logic* for a grid (e.g., "Row 0, Col 0 = Mute").
A Device sends *signals* (e.g., `ScanCode 0x1E` or `HID Usage 0x05`).
We need a way to tell the system: "For this device, ScanCode 0x1E corresponds to Row 0, Col 0."

### 7.2. Device Definition Files (Templates)
We will introduce a library of **Device Definitions** (likely JSON or TOML) that describe the physical properties of supported hardware.

```toml
# example-streamdeck-mk2.toml
name = "Stream Deck MK.2"
vendor_id = 0x0fd9
product_id = 0x0080
layout_type = "Matrix"
rows = 3
cols = 5

# The Physical -> Logical Mapping
[matrix]
# (Row, Col) = UsageID / ScanCode
"0,0" = 0x01
"0,1" = 0x02
...
```

#### Why TOML? (vs. Rhai)
While we use Rhai for *Profiles* (Logic), we strictly use TOML for *Device Definitions* (Data).
1.  **Static vs. Dynamic:** Device properties (VID/PID, matrix size) are immutable facts. They don't need logic, loops, or functions.
2.  **Safety & Sharing:** A community repository of TOML files is safe to auto-download and parse. Executing external Rhai scripts just to read device dimensions introduces security risks.
3.  **Performance:** We can scan 1000 TOML headers instantly to find a matching device. running 1000 scripts to find a match is too heavy.

### 7.3. Runtime Resolution
1. **Device Connects:** System identifies `VID:PID`.
2. **Template Lookup:** Finds matching `Device Definition`.
3. **Input Normalization:**
   - **Standard Keyboard:** Uses a default "ANSI Standard" definition mapping Scan Codes to standard Row/Col positions.
   - **Custom Device:** Uses the specific definition file to translate proprietary Usage IDs into the standardized (Row, Col) coordinate system.
4. **Profile Execution:** The active Profile receives the normalized (Row, Col) and executes the mapped action.

This allows the **Visual Editor** to render the correct grid (3x5, Split, etc.) automatically based on the connected device's definition.



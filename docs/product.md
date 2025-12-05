# KeyRx Product Vision

**Version:** 2.0 - Revolutionary Mapping Architecture
**Last Updated:** 2025-12-06

---

## 1. Product Identity

**KeyRx is a Professional-Grade Input Management System.**

We are not a simple key remapper. We are a comprehensive platform that transforms how users interact with their input devices by decoupling physical hardware from logical behavior.

### Core Value Proposition

> "One Profile, Any Device. One Device, Any Profile."

KeyRx enables power users, developers, gamers, and professionals to:
- Manage multiple identical devices independently (e.g., two Stream Decks with different configurations)
- Swap device purposes instantly by changing profiles
- Create purpose-specific configurations that transcend hardware
- Maintain professional workflows across different physical setups

---

## 2. The Revolutionary Paradigm Shift

### From: Simple Remapping
**Old Model:** "Make key A output key B on my keyboard"
- Tightly coupled to specific hardware
- One configuration per device model
- No multi-device scenarios
- Port changes break configuration

### To: Input Management System
**New Model:** "Device X hosts Profile Y, which defines a complete behavior system"
- Physical devices are unique individuals
- Profiles are portable, reusable configurations
- Serial number-based device identity
- One device, one active profile at any time
- Per-device remap enable/disable

---

## 3. Target Users & Use Cases

### Power Users
**Scenario:** Own two identical macro pads - one for work, one for streaming.
- **Current Pain:** Can't distinguish them; both get same config
- **KeyRx Solution:** Label them "Work Pad" and "Stream Pad", assign different profiles

### Content Creators
**Scenario:** Stream Deck for OBS, macro pad for Premiere, custom keypad for Photoshop.
- **Current Pain:** Generic tools don't understand custom devices
- **KeyRx Solution:** Define device layouts (5×5 grid, split layouts), create visual mappings

### Developers
**Scenario:** Programmable keyboard with Vim layer, Emacs layer, IDE layer.
- **Current Pain:** Static configs, can't share layers across devices
- **KeyRx Solution:** Profile library - swap "Vim Power User" profile to any compatible keyboard

### Professionals
**Scenario:** CAD workstation with 3D mouse, macro pad, and custom keypad.
- **Current Pain:** Tools don't coordinate multi-device workflows
- **KeyRx Solution:** Per-device profiles that work together, instant enable/disable per device

### IT Administrators
**Scenario:** Manage standardized input setups across multiple workstations.
- **Current Pain:** USB port changes invalidate configs
- **KeyRx Solution:** Serial number tracking ensures profiles follow devices, not ports

---

## 4. Core Product Principles

### 4.1. Hardware First, Configuration Second
**Navigation Philosophy:** Users first establish **what** is connected (Devices), then define **how** it works (Profiles/Editor).

**UI Implication:** Devices tab **ABOVE** Editor tab in navigation hierarchy.

### 4.2. One-to-One Mapping Clarity
- One device can host exactly one active profile at a time
- One profile can be assigned to multiple devices (but each device gets one instance)
- No ambiguity, no "which config is active?" confusion

### 4.3. Device Individuality
Every device is a unique entity:
- Tracked by VID + PID + Serial Number
- User can assign semantic names ("Work Stream Deck", "Gaming Macro Pad")
- Per-device state: Active (remapping enabled) or Passthrough (disabled)

### 4.4. Profile Portability
Profiles exist independently:
- Stored separately from device registry
- Can be created, edited, deleted without a device connected
- Defined by row-column layout, compatible with any device matching that layout

### 4.5. Layout Awareness
- Supports non-standard layouts: 5×5 macro pads, split keyboards, custom grids
- Supports standard layouts: ANSI, ISO keyboards
- Visual editor renders actual device geometry
- Device Definitions (TOML) describe hardware capabilities

---

## 5. User Experience (UX) Flows

### 5.1. First-Time Setup Flow

**Step 1: Connect Device**
- User plugs in keyboard/macro pad
- KeyRx detects device via serial number
- If first time: Prompt "Would you like to label this device?"
- User enters friendly name: "Main Keyboard"

**Step 2: Assign or Create Profile**
- System shows available compatible profiles (matching layout)
- Options:
  - Select existing profile from library
  - Create new blank profile
  - Discover device layout (if custom hardware)

**Step 3: Enable Remapping**
- Device shows in Devices tab with toggle switch (default: OFF)
- User toggles ON to activate profile
- Engine starts processing input through profile

### 5.2. Multi-Device Management Flow

**Scenario:** User has two identical Stream Decks

**Current View (Devices Tab):**
```
┌─────────────────────────────────────────────────┐
│ Connected Devices                               │
├─────────────────────────────────────────────────┤
│ 🎛️ Work Stream Deck                             │
│    VID: 0fd9  PID: 0080  Serial: ABC123        │
│    Profile: [OBS Studio Controls ▼] [🔄 ON ]   │
├─────────────────────────────────────────────────┤
│ 🎛️ Streaming Desk                               │
│    VID: 0fd9  PID: 0080  Serial: XYZ789        │
│    Profile: [Twitch Bot Controls  ▼] [🔄 OFF]  │
└─────────────────────────────────────────────────┘
```

**User Actions:**
- Click dropdown → Change profile assignment
- Toggle switch → Enable/disable remapping per device
- Click device row → Edit device label or manage profiles

### 5.3. Profile Creation Flow (Visual Editor)

**Step 1: Select Profile to Edit**
- User navigates to Editor tab
- Dropdown at top: "Select Profile: [Create New] [Work Layout] [Gaming]"

**Step 2: Visual Mapping Interface**
- **Left Panel:** Device layout (row-col grid matching device geometry)
  - 5×5 grid for macro pad
  - ANSI 104-key layout for keyboard
  - Split layout for ergonomic keyboards
- **Right Panel:** Soft Keyboard Palette (all available output keycodes)
- **Interaction:** Drag from profile position to output key
  - "Row 0, Col 0 → Mute"
  - "Row 1, Col 2 → Ctrl+C"

**Step 3: Save and Assign**
- Save profile with name
- Return to Devices tab
- Assign profile to desired device

### 5.4. Profile Swapping Flow

**Scenario:** User switches from work to gaming mode

**Single Action:**
1. Navigate to Devices tab
2. Find "Main Keyboard" device
3. Click profile dropdown
4. Select "Gaming FPS Profile"
5. Done - instant behavior swap, no restart required

**Result:** Same physical keyboard now behaves completely differently.

---

## 6. Feature Differentiation (vs Competitors)

| Feature | KeyRx | Karabiner | AutoHotkey | QMK/VIA |
|---------|-------|-----------|------------|---------|
| **Multi-device support** | ✅ Per-device profiles | ❌ Global only | ❌ Generic hooks | ✅ Per-keyboard firmware |
| **Serial tracking** | ✅ Unique device identity | ❌ Model-based | ❌ None | ❌ Port-dependent |
| **Profile library** | ✅ Portable profiles | ⚠️ Complex JSON | ⚠️ Scripts only | ❌ Firmware-bound |
| **Visual editor** | ✅ Row-col aware | ⚠️ Basic | ❌ None | ✅ Web UI |
| **Custom layouts** | ✅ TOML definitions | ❌ Keyboard only | ❌ Generic | ✅ Firmware compile |
| **Per-device toggle** | ✅ Individual enable/disable | ❌ Global only | ❌ None | N/A |
| **Cross-platform** | ✅ Windows + Linux | ❌ macOS only | ❌ Windows only | ✅ Hardware |
| **Scripting** | ✅ Rhai (sandboxed) | ❌ JSON only | ✅ AHK script | ❌ C firmware |
| **Hot-swap profiles** | ✅ Runtime swap | ❌ Config reload | ⚠️ Script reload | ❌ Reflash firmware |

**KeyRx Unique Selling Points:**
1. Only solution with per-device instance profiles
2. Only software solution with full layout awareness (including custom grids)
3. Only cross-platform solution with visual editor
4. Only system supporting multiple identical devices independently

---

## 7. Product Roadmap

### Phase 1: Device Identity Foundation (Revolutionary Core)
**Goal:** Implement unique device tracking and profile decoupling

**Deliverables:**
- Serial number extraction (Windows PnP paths, Linux evdev EVIOCGUNIQ)
- Device Registry (runtime device state management)
- Profile Registry (persistent profile storage, decoupled from devices)
- Device-to-Profile binding system
- Per-device remap enable/disable state

**Success Metric:** User can connect two identical keyboards and assign different profiles.

### Phase 2: Navigation & Device Management UI
**Goal:** Rebuild UI to match "Hardware First" philosophy

**Deliverables:**
- Move Devices tab ABOVE Editor in navigation
- Device list with per-device toggle switches
- Profile assignment dropdown per device
- Device labeling/aliasing UI
- Active profile indicator per device

**Success Metric:** User can manage 5+ devices from Devices tab without confusion.

### Phase 3: Visual Editor Enhancement
**Goal:** Support dynamic layouts and row-col mapping

**Deliverables:**
- Device Definition loader (TOML-based)
- Dynamic layout rendering (5×5 grid, ANSI, split, etc.)
- Row-col grid visualization
- Soft keyboard palette for output mapping
- Drag-and-drop row-col to keycode mapping

**Success Metric:** User can visually map a 5×5 Stream Deck and a split keyboard using same editor.

### Phase 4: Device Definition Library
**Goal:** Community-driven hardware support

**Deliverables:**
- TOML device definition format specification
- Built-in definitions for common devices (Stream Deck, macro pads, standard keyboards)
- Auto-detection: VID:PID → Device Definition lookup
- Community contribution system (GitHub repo for definitions)

**Success Metric:** 20+ device definitions available, auto-detected 95% of common hardware.

### Phase 5: Profile Marketplace & Sharing
**Goal:** Enable community-driven profiles

**Deliverables:**
- Profile import/export (JSON format)
- Profile library UI (browse, search, install)
- Profile sharing platform (optional cloud sync)
- Profile compatibility checking (layout requirements)

**Success Metric:** 100+ community profiles available, 50% adoption rate.

---

## 8. Success Metrics (KPIs)

### Adoption Metrics
- **Active Users:** 10K in Year 1, 50K in Year 2
- **Device Profiles Created:** Avg 3 per user
- **Multi-Device Users:** 30% of user base manages 2+ devices

### Usage Metrics
- **Profile Swaps per Week:** Avg 5 per active user
- **Per-Device Toggle Usage:** 40% of users use toggle weekly
- **Custom Layout Support:** 20% of users define custom devices

### Quality Metrics
- **Input Latency:** <1ms overhead (99th percentile)
- **Crash Rate:** <0.1% of sessions
- **Profile Load Time:** <100ms for profiles <1000 mappings

### Community Metrics
- **Device Definitions Contributed:** 50+ in Year 1
- **Profile Library Size:** 500+ profiles in Year 2
- **User Satisfaction:** 4.5+ stars (5-point scale)

---

## 9. Non-Goals (Explicitly Out of Scope)

### What KeyRx Will NOT Do

**1. Mouse Automation**
- We are not AutoClicker or macro recorder
- Focus: Input remapping and logical behavior, not automation scripts

**2. Game-Specific Anti-Cheat Bypass**
- We respect anti-cheat systems
- Users responsible for compliance with game ToS

**3. Hardware Firmware Flashing**
- We are software-only
- We don't compete with QMK/VIA firmware solutions

**4. Cloud-Required Operation**
- All core features work offline
- Cloud sync is optional enhancement only

**5. GUI-Only Configuration**
- Power users can edit profiles as JSON/Rhai directly
- CLI tools remain first-class citizens

---

## 10. Design Philosophy

### 10.1. Progressive Complexity
- **Beginner:** Simple profile selection, basic remapping
- **Intermediate:** Visual editor, multi-device management
- **Expert:** Direct JSON/Rhai editing, custom device definitions

UI surfaces appropriate complexity at each level.

### 10.2. Fail-Safe Operation
- Emergency exit always available (Ctrl+Alt+Shift+Escape)
- Safe mode toggle (bypass all profiles, passthrough only)
- Per-device disable prevents total lockout

### 10.3. Transparency
- User always knows which profile is active
- Visual indicators show device state (active/passthrough/failed)
- Debugger shows real-time input processing

### 10.4. Performance as Feature
- Latency metrics visible to user
- Trade-off visualizer (features vs speed)
- No background telemetry or analytics (privacy-first)

---

## 11. Brand Positioning

**KeyRx is the "Pro Tool" for Input Management.**

- **Not:** A hobbyist project or toy remapper
- **Is:** A professional-grade system for power users

**Tone:** Technical, precise, respectful of user expertise.

**Messaging:**
- "Prescribe Your Input Reality"
- "One Profile, Any Device"
- "Beyond Remapping: Input Management"

**Visual Identity:**
- Clean, modern UI (Flutter Material Design)
- Technical aesthetics (monospace fonts for data, grid layouts)
- High contrast, accessibility-first

---

## 12. Competitive Moat

### Technical Moat
1. **Serial Number Tracking:** Unique implementation solving multi-device problem
2. **Layout-Aware Profiles:** TOML device definitions enable any hardware
3. **Rust Performance:** <1ms latency while maintaining safety

### Community Moat
1. **Device Definition Library:** Network effect - more devices = more value
2. **Profile Marketplace:** User-generated content creates lock-in
3. **Open Source:** Community contributions accelerate development

### UX Moat
1. **Navigation Philosophy:** Hardware-first approach reduces cognitive load
2. **Visual Editor:** Drag-and-drop simplicity for complex mappings
3. **Per-Device Control:** Granular management competitors lack

---

## 13. Future Vision (Beyond MVP)

### Advanced Features (Post-Phase 5)

**Context-Aware Profiles:**
- Auto-swap profiles based on active application
- Time-of-day profile switching
- Location-based profiles (work vs home)

**Multi-Device Coordination:**
- Cross-device macros (trigger on Device A, output on Device B)
- Device groups (enable/disable sets of devices together)
- Synchronized state across devices

**Analytics & Insights:**
- Heatmaps showing key usage patterns
- Efficiency metrics (keystrokes saved, shortcuts used)
- Profile optimization suggestions

**Advanced Scripting:**
- Lua support (optional, alongside Rhai)
- Plugin system for custom actions
- External API integration (webhook triggers)

**Enterprise Features:**
- Centralized profile management for teams
- Role-based device configurations
- Audit logging for compliance

---

## 14. Open Questions & Research Needed

### Device Identity Edge Cases
**Q:** How do we handle devices that change serial numbers?
**Research:** Survey common problematic hardware, document workarounds.

### Profile Compatibility
**Q:** What happens when user assigns 10×10 profile to 5×5 device?
**Decision:** Validation check on assignment, warn user, allow override with truncation.

### Port-Bound Devices
**Q:** Should we support "bind to port" mode for legacy hardware?
**Proposal:** Advanced setting, warn user about port dependency.

### Offline Profile Discovery
**Q:** Can we auto-generate layouts without user input?
**Research:** Investigate standard HID descriptor parsing for layout hints.

---

## 15. Success Definition

**KeyRx succeeds when:**

1. A user can walk up to any computer with KeyRx installed
2. Plug in their device (recognized by serial number)
3. Have their personal profile instantly active
4. Work with identical productivity regardless of location

**The North Star:** "Your device, your profile, anywhere."

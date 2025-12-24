# Windows Implementation Options

**Status**: RFC - Awaiting Decision
**Created**: 2024-12-24
**Purpose**: Architectural decisions for Windows platform support

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Windows Daemon Architecture](#windows-daemon-architecture)
3. [Keyboard Interception Methods](#keyboard-interception-methods)
4. [Control & UI Options](#control--ui-options)
5. [React Integration](#react-integration)
6. [Technology Stack Options](#technology-stack-options)
7. [Recommended Architecture](#recommended-architecture)
8. [Decision Matrix](#decision-matrix)
9. [Implementation Phases](#implementation-phases)

---

## Executive Summary

### Core Questions

1. **Installation**: Windows Service vs Desktop Application?
2. **Keyboard Hooks**: Low-level hooks vs Interception driver?
3. **Control Interface**: Web UI, Native GUI, Tray icon, or CLI only?
4. **React App**: Embedded web server vs Standalone Electron/Tauri?
5. **Distribution**: MSI installer, winget, Chocolatey, or portable EXE?

### Current Linux Architecture (for comparison)

```
keyrx_daemon (Linux)
├── Runs as: systemd service OR standalone CLI
├── Keyboard: evdev (input) + uinput (output)
├── Control: Embedded web server (axum) + React UI
├── Config: .krx binary file
└── Install: cargo install OR manual systemd setup
```

---

## 1. Windows Daemon Architecture

### Option A: Windows Service (Background Process)

**What it is**: System service that runs in the background, starts automatically with Windows.

**Pros**:
- ✅ Starts automatically on boot
- ✅ Runs without user login
- ✅ Consistent with Linux systemd approach
- ✅ Professional deployment model
- ✅ Can run with elevated privileges

**Cons**:
- ❌ Requires admin rights to install
- ❌ No GUI (services run in Session 0)
- ❌ Complex to debug
- ❌ Requires separate control app for configuration

**Installation**:
```powershell
# Via sc.exe (native)
sc.exe create KeyRxDaemon binPath= "C:\Program Files\KeyRx\keyrx_daemon.exe"
sc.exe start KeyRxDaemon

# Or via MSI installer with WiX Toolset
```

**Control Methods**:
- Web UI via embedded server (http://localhost:8080)
- Tray icon application (separate process)
- Command-line: `keyrx-cli.exe`

---

### Option B: Desktop Application (User Process)

**What it is**: Regular application that runs when user is logged in.

**Pros**:
- ✅ No admin rights required
- ✅ Can have GUI/tray icon
- ✅ Easier to debug
- ✅ Simpler installation (copy EXE)
- ✅ Direct user interaction

**Cons**:
- ❌ Only runs when user is logged in
- ❌ Must be added to startup manually/via installer
- ❌ Less "professional" for enterprise use
- ❌ May be killed by user accidentally

**Installation**:
```powershell
# Copy to Program Files
xcopy keyrx_daemon.exe "C:\Program Files\KeyRx\"

# Add to startup
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v KeyRx /t REG_SZ /d "C:\Program Files\KeyRx\keyrx_daemon.exe"
```

**Control Methods**:
- System tray icon with menu
- Web UI (embedded server)
- Native GUI window

---

### Option C: Hybrid Approach (Service + Tray App)

**What it is**: Windows Service for keyboard handling + Tray application for control.

**Architecture**:
```
┌─────────────────────────────────────┐
│  keyrx_service.exe (Windows Service)│
│  - Keyboard hooks                   │
│  - Event processing                 │
│  - IPC server (Named Pipes)         │
└─────────────────────────────────────┘
              ↕ IPC
┌─────────────────────────────────────┐
│  keyrx_tray.exe (User App)          │
│  - System tray icon                 │
│  - Configuration UI                 │
│  - Status display                   │
└─────────────────────────────────────┘
```

**Pros**:
- ✅ Best of both worlds
- ✅ Professional architecture
- ✅ Service runs at boot, tray provides UI
- ✅ Separation of concerns

**Cons**:
- ❌ Two separate binaries to maintain
- ❌ IPC complexity (Named Pipes, shared memory, etc.)
- ❌ More complex installation

---

## 2. Keyboard Interception Methods

### Option A: Low-Level Keyboard Hooks (SetWindowsHookEx)

**API**: `SetWindowsHookEx(WH_KEYBOARD_LL, ...)`

**How it works**:
- Hook into Windows message queue
- Receives all keyboard events before applications
- Can block/modify events

**Pros**:
- ✅ Built into Windows (no drivers)
- ✅ User-mode implementation (safer)
- ✅ No code signing required
- ✅ Works without admin rights (in some cases)

**Cons**:
- ❌ Can be bypassed by some applications
- ❌ Higher latency than driver-based approach
- ❌ May not intercept admin-level applications
- ❌ Anti-cheat software may block hooks

**Code Example**:
```rust
use windows_sys::Win32::UI::WindowsAndMessaging::*;

unsafe extern "system" fn keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code >= 0 {
        let kbd = *(lparam as *const KBDLLHOOKSTRUCT);
        // Process event, decide whether to block
        if should_block_event(kbd.vkCode) {
            return 1; // Block
        }
    }
    CallNextHookEx(0, code, wparam, lparam)
}
```

**Similar Projects Using This**:
- AutoHotkey
- PowerToys Keyboard Manager

---

### Option B: Kernel-Mode Interception Driver

**Method**: Install kernel driver (like Interception driver)

**How it works**:
- Kernel-mode filter driver
- Intercepts at lowest level
- Complete control over input

**Pros**:
- ✅ Cannot be bypassed
- ✅ Works with all applications
- ✅ Lowest latency
- ✅ Most reliable

**Cons**:
- ❌ Requires driver signing ($$$)
- ❌ Must be digitally signed for Windows 10/11
- ❌ Users must disable Driver Signature Enforcement (bad UX)
- ❌ Can cause system instability if buggy
- ❌ Antivirus may flag as malware

**Similar Projects**:
- Interception driver (used by QMK on Windows)
- Some commercial remapping tools

**Cost & Complexity**:
- Code signing certificate: ~$300-500/year
- Kernel development expertise required
- Testing on multiple Windows versions

---

### Option C: Windows Filtering Platform (WFP)

**Method**: Use Windows built-in filtering for HID devices

**How it works**:
- User-mode driver framework (UMDF)
- Filter HID input reports

**Pros**:
- ✅ Official Microsoft approach
- ✅ More stable than low-level hooks
- ✅ No kernel-mode driver needed

**Cons**:
- ❌ Still requires driver signing
- ❌ Complex setup
- ❌ Limited documentation

---

### ⭐ Recommendation: Low-Level Hooks (Option A)

**Rationale**:
- No cost (driver signing is expensive)
- Sufficient for 95% of use cases
- Easy to implement and test
- Can upgrade to driver later if needed

**Limitations to document**:
- May not work with some games (anti-cheat)
- May not work with admin applications (unless running as admin)

---

## 3. Control & UI Options

### Current Linux Approach

```rust
// Embedded web server (already implemented)
keyrx_daemon --features web
// Serves React UI at http://localhost:8080
```

### Option A: Embedded Web UI (Same as Linux)

**Architecture**:
```
keyrx_daemon.exe
├── Axum web server (http://localhost:8080)
├── React UI (compiled to static files, embedded)
└── WebSocket for live updates
```

**Access**:
- User opens browser: `http://localhost:8080`
- Or tray icon with "Open Control Panel" → opens browser

**Pros**:
- ✅ Code reuse from Linux implementation
- ✅ Cross-platform UI (same React code)
- ✅ Easy to develop (web technologies)
- ✅ No separate GUI framework needed

**Cons**:
- ❌ Requires browser
- ❌ Not "native" Windows feel
- ❌ Port conflicts possible

---

### Option B: Native Windows GUI (WPF/WinUI)

**Technology Options**:

1. **WPF (Windows Presentation Foundation)**
   - Mature, well-documented
   - XAML-based UI
   - Rust bindings: Limited

2. **WinUI 3**
   - Modern, Fluent Design
   - Future of Windows UI
   - Rust bindings: Minimal

**Pros**:
- ✅ Native Windows look and feel
- ✅ Better integration with Windows
- ✅ No browser required

**Cons**:
- ❌ Windows-only (no code reuse)
- ❌ Two separate UIs to maintain (web + native)
- ❌ Limited Rust support

---

### Option C: System Tray Icon Only

**What it is**: Minimal UI via right-click menu on tray icon

**Features**:
- Start/Stop daemon
- Reload configuration
- Open web UI
- Exit

**Pros**:
- ✅ Minimal resource usage
- ✅ Easy to implement
- ✅ Standard Windows UX

**Cons**:
- ❌ Limited functionality
- ❌ Still need web/native UI for configuration

---

### Option D: Tauri-based Control App (React + Rust)

**What it is**: Standalone desktop app using Tauri framework

**Architecture**:
```
keyrx_daemon.exe (Service/Background)
     ↕ IPC (Named Pipes / HTTP)
keyrx_control.exe (Tauri App)
├── React frontend
├── Rust backend (Tauri)
└── Native window
```

**Pros**:
- ✅ Reuse React UI code
- ✅ Native window (no browser)
- ✅ Small bundle size (~3MB)
- ✅ Rust-based (consistent tech stack)

**Cons**:
- ❌ Additional dependency (Tauri)
- ❌ Separate app to maintain
- ❌ IPC complexity

---

### Option E: Electron-based Control App (React + Node.js)

**Similar to Tauri but heavier**

**Pros**:
- ✅ Reuse React UI
- ✅ More mature ecosystem
- ✅ Better documentation

**Cons**:
- ❌ Large bundle size (~100MB)
- ❌ Node.js dependency (not Rust)
- ❌ Higher memory usage

---

### ⭐ Recommendation: Hybrid (Embedded Web + Tray Icon)

**Phase 1**: Embedded web server + system tray icon
**Phase 2**: Optional Tauri app for native feel

**Rationale**:
- Reuse existing Linux web UI code
- Tray icon for quick access
- Can add Tauri later if users demand it

---

## 4. React Integration

### Current Setup (Linux)

```rust
// keyrx_daemon/src/web/static_files.rs
// Embeds compiled React app into binary
include_dir!("../keyrx_ui/dist")
```

### Windows Integration Options

#### Option 1: Same as Linux (Embedded Web Server)

**How it works**:
1. Build React app: `npm run build` → `dist/`
2. Embed into Rust binary at compile time
3. Serve via Axum on `http://localhost:8080`

**Works on Windows**: ✅ Yes, exactly the same

**Code**:
```rust
// Same code works on Windows
use axum::Router;
use tower_http::services::ServeDir;

let app = Router::new()
    .nest_service("/", ServeDir::new("ui_dist"));
```

---

#### Option 2: Separate React App (Electron/Tauri)

**How it works**:
1. Build React app
2. Package with Tauri/Electron
3. Separate executable: `keyrx-control.exe`
4. Communicates with daemon via IPC/HTTP

**Tauri Example**:
```rust
// src-tauri/main.rs
#[tauri::command]
async fn reload_config() -> Result<String, String> {
    // Call keyrx_daemon via HTTP/IPC
    let client = reqwest::Client::new();
    client.post("http://localhost:8080/api/reload")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok("Config reloaded".to_string())
}
```

---

#### Option 3: Native Windows App (No React)

**Technology**: WinUI 3 / WPF (XAML)

**Pros**:
- Native Windows controls
- Better performance

**Cons**:
- Cannot reuse React code
- Windows-only

---

### ⭐ Recommendation: Embedded Web Server (Same as Linux)

**Rationale**:
- Zero additional code
- Works identically on Linux and Windows
- Can add Tauri wrapper later if needed

---

## 5. Technology Stack Options

### Comparison Matrix

| Component | Option 1 (Minimal) | Option 2 (Balanced) | Option 3 (Full-Featured) |
|-----------|-------------------|---------------------|--------------------------|
| **Daemon Type** | Desktop App | Windows Service | Service + Tray App |
| **Keyboard Hook** | Low-level hooks | Low-level hooks | Kernel driver |
| **UI Framework** | Web UI only | Web UI + Tray | Native GUI (Tauri) |
| **Installer** | Portable EXE | MSI installer | MSI + Auto-update |
| **Distribution** | Manual download | winget | winget + Chocolatey |
| **Complexity** | Low | Medium | High |
| **Dev Time** | 1-2 weeks | 3-4 weeks | 6-8 weeks |

---

### Recommended Stack (Balanced Approach)

```
┌─────────────────────────────────────────────────────┐
│  keyrx_daemon.exe (Windows Service OR Desktop App)  │
│  ├── Low-level keyboard hooks (SetWindowsHookEx)    │
│  ├── Event processing (keyrx_core)                  │
│  ├── Embedded web server (axum) - optional          │
│  └── System tray icon (windows-rs)                  │
└─────────────────────────────────────────────────────┘
         ↓ Serves
┌─────────────────────────────────────────────────────┐
│  React UI (http://localhost:8080)                   │
│  ├── Configuration editor                           │
│  ├── Live event viewer                              │
│  └── Status dashboard                               │
└─────────────────────────────────────────────────────┘
```

**Dependencies**:
```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_UI_WindowsAndMessaging",
    "Win32_System_Threading",
    "Win32_UI_Shell",  # For tray icon
]}
```

---

## 6. Decision Matrix

### Critical Decisions Needed

#### Decision 1: Service vs Desktop App

**Factors to consider**:
- Target users: Home users vs Enterprise?
- Admin rights availability?
- Auto-start requirement?

**Recommendation**: **Desktop App** for v0.2.0, upgrade to Service in v0.3.0

**Rationale**:
- Easier initial implementation
- Better debugging experience
- Can migrate to service later
- Most home users prefer desktop apps

---

#### Decision 2: Keyboard Hook Method

**Recommendation**: **Low-level hooks (SetWindowsHookEx)**

**Rationale**:
- No cost (driver signing is $300-500/year)
- Sufficient for 95% of users
- Easy to implement
- Document limitations clearly

---

#### Decision 3: UI Approach

**Recommendation**: **Embedded Web UI + System Tray**

**Rationale**:
- Reuse existing React UI (zero additional work)
- Tray icon for quick start/stop
- Can add Tauri wrapper later if users request it

---

#### Decision 4: Installer

**Recommendation**: **Portable EXE** for v0.2.0, **MSI** for v0.3.0

**Rationale**:
- Start simple (portable EXE)
- Add MSI installer once stable
- MSI enables winget distribution

---

## 7. Recommended Architecture

### Phase 1: Windows v0.2.0 (MVP)

**Goal**: Feature parity with Linux

```
keyrx_daemon.exe
├── Desktop application (runs in user session)
├── Low-level keyboard hooks
├── Embedded web server (optional --features web)
├── System tray icon
│   ├── Start/Stop
│   ├── Reload config
│   ├── Open web UI
│   └── Exit
└── Installation: Portable EXE + manual startup config
```

**Features**:
- ✅ Keyboard remapping (same as Linux)
- ✅ Web UI (same as Linux)
- ✅ System tray icon (Windows-specific)
- ✅ Configuration hot-reload

**Installation**:
1. Download `keyrx_daemon.exe`
2. Copy to `C:\Program Files\KeyRx\`
3. Create shortcut in Startup folder (optional)
4. Run: `keyrx_daemon.exe run --config my-config.krx`

---

### Phase 2: Windows v0.3.0 (Polished)

**Additions**:
- Windows Service option
- MSI installer (WiX Toolset)
- winget package
- Auto-update mechanism
- Better error messages

---

### Phase 3: Windows v0.4.0 (Advanced)

**Additions**:
- Tauri-based native control app (optional)
- Per-application configuration (window-based rules)
- Gaming mode (detect games, disable remapping)

---

## 8. Implementation Phases

### Phase 1: Core Keyboard Hooks (Week 1-2)

**Files to create/modify**:
- `keyrx_daemon/src/platform/windows.rs` - Main implementation
- `keyrx_daemon/src/platform/windows/hooks.rs` - Hook setup
- `keyrx_daemon/src/platform/windows/keycode.rs` - VK_* to KeyCode mapping

**Tasks**:
1. Implement `SetWindowsHookEx(WH_KEYBOARD_LL)`
2. Map Virtual Key Codes to KeyRx KeyCode enum
3. Inject remapped events (SendInput)
4. Test with simple remapping (A → B)

---

### Phase 2: System Tray Icon (Week 2)

**Files**:
- `keyrx_daemon/src/windows/tray.rs`

**Tasks**:
1. Create tray icon
2. Right-click menu
3. Start/stop daemon
4. Open web UI in browser

---

### Phase 3: Testing & Polish (Week 3)

**Tasks**:
1. Test on Windows 10/11
2. Test with different keyboards
3. Handle edge cases (UAC prompts, admin apps)
4. Documentation
5. CI/CD for Windows builds

---

## 9. Questions for You

Please decide on the following:

### Q1: Service or Desktop App?
- [ ] **A**: Desktop Application (easier, for home users)
- [ ] **B**: Windows Service (professional, for enterprise)
- [ ] **C**: Both (hybrid approach)

### Q2: Keyboard Hook Method?
- [ ] **A**: Low-level hooks (free, 95% coverage)
- [ ] **B**: Kernel driver (expensive, 100% coverage)

### Q3: UI Approach?
- [ ] **A**: Embedded web UI only (reuse existing)
- [ ] **B**: Web UI + System tray icon (recommended)
- [ ] **C**: Native Tauri app (more work, native feel)

### Q4: Installer Type?
- [ ] **A**: Portable EXE (simple)
- [ ] **B**: MSI installer (professional)
- [ ] **C**: Both (portable + installer)

### Q5: Target Timeline?
- [ ] **A**: Rush (1-2 weeks, minimal features)
- [ ] **B**: Balanced (3-4 weeks, tray icon + web UI)
- [ ] **C**: Polished (6-8 weeks, service + installer + native app)

---

## 10. Next Steps

Once you make the decisions above, I will:

1. ✅ Implement Windows keyboard hooks
2. ✅ Add system tray icon
3. ✅ Test on Windows 10/11
4. ✅ Update documentation
5. ✅ Create installer (if selected)
6. ✅ Publish v0.2.0 with Windows support

---

## Appendix: Similar Projects on Windows

| Project | Architecture | Hook Method | UI |
|---------|-------------|-------------|-----|
| AutoHotkey | Desktop app | Low-level hooks | Script-based |
| PowerToys | Service + Tray | Low-level hooks | Native WinUI |
| SharpKeys | Desktop app | Registry (no hooks) | WPF |
| Kanata | Desktop app | Interception driver | CLI only |
| QMK (Via) | Desktop app | Interception driver | Electron |

**Observation**: Most use low-level hooks or Interception driver. Embedded web UI is rare but works well.

---

**Document Status**: Awaiting decisions to proceed with implementation.

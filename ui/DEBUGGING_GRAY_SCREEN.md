# Debugging Gray Screen Issue

## What Was Added

### 1. Comprehensive Debug Logging
Added `print()` statements to track initialization:
- `[DEBUG]` - Normal execution flow
- `[ERROR]` - Errors with stack traces

### 2. Error Boundary Widget
If app fails to initialize, you'll see:
- Red error icon
- Error message
- "Check browser console for details"

### 3. Source Maps
Profile build includes source maps for better error messages

## How to Debug

### Step 1: Open Browser Console

**Chrome/Edge:**
1. Press `F12` or `Ctrl+Shift+I` (Windows/Linux)
2. Click "Console" tab

**Firefox:**
1. Press `F12` or `Ctrl+Shift+K`
2. Click "Console" tab

### Step 2: Access the App

1. Navigate to: https://192.168.11.24:4443
2. Accept certificate warning
3. Watch console output

### Step 3: Look for Debug Messages

**Expected console output (successful initialization):**
```
[DEBUG] main: Starting KeyRx app
[DEBUG] main: Flutter binding initialized
[DEBUG] main: Running app (kIsWeb=true)
[DEBUG] KeyrxApp.build: starting
[DEBUG] KeyrxApp.build: watching providers...
[DEBUG] bridgeProvider: initializing (kIsWeb=true)
[DEBUG] bridgeProvider: returning null for web platform
[DEBUG] serviceRegistryProvider: initializing
[DEBUG] serviceRegistryProvider: bridge obtained (isNull=true)
[DEBUG] serviceRegistryProvider: backendUrl=http://192.168.11.24:8888
[DEBUG] serviceRegistryProvider: initialized successfully
[DEBUG] KeyrxApp.build: registry obtained
[DEBUG] keyrxFacadeProvider: initializing
[DEBUG] keyrxFacadeProvider: registry obtained
[DEBUG] keyrxFacadeProvider: initialized successfully
[DEBUG] KeyrxApp.build: facade obtained
[DEBUG] apiDocsServiceProvider: initializing
[DEBUG] apiDocsServiceProvider: initialized successfully
[DEBUG] KeyrxApp.build: apiDocsService obtained
[DEBUG] appStateProvider: initializing
[DEBUG] appStateProvider: registry obtained
[DEBUG] appStateProvider: initialized successfully
[DEBUG] KeyrxApp.build: appState obtained
[DEBUG] KeyrxApp.build: creating MultiProvider
Backend URL configured: http://192.168.11.24:8888
```

**If you see `[ERROR]` messages:**
```
[ERROR] serviceRegistryProvider: failed to initialize: <error details>
[ERROR] Stack trace: <stack trace>
```

This tells us EXACTLY where initialization is failing.

### Step 4: Check for JavaScript Errors

Look for red error messages in console:
- `TypeError`
- `ReferenceError`
- `Uncaught exception`
- Any stack traces

### Step 5: Report Back

Copy and send me:
1. All `[DEBUG]` messages
2. All `[ERROR]` messages
3. Any JavaScript errors
4. Screenshot of what you see (gray screen or error screen)

## Common Issues and What to Look For

### Issue 1: Gray Screen with No Errors
**Look for:**
- Are debug messages appearing?
- Does it stop at a specific provider?
- Any warnings about missing dependencies?

### Issue 2: Error Screen Showing
**Look for:**
- What's the error message?
- Which provider failed?
- Stack trace details

### Issue 3: App Hangs During Load
**Look for:**
- Which was the last `[DEBUG]` message?
- Any pending network requests (Network tab)?
- Any infinite loops or timeouts?

### Issue 4: White/Blank Screen
**Look for:**
- Did Flutter initialize? (`[DEBUG] main:` messages)
- Did providers start initializing?
- Any console errors before initialization?

## Additional Debug Tools

### Check Network Requests

1. Open DevTools → Network tab
2. Reload page
3. Look for:
   - Failed requests (red)
   - Backend API calls to :8888
   - CORS errors

### Check Service Worker

1. DevTools → Application tab
2. Click "Service Workers"
3. Check status and errors

### Check Local Storage

1. DevTools → Application tab
2. Click "Local Storage"
3. Check for cached state

### Enable Verbose Logging

In browser console, run:
```javascript
localStorage.debug = '*';
```
Then reload page for more detailed logs.

## Quick Fixes to Try

### 1. Clear Browser Cache
```
Ctrl+Shift+Delete → Clear cached images and files
```

### 2. Hard Reload
```
Ctrl+Shift+R (or Cmd+Shift+R on Mac)
```

### 3. Disable Service Worker
```
DevTools → Application → Service Workers → Unregister
```

### 4. Check Backend is Running
```bash
curl http://192.168.11.24:8888/api/devices
```
Should return JSON with devices.

## Server Status

Check what's running:
```bash
# In terminal on server
ss -tlnp | grep -E "4443|8888"
```

Should show:
- Port 4443: HTTPS web server
- Port 8888: Backend server

## Debugging Tips

1. **Start simple:** Look at the FIRST error, not the last
2. **Check sequence:** Debug messages should appear in order
3. **Network matters:** Check DevTools Network tab for failed requests
4. **Backend first:** Verify backend is responding before debugging frontend
5. **Source maps:** Stack traces will show actual TypeScript/Dart file locations

## Need More Help?

If debug messages don't reveal the issue, we can add more logging to specific services:
- Service registry initialization
- WebApiClient requests
- Provider lifecycle
- Widget rendering

Just let me know what you see in the console!

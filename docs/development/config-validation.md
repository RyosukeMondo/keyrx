# Configuration Validation Guide

## Introduction

KeyRX provides **real-time configuration validation** to help you catch errors early in your keyboard remapping configurations. The validation system offers an IDE-like experience with instant feedback as you type, making it easier to write correct configurations without trial-and-error testing.

### Key Benefits

- **Instant Feedback**: Errors appear as you type (with 500ms debounce)
- **Syntax Validation**: Powered by the same WASM engine used at runtime
- **Code Quality Hints**: Detects unused layers, naming inconsistencies, and more
- **Quick Fixes**: One-click solutions for common mistakes
- **No Manual Testing**: Catch errors before deploying to your keyboard

## Getting Started

### Opening the Config Editor

1. Launch the KeyRX web interface (default: http://localhost:8080)
2. Click **"Config Editor"** in the navigation menu
3. The Monaco editor will load with syntax highlighting for Rhai

### Basic Workflow

```
┌──────────────────────────────────────────────┐
│ 1. Type your configuration in the editor     │
│    (Rhai syntax highlighting active)         │
└──────────────┬───────────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────────┐
│ 2. Validation runs automatically after 500ms │
│    (no action required)                      │
└──────────────┬───────────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────────┐
│ 3. Errors appear as red squiggly lines       │
│    Warnings appear as orange squiggly lines  │
└──────────────┬───────────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────────┐
│ 4. Hover over errors to see details          │
│    Click "Quick Fix" if available            │
└──────────────┬───────────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────────┐
│ 5. Save button enables when all errors fixed │
│    Click "Apply Configuration" to save       │
└──────────────────────────────────────────────┘
```

## Understanding Validation Results

### Validation Status Panel

The status panel shows a summary of your configuration's health:

```
┌─────────────────────────────────────────┐
│ Validation Status                    ▼ │
├─────────────────────────────────────────┤
│ ❌ 2 Errors                             │
│ ⚠️  1 Warning                            │
│ 💡 1 Hint                               │
│                                         │
│ Errors:                                 │
│ • Line 12: Unknown key code 'KEY_INVALD'│
│   [Jump] [Quick Fix]                    │
│ • Line 23: Missing closing brace        │
│   [Jump]                                │
└─────────────────────────────────────────┘
```

**Status Indicators:**

- **❌ Red Badge**: Critical errors that prevent saving
- **⚠️ Orange Badge**: Warnings that don't block saving but indicate potential issues
- **💡 Blue Badge**: Hints for code quality improvements
- **✓ Green Checkmark**: Configuration is valid and ready to save

### Error Squiggles

Errors are highlighted directly in the editor:

```rhai
layer "base" {
    map KEY_A to KEY_B;
    map KEY_INVALD to KEY_C;  // Red squiggly line under KEY_INVALD
    //      ^^^^^^
    //      Unknown key code
}
```

**Color Coding:**
- **Red squiggles**: Syntax errors, undefined references, type mismatches
- **Orange squiggles**: Warnings (unused layers, deprecated syntax)
- **Blue squiggles**: Hints (naming conventions, code style suggestions)

## Common Errors and Solutions

### 1. Syntax Errors

#### Missing Semicolon

**Error Message:**
```
Parse error at line 4, column 23: Expected ';' after statement
```

**Example:**
```rhai
layer "base" {
    map KEY_A to KEY_B  // ❌ Missing semicolon
    map KEY_C to KEY_D;
}
```

**Solution:**
```rhai
layer "base" {
    map KEY_A to KEY_B;  // ✅ Added semicolon
    map KEY_C to KEY_D;
}
```

#### Unclosed Brace

**Error Message:**
```
Parse error at line 12, column 1: Expected '}' to close block
```

**Example:**
```rhai
layer "base" {
    map KEY_A to KEY_B;
    // ❌ Missing closing brace
```

**Solution:**
```rhai
layer "base" {
    map KEY_A to KEY_B;
}  // ✅ Added closing brace
```

### 2. Invalid Key Codes

#### Typo in Key Name

**Error Message:**
```
Unknown key code 'KEY_INVALD' at line 5, column 14
```

**Example:**
```rhai
layer "base" {
    map KEY_INVALD to KEY_B;  // ❌ Typo: INVALD instead of INVALID
}
```

**Quick Fix Available:**
```
Did you mean 'KEY_INVALID'?
[Apply Quick Fix]
```

**Solution:** Click "Apply Quick Fix" or manually correct to `KEY_INVALID`

#### Non-Existent Key Code

**Error Message:**
```
Unknown key code 'KEY_MAGIC' at line 8, column 9
```

**Example:**
```rhai
layer "base" {
    map KEY_MAGIC to KEY_B;  // ❌ KEY_MAGIC doesn't exist
}
```

**Solution:** Replace with a valid key code from the [key code reference](key-codes.md)

### 3. Undefined References

#### Undefined Layer

**Error Message:**
```
Undefined layer 'gaming' at line 15, column 21
```

**Example:**
```rhai
layer "base" {
    on KEY_F1 activate_layer "gaming";  // ❌ Layer 'gaming' not defined
}
```

**Solution:** Define the layer before referencing it:
```rhai
layer "gaming" {
    // Gaming layer configuration
}

layer "base" {
    on KEY_F1 activate_layer "gaming";  // ✅ Now 'gaming' exists
}
```

#### Undefined Modifier

**Error Message:**
```
Undefined modifier 'super_mod' at line 10, column 8
```

**Example:**
```rhai
layer "base" {
    if modifier_active("super_mod") {  // ❌ Modifier not defined
        map KEY_A to KEY_B;
    }
}
```

**Solution:** Define the modifier first:
```rhai
modifier "super_mod" {
    keys: [KEY_LEFTCTRL, KEY_LEFTSHIFT];
}

layer "base" {
    if modifier_active("super_mod") {  // ✅ Now 'super_mod' exists
        map KEY_A to KEY_B;
    }
}
```

### 4. Type Mismatches

#### Wrong Argument Type

**Error Message:**
```
Type error at line 18, column 24: Expected string, got number
```

**Example:**
```rhai
layer "base" {
    on KEY_A activate_layer 123;  // ❌ activate_layer expects string
}
```

**Solution:**
```rhai
layer "base" {
    on KEY_A activate_layer "layer_123";  // ✅ String argument
}
```

## Linting Rules

KeyRX includes code quality linting rules to help maintain clean configurations. Linting rules generate **warnings** or **hints** that don't block saving but indicate potential issues.

### Unused Layer Detection

**Purpose:** Identifies layers that are defined but never activated.

**Warning Message:**
```
Layer 'debug' is defined but never activated (line 45)
```

**Example:**
```rhai
layer "debug" {  // ⚠️ This layer is never activated
    map KEY_A to KEY_B;
}

layer "base" {
    // No activation of "debug" layer anywhere
    map KEY_C to KEY_D;
}
```

**Why It Matters:**
- Dead code that adds complexity
- May indicate forgotten functionality
- Increases configuration size unnecessarily

**Resolution Options:**
1. Add activation trigger: `on KEY_F12 activate_layer "debug";`
2. Remove the unused layer if not needed
3. Ignore if layer is activated dynamically at runtime

### Naming Consistency

**Purpose:** Detects mixed naming conventions (camelCase vs snake_case).

**Hint Message:**
```
Consider using consistent naming (e.g., all snake_case).
Found: camelCase (3) and snake_case (5)
```

**Example:**
```rhai
layer "baseLayer" {     // camelCase
    map KEY_A to KEY_B;
}

layer "gaming_layer" {  // snake_case
    map KEY_C to KEY_D;
}

layer "debugMode" {     // camelCase
    map KEY_E to KEY_F;
}
```

**Recommendation:** Choose one style and apply consistently:

**Option 1: snake_case (Recommended)**
```rhai
layer "base_layer" {
    map KEY_A to KEY_B;
}

layer "gaming_layer" {
    map KEY_C to KEY_D;
}

layer "debug_mode" {
    map KEY_E to KEY_F;
}
```

**Option 2: camelCase**
```rhai
layer "baseLayer" {
    map KEY_A to KEY_B;
}

layer "gamingLayer" {
    map KEY_C to KEY_D;
}

layer "debugMode" {
    map KEY_E to KEY_F;
}
```

### Large Configuration Warning

**Purpose:** Warns when configuration exceeds 500 lines.

**Warning Message:**
```
Configuration is large (823 lines). Consider splitting into multiple files or simplifying logic.
```

**Why It Matters:**
- Large configs are harder to maintain
- Performance may degrade
- Difficult to troubleshoot

**Solutions:**
1. Split into logical modules (base layer, gaming layer, etc.)
2. Remove duplicate or redundant mappings
3. Use functions to reduce repetition

## Using Quick Fixes

Quick Fixes provide one-click solutions for common errors.

### How to Access Quick Fixes

1. **Hover** over the red/orange squiggle
2. Wait for the tooltip to appear
3. Look for the **"Quick Fix"** button
4. Click to apply the suggested fix automatically

### Example: Typo Correction

**Before Quick Fix:**
```rhai
layer "base" {
    map KEY_INVALD to KEY_B;  // ❌ Typo detected
}
```

**Quick Fix Suggestion:**
```
Did you mean 'KEY_INVALID'?
[Apply Quick Fix]
```

**After Quick Fix:**
```rhai
layer "base" {
    map KEY_INVALID to KEY_B;  // ✅ Automatically corrected
}
```

### Example: Missing Semicolon

**Before Quick Fix:**
```rhai
layer "base" {
    map KEY_A to KEY_B  // ❌ Missing semicolon
}
```

**Quick Fix Suggestion:**
```
Add semicolon after statement
[Apply Quick Fix]
```

**After Quick Fix:**
```rhai
layer "base" {
    map KEY_A to KEY_B;  // ✅ Semicolon added
}
```

### Available Quick Fixes

| Error Type | Quick Fix Action |
|------------|------------------|
| Typo in key code | Suggest closest matching key code |
| Missing semicolon | Insert semicolon at correct position |
| Unclosed string | Add closing quote |
| Wrong quote type | Convert single to double quotes (or vice versa) |
| Undefined layer | Offer to create layer definition |

## Keyboard Shortcuts

Speed up your workflow with keyboard shortcuts:

| Shortcut | Action |
|----------|--------|
| **F8** | Jump to next error |
| **Shift+F8** | Jump to previous error |
| **Ctrl+S** / **Cmd+S** | Save configuration (if no errors) |
| **Ctrl+Space** | Trigger autocomplete |
| **Ctrl+/** / **Cmd+/** | Toggle line comment |
| **Alt+Shift+F** | Format document |

## Troubleshooting

### "Validation Unavailable" Error

**Symptom:**
```
⚠️ Validation is unavailable. Using basic syntax checking only.
```

**Causes:**
1. WASM module failed to load
2. Browser doesn't support WebAssembly
3. Network error prevented WASM download

**Solutions:**

1. **Check Browser Compatibility:**
   - Chrome 57+, Firefox 52+, Safari 11+, Edge 79+
   - WebAssembly must be enabled (check `about:flags`)

2. **Clear Browser Cache:**
   ```
   Chrome: Ctrl+Shift+Delete → Clear cached images and files
   Firefox: Ctrl+Shift+Delete → Cookies and Site Data
   ```

3. **Check Console for Errors:**
   - Open DevTools (F12)
   - Look for errors mentioning "WASM" or "WebAssembly"
   - Report errors to KeyRX maintainers

4. **Verify WASM File Exists:**
   - Check that `keyrx_core_bg.wasm` is present in `/static/wasm/`
   - File should be ~50KB in size

### Validation is Slow or Laggy

**Symptom:** Editor freezes or responds slowly when typing.

**Causes:**
1. Configuration is very large (>1000 lines)
2. Complex nested logic with many conditionals
3. Insufficient browser memory

**Solutions:**

1. **Split Large Configs:**
   - Break into smaller modules
   - Use multiple layer files
   - Reduce complexity

2. **Increase Debounce Delay:**
   - Default: 500ms
   - Adjust in settings to 1000ms or higher

3. **Close Other Tabs:**
   - Free up browser memory
   - Disable extensions that may interfere

4. **Use Lighter Browser:**
   - Try Firefox or Edge if Chrome is slow
   - Disable browser animations

### Errors Don't Clear After Fixing

**Symptom:** Red squiggles remain after correcting the error.

**Causes:**
1. Validation hasn't re-run yet (debounce delay)
2. Related errors exist elsewhere
3. Editor state not synchronized

**Solutions:**

1. **Wait for Debounce:**
   - Stop typing for 500ms
   - Validation will run automatically

2. **Force Re-Validation:**
   - Press **Ctrl+S** (doesn't save if errors exist, but triggers validation)
   - Or type any character and delete it

3. **Check Related Errors:**
   - One error may depend on fixing another
   - Use **F8** to cycle through all errors

4. **Refresh Editor:**
   - Save work to external file first
   - Reload page
   - Paste configuration back

### Save Button Stays Disabled

**Symptom:** "Apply Configuration" button remains disabled even after fixing all visible errors.

**Causes:**
1. Hidden errors in collapsed code sections
2. Validation still running (check "Validating..." indicator)
3. WASM validation failed silently

**Solutions:**

1. **Check Validation Status:**
   - Look for "Validating..." spinner
   - Wait for validation to complete

2. **Expand All Code:**
   - Check for errors in collapsed regions
   - Use **Ctrl+K, Ctrl+0** to unfold all

3. **Review Validation Panel:**
   - Open the validation status panel
   - Verify error count is actually zero
   - Check for warnings that might be miscounted

4. **Check Console:**
   - Open DevTools (F12)
   - Look for validation errors logged to console

## Advanced Tips

### Using Validation with Version Control

**Best Practice:** Always validate before committing configuration changes.

```bash
# Save validated config
git add configs/keyboard.rhai

# Commit with validation status
git commit -m "feat: add gaming layer (✓ validated)"

# Push to remote
git push origin main
```

### Testing Configurations Locally

Even with validation, it's good practice to test configurations:

1. **Validate** in editor (catches syntax errors)
2. **Test** in simulator (catches logic errors)
3. **Deploy** to daemon (verifies runtime behavior)

### Validation API for CI/CD

You can validate configurations in automated pipelines:

```bash
# Validate config via CLI (requires keyrx_compiler)
keyrx_compiler validate configs/keyboard.rhai

# Check exit code
if [ $? -eq 0 ]; then
    echo "✓ Configuration is valid"
else
    echo "✗ Validation failed"
    exit 1
fi
```

### Custom Validation Rules

Future versions will support custom linting rules via plugins. Stay tuned!

## FAQ

### Q: Can I disable validation?

**A:** Validation always runs, but you can ignore warnings/hints and save with them present. Errors must be fixed before saving.

### Q: Does validation work offline?

**A:** Yes, once the WASM module is loaded, validation works fully offline.

### Q: How accurate is validation compared to runtime?

**A:** Validation uses the exact same WASM code as the runtime daemon, so it's 100% accurate for syntax and semantic errors. Logic errors (e.g., incorrect key mappings) require testing in the simulator.

### Q: Can I validate multiple files at once?

**A:** Not currently. Each configuration file must be validated individually. Use the "Test Configuration" button in the simulator to test merged configs.

### Q: Why doesn't Quick Fix work for all errors?

**A:** Quick Fix is only available for errors with unambiguous solutions (typos, missing punctuation). Complex logic errors require manual fixes.

### Q: How do I report false positive warnings?

**A:** Open an issue on the KeyRX GitHub repository with:
- The configuration snippet
- The warning message
- Why you believe it's a false positive

## Resources

- [Rhai Language Reference](https://rhai.rs/book/)
- [Key Code Reference](key-codes.md)
- [Configuration Examples](examples/)
- [Troubleshooting Guide](troubleshooting.md)
- [GitHub Issues](https://github.com/your-org/keyrx/issues)

## Changelog

### v1.0.0 (2025-01-15)

- ✅ Real-time syntax validation with WASM
- ✅ Monaco editor integration with Rhai syntax highlighting
- ✅ Quick Fix suggestions for common errors
- ✅ Unused layer detection linting rule
- ✅ Naming consistency hints
- ✅ Keyboard shortcuts (F8 error navigation)
- ✅ WCAG 2.1 AA accessibility compliance

---

**Need help?** Join the KeyRX community on Discord or open a GitHub issue.

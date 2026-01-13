# UAT UI Fixes - Requirements

## Overview
User Acceptance Testing identified several UI/UX issues that need to be addressed before production readiness.

## Requirements

### R1: Dashboard Device Type Indicator
**Priority:** High
- R1.1: Display clear visual indicator distinguishing virtual keyboard (keyrx uinput) from physical hardware keyboards
- R1.2: Virtual device shows software/computer icon with "Virtual" label
- R1.3: Physical devices show keyboard hardware icon with "Hardware" label

### R2: Device Enable/Disable Toggle
**Priority:** Critical
- R2.1: Replace "Delete/Forget" functionality with enable/disable toggle
- R2.2: Disabled devices appear grayed out (opacity reduction)
- R2.3: Enabled state persists across page refresh
- R2.4: Disabled devices do not appear on Config page device selector
- R2.5: Remove confusing "cannot undo" delete confirmation

### R3: Profile Page Improvements
**Priority:** High
- R3.1: Name and description editable inline (click to edit, save on blur)
- R3.2: Remove separate Edit button
- R3.3: Clear visual indicator for currently active profile (one at a time)
- R3.4: Toast notification on successful profile activation ("Profile '{name}' activated!")
- R3.5: Activation is exclusive (mutex) - only one profile active

### R4: Navigation Consistency
**Priority:** Medium
- R4.1: When clicking file path to navigate from Profiles to Config page, sidebar menu must update to highlight "Config"
- R4.2: Selected profile should be passed to Config page

### R5: RPC Error Fix
**Priority:** Critical
- R5.1: Fix "Invalid client RPC message" error when saving configuration
- R5.2: Error indicates payload format mismatch - "content" field expected object
- R5.3: Add test coverage to prevent regression

### R6: Layers Display
**Priority:** High
- R6.1: Show all 256 layers (Base + MD_00 through MD_FF) in vertical scrollable layout
- R6.2: Currently only shows 6 layers - expand to full range
- R6.3: Use hex display format for modifier layers

### R7: Key Dropdown Population
**Priority:** Critical
- R7.1: Key selection dropdown currently shows empty - fix to show all available keys
- R7.2: Include all categories: Letters, Numbers, Function keys, Modifiers, Navigation, Numpad, Special
- R7.3: Organize keys by category for easy browsing
- R7.4: Works for both global and device-specific key assignment

## Acceptance Criteria

- [ ] Virtual/physical device indicator visible on Dashboard
- [ ] Device toggle replaces delete, persists state
- [ ] Profile inline editing works, active indicator clear
- [ ] Navigation highlights correct menu item
- [ ] No RPC errors when saving config
- [ ] All 256 layers visible in scrollable list
- [ ] Key dropdown populated with all key codes
- [ ] All new functionality has test coverage

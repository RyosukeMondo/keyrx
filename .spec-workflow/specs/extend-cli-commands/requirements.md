# Requirements: Extend CLI Commands

## Overview

Add CLI commands to manage runtime, hardware, layout, and keymap operations using shared services from core_logic.

## Context

**Problem**: CLI can only manage virtual devices. Cannot manage runtime slots, hardware profiles, layouts, or keymaps.

**Success Criteria**:
- CLI supports runtime/hardware/layout/keymap commands
- JSON protocol maintained
- All commands tested

## User Stories

### REQ-1: Runtime slot management
**As a** CLI user, **I want** runtime slot commands **So that** I can configure profiles without UI

**Commands**: `listRuntime`, `addSlot`, `removeSlot`, `activateSlot`

### REQ-2: Hardware profile management
**As a** CLI user, **I want** hardware profile commands **So that** I can configure device wiring without UI

**Commands**: `listHardware`, `saveHardware`, `deleteHardware`

### REQ-3: Layout management
**As a** CLI user, **I want** layout commands **So that** I can define keyboard layouts without UI

**Commands**: `listLayouts`, `saveLayout`, `deleteLayout`

### REQ-4: Keymap management
**As a** CLI user, **I want** keymap commands **So that** I can configure mappings without UI

**Commands**: `listKeymaps`, `saveKeymap`, `deleteKeymap`

## Dependencies

- move-config-services-to-core completed
- refactor-flutter-services completed

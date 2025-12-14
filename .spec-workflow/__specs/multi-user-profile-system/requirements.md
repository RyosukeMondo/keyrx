# Requirements Document

## Introduction

Currently only one configuration is active at a time. There's no way to quickly switch between profiles for different contexts (gaming, coding, writing). Users must manually swap config files, and there's no persistence of profile preferences across sessions.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Productivity**: Context-appropriate configurations
- **User Experience**: Seamless profile switching
- **Flexibility**: Multiple use cases supported

## Requirements

### Requirement 1: Profile Management

**User Story:** As a user, I want multiple profiles, so that I can switch contexts quickly.

#### Acceptance Criteria

1. WHEN profile created THEN it SHALL be stored persistently
2. IF profile switched THEN config SHALL change immediately
3. WHEN profile deleted THEN it SHALL be removed completely
4. IF profile renamed THEN references SHALL be updated

### Requirement 2: Profile Switching

**User Story:** As a user, I want easy switching, so that I don't lose productivity.

#### Acceptance Criteria

1. WHEN CLI switch command used THEN profile SHALL activate
2. IF hotkey configured THEN profile SHALL switch on press
3. WHEN UI selector used THEN profile SHALL change
4. IF application rules set THEN auto-switch SHALL occur

### Requirement 3: Profile Features

**User Story:** As a user, I want profile features, so that I can organize my configs.

#### Acceptance Criteria

1. WHEN profile inherited THEN base settings SHALL apply
2. IF profile exported THEN it SHALL be shareable
3. WHEN profile imported THEN validation SHALL occur
4. IF profile has icon THEN it SHALL display in UI

## Non-Functional Requirements

### Performance
- Profile switch SHALL complete in < 100ms
- Profile storage SHALL use < 1MB per profile
- Auto-switch detection SHALL add < 5ms latency

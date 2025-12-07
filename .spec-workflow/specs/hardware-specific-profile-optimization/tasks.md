# Tasks Document

_Status: Priority #8 in 2025 implementation order; all items pending. Layer on detector/calibration after profile system lands._

## Phase 1: Hardware Detection

- [x] 1. Create HardwareDetector
  - File: `core/src/hardware/detector.rs`
  - Vendor/product ID extraction
  - Device class inference
  - Purpose: Hardware identification
  - _Requirements: 1.1, 1.3_

- [x] 2. Implement device classification
  - File: `core/src/hardware/classification.rs`
  - Mechanical vs membrane detection
  - Heuristic-based classification
  - Purpose: Device categorization
  - _Requirements: 1.1_

## Phase 2: Profile Database

- [x] 3. Create HardwareProfile struct
  - File: `core/src/hardware/profile.rs`
  - Timing configuration
  - Profile metadata
  - Purpose: Profile data model
  - _Requirements: 2.1_

- [x] 4. Build profile database
  - File: `core/src/hardware/database.rs`
  - Builtin profiles
  - Lookup by VID/PID
  - Purpose: Profile storage
  - _Requirements: 1.2, 2.1, 2.4_

- [x] 5. Add cloud profile sync
  - File: `core/src/hardware/cloud_sync.rs`
  - Community profile download
  - Update checking
  - Purpose: Profile updates
  - _Requirements: 2.2, 2.3_

## Phase 3: Calibration

- [x] 6. Create Calibrator
  - File: `core/src/hardware/calibrator.rs`
  - Test sequence runner
  - Latency measurement
  - Purpose: Hardware calibration
  - _Requirements: 3.1, 3.2_

- [ ] 7. Implement timing optimizer
  - File: `core/src/hardware/optimizer.rs`
  - Optimal timing calculation
  - Confidence scoring
  - Purpose: Profile generation
  - _Requirements: 3.2, 3.3_

## Phase 4: Integration

- [ ] 8. Add CLI commands
  - File: `core/src/cli/commands/hardware.rs`
  - detect, calibrate, profile
  - Purpose: CLI access
  - _Requirements: 1.1, 3.1_

- [ ] 9. Create calibration UI
  - File: `ui/lib/pages/calibration_page.dart`
  - Interactive calibration
  - Before/after comparison
  - Purpose: User interface
  - _Requirements: 3.1, 3.4_

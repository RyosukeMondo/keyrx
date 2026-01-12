# Tasks Document

- [x] 1. Create ANSI 87 (TKL) layout JSON
  - File: keyrx_ui/src/data/layouts/ANSI_87.json
  - Create tenkeyless ANSI layout with 87 keys (full-size minus numpad)
  - Follow existing ANSI_104.json structure and key positioning conventions
  - Purpose: Provide TKL layout for users without numpad
  - _Leverage: keyrx_ui/src/data/layouts/ANSI_104.json_
  - _Requirements: 2.1_
  - _Prompt: Implement the task for spec keyboard-layouts, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend Developer specializing in keyboard layout data structures | Task: Create ANSI_87.json containing 87 keys for tenkeyless layout, copying main section from ANSI_104.json but excluding numpad keys (KC_NLCK, KC_PSLS, KC_PAST, KC_PMNS, KC_P7-KC_P0, KC_PPLS, KC_PENT, KC_PDOT) | Restrictions: Do not modify ANSI_104.json, maintain exact coordinate system and key code conventions, ensure 87 keys total | Success: JSON file validates, contains exactly 87 keys, integrates with KeyboardVisualizer without errors. After implementation, mark task [ ] as [-] in tasks.md, then use log-implementation tool to record details, then mark [-] as [x]._

- [x] 2. Create ISO 105 layout JSON
  - File: keyrx_ui/src/data/layouts/ISO_105.json
  - Create full-size ISO layout with 105 keys including ISO-specific Enter shape and extra key
  - ISO Enter is L-shaped (spans rows 2.5-3.5), extra key between left Shift and Z
  - Purpose: Provide European standard keyboard layout
  - _Leverage: keyrx_ui/src/data/layouts/ANSI_104.json_
  - _Requirements: 1.2_
  - _Prompt: Implement the task for spec keyboard-layouts, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend Developer with knowledge of ISO keyboard standards | Task: Create ISO_105.json with ISO-specific modifications: L-shaped Enter key using multi-row positioning, KC_NUBS (non-US backslash) key between left Shift and Z, KC_NUHS (non-US hash) key replacing ANSI backslash position | Restrictions: Maintain compatibility with parseKLEJson, use standard QMK keycodes, ensure 105 keys total | Success: JSON validates, ISO Enter renders correctly, extra key visible, 105 keys total. After implementation, mark task [ ] as [-] in tasks.md, then use log-implementation tool to record details, then mark [-] as [x]._

- [x] 3. Create ISO 88 (TKL) layout JSON
  - File: keyrx_ui/src/data/layouts/ISO_88.json
  - Create tenkeyless ISO layout with 88 keys
  - Combines ISO-specific keys with TKL form factor
  - Purpose: Provide European TKL keyboard layout
  - _Leverage: keyrx_ui/src/data/layouts/ISO_105.json, keyrx_ui/src/data/layouts/ANSI_87.json_
  - _Requirements: 2.2_
  - _Prompt: Implement the task for spec keyboard-layouts, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend Developer with knowledge of keyboard layouts | Task: Create ISO_88.json by combining ISO_105 main section with TKL form factor (no numpad), maintaining ISO Enter and KC_NUBS key | Restrictions: Do not duplicate code from other layouts, ensure consistent coordinate system, 88 keys total | Success: JSON validates, renders ISO TKL layout correctly, 88 keys total. After implementation, mark task [ ] as [-] in tasks.md, then use log-implementation tool to record details, then mark [-] as [x]._

- [x] 4. Create JIS 109 layout JSON
  - File: keyrx_ui/src/data/layouts/JIS_109.json
  - Create Japanese Industrial Standard keyboard layout with 109 keys
  - Include Japanese-specific keys: Henkan, Muhenkan, Katakana/Hiragana, Yen, Ro
  - Purpose: Provide Japanese standard keyboard layout
  - _Leverage: keyrx_ui/src/data/layouts/ANSI_104.json_
  - _Requirements: 1.3_
  - _Prompt: Implement the task for spec keyboard-layouts, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend Developer with knowledge of JIS keyboard standards | Task: Create JIS_109.json with Japanese-specific keys: KC_JYEN (Yen key), KC_RO (Ro key), KC_HENK (Henkan), KC_MHEN (Muhenkan), KC_KANA (Katakana/Hiragana), smaller spacebar with surrounding keys, JIS Enter shape | Restrictions: Use standard QMK JIS keycodes, maintain coordinate system compatibility, 109 keys total | Success: JSON validates, Japanese keys positioned correctly, 109 keys total. After implementation, mark task [ ] as [-] in tasks.md, then use log-implementation tool to record details, then mark [-] as [x]._

- [x] 5. Create 60% compact layout JSON
  - File: keyrx_ui/src/data/layouts/COMPACT_60.json
  - Create standard 60% keyboard layout with ~61 keys
  - No F-row, no nav cluster, no arrows, no numpad
  - Purpose: Provide compact 60% keyboard layout for enthusiasts
  - _Leverage: keyrx_ui/src/data/layouts/ANSI_104.json_
  - _Requirements: 3.1_
  - _Prompt: Implement the task for spec keyboard-layouts, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend Developer specializing in compact keyboard layouts | Task: Create COMPACT_60.json with only alphanumeric rows plus modifiers, no F-row (y=0 row), no nav/arrow cluster, right Shift extends to 2.75u | Restrictions: Maintain standard 60% proportions, use consistent key sizing, approximately 61 keys | Success: JSON validates, renders compact layout without gaps, ~61 keys. After implementation, mark task [ ] as [-] in tasks.md, then use log-implementation tool to record details, then mark [-] as [x]._

- [x] 6. Create 65% compact layout JSON
  - File: keyrx_ui/src/data/layouts/COMPACT_65.json
  - Create 65% keyboard layout with ~68 keys
  - Includes arrow keys and minimal navigation column
  - Purpose: Provide popular 65% layout with arrows
  - _Leverage: keyrx_ui/src/data/layouts/COMPACT_60.json_
  - _Requirements: 3.2_
  - _Prompt: Implement the task for spec keyboard-layouts, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend Developer specializing in compact keyboard layouts | Task: Create COMPACT_65.json extending 60% with right-side column containing Delete/PgUp/PgDn/End and dedicated arrow keys cluster | Restrictions: Maintain compact proportions, no F-row, arrow cluster aligned with bottom row, approximately 68 keys | Success: JSON validates, arrows and nav visible, ~68 keys. After implementation, mark task [ ] as [-] in tasks.md, then use log-implementation tool to record details, then mark [-] as [x]._

- [x] 7. Create 75% compact layout JSON
  - File: keyrx_ui/src/data/layouts/COMPACT_75.json
  - Create 75% keyboard layout with ~84 keys
  - Includes F-row and compressed navigation
  - Purpose: Provide compact layout with function keys
  - _Leverage: keyrx_ui/src/data/layouts/COMPACT_65.json_
  - _Requirements: 3.3_
  - _Prompt: Implement the task for spec keyboard-layouts, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend Developer specializing in compact keyboard layouts | Task: Create COMPACT_75.json adding compressed F-row to 65% base, F-keys in tight row with minimal gaps, Delete/Home/End in rightmost column | Restrictions: No numpad, maintain compact proportions, approximately 84 keys | Success: JSON validates, F-row visible, compact navigation, ~84 keys. After implementation, mark task [ ] as [-] in tasks.md, then use log-implementation tool to record details, then mark [-] as [x]._

- [x] 8. Create 96% compact layout JSON
  - File: keyrx_ui/src/data/layouts/COMPACT_96.json
  - Create 96% keyboard layout with ~96 keys
  - Full functionality in compact form factor (no gaps between sections)
  - Purpose: Provide compact full-size alternative
  - _Leverage: keyrx_ui/src/data/layouts/ANSI_104.json_
  - _Requirements: 4.2_
  - _Prompt: Implement the task for spec keyboard-layouts, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend Developer specializing in compact keyboard layouts | Task: Create COMPACT_96.json with all sections from full-size but no gaps between F-row and alphanumeric, no gap before nav cluster, compressed numpad adjacent to main section | Restrictions: Maintain all key functions from full-size, approximately 96 keys, no spacing gaps | Success: JSON validates, no visual gaps, numpad present, ~96 keys. After implementation, mark task [ ] as [-] in tasks.md, then use log-implementation tool to record details, then mark [-] as [x]._

- [ ] 9. Create HHKB layout JSON
  - File: keyrx_ui/src/data/layouts/HHKB.json
  - Create Happy Hacking Keyboard layout with 60 keys
  - Split backspace, Fn replacing Ctrl, Unix-optimized layout
  - Purpose: Provide HHKB-specific layout for enthusiasts
  - _Leverage: keyrx_ui/src/data/layouts/COMPACT_60.json_
  - _Requirements: 4.1_
  - _Prompt: Implement the task for spec keyboard-layouts, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend Developer with knowledge of HHKB layout | Task: Create HHKB.json with HHKB-specific features: split backspace (Backspace + backslash), Control in Caps Lock position, Fn key replacing right Ctrl, blank modifiers in bottom corners | Restrictions: Maintain HHKB proportions, 60 keys total, use standard QMK keycodes | Success: JSON validates, HHKB-specific layout renders correctly, 60 keys. After implementation, mark task [ ] as [-] in tasks.md, then use log-implementation tool to record details, then mark [-] as [x]._

- [ ] 10. Create Numpad layout JSON
  - File: keyrx_ui/src/data/layouts/NUMPAD.json
  - Create standalone numpad layout with 17 keys
  - Standard numpad with Num Lock, operators, and Enter
  - Purpose: Provide standalone numpad visualization
  - _Leverage: keyrx_ui/src/data/layouts/ANSI_104.json_
  - _Requirements: 4.3_
  - _Prompt: Implement the task for spec keyboard-layouts, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend Developer | Task: Create NUMPAD.json with standard 17-key numpad: Num Lock, divide, multiply, minus in top row, 7-8-9 plus, 4-5-6, 1-2-3 Enter, 0 (2u) and dot | Restrictions: Standalone layout, 17 keys exactly, standard numpad proportions | Success: JSON validates, renders standalone numpad, 17 keys. After implementation, mark task [ ] as [-] in tasks.md, then use log-implementation tool to record details, then mark [-] as [x]._

- [ ] 11. Update KeyboardVisualizer layout type union
  - File: keyrx_ui/src/components/KeyboardVisualizer.tsx
  - Expand layout type to include all new layouts
  - Update layoutData object with imports for all layouts
  - Purpose: Enable KeyboardVisualizer to render all new layouts
  - _Leverage: keyrx_ui/src/components/KeyboardVisualizer.tsx_
  - _Requirements: 1, 2, 3, 4_
  - _Prompt: Implement the task for spec keyboard-layouts, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React TypeScript Developer | Task: Update KeyboardVisualizerProps layout type union to include ANSI_87, ISO_105, ISO_88, JIS_109, COMPACT_60, COMPACT_65, COMPACT_75, COMPACT_96, HHKB, NUMPAD. Import all new layout JSON files and add to layoutData object. Remove placeholder comments. | Restrictions: Do not change rendering logic, maintain backward compatibility with ANSI_104, preserve existing props interface | Success: TypeScript compiles without errors, all layouts render correctly, no placeholders remain. After implementation, mark task [ ] as [-] in tasks.md, then use log-implementation tool to record details, then mark [-] as [x]._

- [ ] 12. Add layout validation tests
  - File: keyrx_ui/src/data/layouts/__tests__/layouts.test.ts
  - Create tests validating each layout JSON against schema
  - Verify key counts match expected values for each layout
  - Purpose: Ensure layout data integrity
  - _Leverage: keyrx_ui/src/data/layouts/*.json_
  - _Requirements: All_
  - _Prompt: Implement the task for spec keyboard-layouts, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with TypeScript testing expertise | Task: Create Jest tests that import all layout JSON files, validate each has required name and keys fields, verify key counts match expectations (ANSI_104=104, ANSI_87=87, etc.), check each key has code, label, x, y fields | Restrictions: Use existing test utilities, do not modify layout files, test all 11 layouts | Success: All tests pass, each layout validated for structure and key count. After implementation, mark task [ ] as [-] in tasks.md, then use log-implementation tool to record details, then mark [-] as [x]._

- [ ] 13. Add KeyboardVisualizer layout integration tests
  - File: keyrx_ui/src/components/__tests__/KeyboardVisualizer.layout.test.tsx
  - Create tests verifying each layout renders correctly
  - Test layout switching and key count verification
  - Purpose: Ensure component handles all layouts correctly
  - _Leverage: keyrx_ui/tests/testUtils.tsx, keyrx_ui/src/components/KeyboardVisualizer.tsx_
  - _Requirements: All_
  - _Prompt: Implement the task for spec keyboard-layouts, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Testing Engineer | Task: Create integration tests for KeyboardVisualizer testing each layout type renders without errors, produces correct number of key elements, handles layout prop changes | Restrictions: Use renderWithProviders from testUtils, test all 11 layouts, do not test internal implementation details | Success: All tests pass, each layout renders correct key count, no console errors. After implementation, mark task [ ] as [-] in tasks.md, then use log-implementation tool to record details, then mark [-] as [x]._

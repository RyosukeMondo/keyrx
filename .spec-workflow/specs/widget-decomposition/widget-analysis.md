# Widget Analysis: editor_widgets.dart

## File Overview
- **File**: `ui/lib/pages/editor_widgets.dart`
- **Total Lines**: 1036
- **Purpose**: Reusable widgets and utilities for the visual keymap editor

## Dependencies
```dart
import 'package:flutter/material.dart';
import '../ffi/bridge.dart';
import '../models/validation.dart' as validation_models;
import '../widgets/layer_panel.dart';
```

## Data Models (56 lines)

### 1. KeyActionType (Enum, Lines 13-27)
- **Type**: Enum with extension
- **Values**: `remap`, `block`, `pass`
- **Purpose**: Define types of key actions
- **Extension**: `KeyActionTypeLabel` - provides human-readable labels
- **Dependencies**: None
- **Target**: `ui/lib/models/key_action_type.dart`

### 2. KeyMapping (Class, Lines 29-46)
- **Type**: Data class
- **Fields**: `from`, `type`, `to`, `layer`, `tapHoldTap`, `tapHoldHold`
- **Purpose**: Represents a single key mapping configuration
- **Dependencies**: `KeyActionType`
- **Target**: `ui/lib/models/key_mapping.dart`

### 3. ComboMapping (Class, Lines 48-54)
- **Type**: Data class
- **Fields**: `keys` (List<String>), `output` (String)
- **Purpose**: Represents combo (multiple keys) configuration
- **Dependencies**: None
- **Target**: `ui/lib/models/combo_mapping.dart`

## Utilities (88 lines)

### 4. ScriptGenerator (Class, Lines 56-111)
- **Type**: Static utility class
- **Methods**: `build()`
- **Purpose**: Build Rhai scripts from mappings and combos
- **Dependencies**: `KeyMapping`, `ComboMapping`, `KeyActionType`
- **Lines**: 56
- **Target**: `ui/lib/utils/script_generator.dart`

### 5. KeyMappings (Class, Lines 113-143)
- **Type**: Static utility class
- **Fields**: `allowedKeys` (List<String>)
- **Methods**: `isKnownKey()`, `updateAllowedKeys()`
- **Purpose**: Key validation and registry management
- **Dependencies**: None
- **Lines**: 31
- **Target**: `ui/lib/utils/key_mappings.dart`

## Common Widgets (253 lines)

### 6. KeyValidityChip (Lines 379-397)
- **Type**: StatelessWidget
- **Purpose**: Display key validity status with colored chip
- **Props**: `label` (String), `isValid` (bool)
- **Dependencies**: Material
- **Lines**: 19
- **Target**: `ui/lib/widgets/common/key_validity_chip.dart`
- **Reusability**: HIGH - Can be used anywhere validity status is shown

### 7. _SuggestionChip (Lines 720-745)
- **Type**: StatelessWidget (private)
- **Purpose**: Display key suggestions in validation
- **Props**: `suggestion` (String)
- **Dependencies**: Material
- **Lines**: 26
- **Target**: `ui/lib/widgets/common/suggestion_chip.dart`
- **Reusability**: MEDIUM - Specific to validation UX

### 8. _CategoryBadge (Lines 1005-1035)
- **Type**: StatelessWidget (private)
- **Purpose**: Display warning category badge
- **Props**: `category` (WarningCategory)
- **Dependencies**: validation_models
- **Lines**: 31
- **Target**: `ui/lib/widgets/common/category_badge.dart`
- **Reusability**: MEDIUM - Used in validation context

## Editor-Specific Widgets (679 lines)

### 9. KeyConfigPanel (Lines 145-221)
- **Type**: StatelessWidget
- **Purpose**: Display and configure a single key mapping
- **Props**:
  - `selectedKey` (String?)
  - `selectedAction` (KeyActionType)
  - `outputController`, `layerController`, `tapOutputController`, `holdOutputController` (TextEditingController)
  - `onActionChanged` (ValueChanged<KeyActionType?>)
  - `onApply` (VoidCallback)
- **Dependencies**: `KeyActionType`, `KeyMappings`
- **Lines**: 77
- **Target**: `ui/lib/widgets/editor/key_config_panel.dart`
- **Sub-widgets**: Uses DropdownButton, TextField, FilledButton
- **Complexity**: MEDIUM - Form with validation

### 10. ComboConfigRow (Lines 223-268)
- **Type**: StatelessWidget
- **Purpose**: Display combo configuration form
- **Props**:
  - `comboKeysController`, `comboOutputController` (TextEditingController)
  - `combos` (List<ComboMapping>)
  - `onAddCombo` (VoidCallback)
  - `onRemoveCombo` (ValueChanged<int>)
- **Dependencies**: `ComboMapping`
- **Lines**: 46
- **Target**: `ui/lib/widgets/editor/combo_config_row.dart`
- **Sub-widgets**: Card, TextField, FilledButton, ListTile
- **Complexity**: LOW-MEDIUM

### 11. MappingListPanel (Lines 270-329)
- **Type**: StatelessWidget
- **Purpose**: Display list of key mappings with layer panel
- **Props**:
  - `mappings` (Map<String, KeyMapping>)
  - `layers` (List<LayerInfo>)
  - `onRemoveMapping` (ValueChanged<String>)
  - `onAddLayer` (VoidCallback)
  - `onToggleLayer` (Function(String, bool))
- **Dependencies**: `KeyMapping`, `LayerPanel`, `KeyValidityChip`, `KeyMappings`
- **Lines**: 60
- **Target**: `ui/lib/widgets/editor/mapping_list_panel.dart`
- **Sub-widgets**: ListView, ListTile, Wrap
- **Complexity**: MEDIUM - Complex layout with nested widgets

### 12. KeyRegistryBanner (Lines 331-377)
- **Type**: StatelessWidget
- **Purpose**: Display key registry status
- **Props**:
  - `isFetchingKeys` (bool)
  - `usingFallbackKeys` (bool)
  - `canonicalKeysCount` (int)
  - `registryError` (String?)
  - `onRefresh` (VoidCallback)
- **Dependencies**: Material
- **Lines**: 47
- **Target**: `ui/lib/widgets/editor/key_registry_banner.dart`
- **Sub-widgets**: Card, Icon, CircularProgressIndicator
- **Complexity**: LOW

### 13. ValidationBanner (Lines 399-470)
- **Type**: StatelessWidget
- **Purpose**: Display script validation status (basic version)
- **Props**:
  - `isValidating` (bool)
  - `validationResult` (ScriptValidationResult?)
  - `onShowErrors` (Function(List<ScriptValidationError>))
- **Dependencies**: bridge.dart (ScriptValidationResult, ScriptValidationError)
- **Lines**: 72
- **Target**: `ui/lib/widgets/editor/validation_banner.dart`
- **Sub-widgets**: Container, Material, InkWell
- **Complexity**: MEDIUM

### 14. showValidationErrorsDialog (Lines 472-500)
- **Type**: Function (Dialog Helper)
- **Purpose**: Show dialog with validation errors
- **Parameters**: `BuildContext`, `List<ScriptValidationError>`
- **Dependencies**: bridge.dart
- **Lines**: 29
- **Target**: `ui/lib/widgets/editor/validation_dialogs.dart`
- **Complexity**: LOW

### 15. ValidationBannerRich (Lines 502-718)
- **Type**: StatelessWidget
- **Purpose**: Rich validation banner with errors, warnings, suggestions
- **Props**:
  - `isValidating` (bool)
  - `validationResult` (validation_models.ValidationResult?)
- **Dependencies**: validation_models
- **Lines**: 217
- **Target**: `ui/lib/widgets/editor/validation_banner_rich.dart`
- **Sub-widgets**: Material, InkWell, Complex nested layout
- **Complexity**: HIGH - Many nested widgets and logic

### 16. _ValidationDetailsDialog (Lines 747-846)
- **Type**: StatelessWidget (private)
- **Purpose**: Full validation details with tabs
- **Props**: `result` (validation_models.ValidationResult)
- **Dependencies**: validation_models, `_ValidationErrorTile`, `_ValidationWarningTile`
- **Lines**: 100
- **Target**: Inline in `ui/lib/widgets/editor/validation_banner_rich.dart`
- **Sub-widgets**: AlertDialog, TabBar, TabBarView, ListView
- **Complexity**: HIGH

### 17. _ValidationErrorTile (Lines 848-936)
- **Type**: StatelessWidget (private)
- **Purpose**: Display validation error with suggestions
- **Props**: `error` (validation_models.ValidationError)
- **Dependencies**: validation_models, `_SuggestionChip`
- **Lines**: 89
- **Target**: Inline in `ui/lib/widgets/editor/validation_banner_rich.dart`
- **Complexity**: MEDIUM

### 18. _ValidationWarningTile (Lines 938-1003)
- **Type**: StatelessWidget (private)
- **Purpose**: Display validation warning
- **Props**: `warning` (validation_models.ValidationWarning)
- **Dependencies**: validation_models, `_CategoryBadge`
- **Lines**: 66
- **Target**: Inline in `ui/lib/widgets/editor/validation_banner_rich.dart`
- **Complexity**: MEDIUM

## Extraction Plan

### Phase 1: Models Extraction
These should be extracted first as they have no widget dependencies:
1. `KeyActionType` → `ui/lib/models/key_action_type.dart`
2. `KeyMapping` → `ui/lib/models/key_mapping.dart`
3. `ComboMapping` → `ui/lib/models/combo_mapping.dart`

### Phase 2: Utilities Extraction
These depend on models but not on widgets:
4. `ScriptGenerator` → `ui/lib/utils/script_generator.dart`
5. `KeyMappings` → `ui/lib/utils/key_mappings.dart`

### Phase 3: Common Widgets Extraction
Simple, reusable widgets:
6. `KeyValidityChip` → `ui/lib/widgets/common/key_validity_chip.dart`
7. `_SuggestionChip` → `ui/lib/widgets/common/suggestion_chip.dart` (make public)
8. `_CategoryBadge` → `ui/lib/widgets/common/category_badge.dart` (make public)

### Phase 4: Editor Widgets Extraction - Simple
Less complex editor widgets:
9. `KeyRegistryBanner` → `ui/lib/widgets/editor/key_registry_banner.dart`
10. `ComboConfigRow` → `ui/lib/widgets/editor/combo_config_row.dart`
11. `KeyConfigPanel` → `ui/lib/widgets/editor/key_config_panel.dart`

### Phase 5: Editor Widgets Extraction - Complex
More complex widgets with many dependencies:
12. `MappingListPanel` → `ui/lib/widgets/editor/mapping_list_panel.dart`
13. `ValidationBanner` → `ui/lib/widgets/editor/validation_banner.dart`
14. `showValidationErrorsDialog` → `ui/lib/widgets/editor/validation_dialogs.dart`
15. `ValidationBannerRich` + private widgets → `ui/lib/widgets/editor/validation_banner_rich.dart`
    - Includes: `_ValidationDetailsDialog`, `_ValidationErrorTile`, `_ValidationWarningTile`

### Phase 6: Barrel Exports
Create index files for each directory:
- `ui/lib/models/models.dart` - Export all models
- `ui/lib/utils/utils.dart` - Export all utils
- `ui/lib/widgets/common/common.dart` - Export all common widgets
- `ui/lib/widgets/editor/editor.dart` - Export all editor widgets
- `ui/lib/widgets/widgets.dart` - Export all widget categories

## Dependency Graph

```
Models (no dependencies):
  - KeyActionType
  - ComboMapping
  - KeyMapping (depends on KeyActionType)

Utils (depend on models):
  - KeyMappings
  - ScriptGenerator (depends on KeyMapping, ComboMapping, KeyActionType)

Common Widgets:
  - KeyValidityChip (Material only)
  - SuggestionChip (Material only)
  - CategoryBadge (depends on validation_models)

Editor Widgets (depend on models, utils, common widgets):
  - KeyRegistryBanner (Material only)
  - ComboConfigRow (depends on ComboMapping)
  - KeyConfigPanel (depends on KeyActionType, KeyMappings)
  - MappingListPanel (depends on KeyMapping, LayerPanel, KeyValidityChip, KeyMappings)
  - ValidationBanner (depends on bridge.dart)
  - ValidationDialogs (depends on bridge.dart)
  - ValidationBannerRich (depends on validation_models, SuggestionChip, CategoryBadge)
```

## Statistics Summary
- **Total lines**: 1036
- **Data Models**: 3 classes, 56 lines
- **Utilities**: 2 classes, 88 lines
- **Common Widgets**: 3 widgets, 76 lines
- **Editor Widgets**: 12+ components, 679 lines
- **Imports**: 4 external dependencies

## Key Findings
1. The file is well-structured but too large (1036 lines exceeds 500 line guideline)
2. Clear separation between models, utilities, and widgets
3. Some private widgets (`_SuggestionChip`, `_CategoryBadge`) should be made public for reusability
4. ValidationBannerRich is the most complex widget (217 lines with nested components)
5. Good use of composition - widgets use other widgets appropriately
6. No circular dependencies detected - extraction will be straightforward
7. External dependencies are minimal and well-defined

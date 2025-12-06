/// Visual keymap editor page with profile-based mapping.
///
/// Provides a visual interface for creating and editing key mapping profiles
/// using the DragDropMapper widget with dynamic layout rendering.
library;

import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:uuid/uuid.dart';

import '../models/layout_type.dart';
import '../models/profile.dart';
import '../services/profile_registry_service.dart';
import '../services/service_registry.dart';
import '../widgets/drag_drop_mapper.dart';
import '../widgets/layout_grid.dart';
import 'visual_editor_widgets.dart' show InlineMessage, InlineMessageVariant;

const _uuid = Uuid();
const double _minPreviewWidth = 380;
const double _minPreviewHeight = 240;
const int _maxMatrixRows = 12;
const int _maxColsPerRow = 20;

/// Visual editor page for profile-based key mapping.
class VisualEditorPage extends StatefulWidget {
  const VisualEditorPage({super.key});

  @override
  State<VisualEditorPage> createState() => _VisualEditorPageState();
}

class _VisualEditorPageState extends State<VisualEditorPage> {
  String? _selectedProfileId;
  Profile? _currentProfile;
  List<String> _profileIds = [];
  bool _isLoading = false;
  String? _errorMessage;
  LayoutInfo? _currentLayoutInfo;
  List<int> _layoutRows = [];
  String? _layoutValidationMessage;
  final Map<String, LayoutInfo> _layoutOverrides = {};

  ProfileRegistryService get _profileService {
    try {
      final registry = Provider.of<ServiceRegistry>(context, listen: false);
      return registry.profileRegistryService;
    } on ProviderNotFoundException {
      return Provider.of<ProfileRegistryService>(context, listen: false);
    }
  }

  @override
  void initState() {
    super.initState();
    _loadProfiles();
  }

  Future<void> _loadProfiles() async {
    setState(() {
      _isLoading = true;
      _errorMessage = null;
    });

    try {
      final profileIds = await _profileService.listProfiles();
      setState(() {
        _profileIds = profileIds;
        _isLoading = false;
      });

      // Auto-select first profile if available
      if (_profileIds.isNotEmpty && _selectedProfileId == null) {
        await _loadProfile(_profileIds.first);
      }
    } catch (e) {
      setState(() {
        _isLoading = false;
        _errorMessage = 'Failed to load profiles: $e';
      });
    }
  }

  Future<void> _loadProfile(String profileId) async {
    setState(() {
      _isLoading = true;
      _errorMessage = null;
    });

    try {
      final profile = await _profileService.getProfile(profileId);
      if (profile == null) {
        setState(() {
          _isLoading = false;
          _errorMessage = 'Failed to load profile: $profileId not found';
        });
        return;
      }

      final layoutInfo =
          _layoutOverrides[profileId] ?? _getLayoutInfoForProfile(profile);

      setState(() {
        _selectedProfileId = profileId;
        _currentProfile = profile;
        _currentLayoutInfo = layoutInfo;
        _layoutRows = _buildRowsFromLayout(layoutInfo);
        _layoutValidationMessage = _validateLayoutRows(_layoutRows);
        _isLoading = false;
      });
    } catch (e) {
      setState(() {
        _isLoading = false;
        _errorMessage = 'Failed to load profile: $e';
      });
    }
  }

  Future<void> _createNewProfile() async {
    final name = await _showCreateProfileDialog();
    if (name == null || name.isEmpty) return;

    final layoutType = await _showLayoutTypeSelector();
    if (layoutType == null) return;

    final now = DateTime.now().toUtc().toIso8601String();
    final newProfile = Profile(
      id: _uuid.v4(),
      name: name,
      layoutType: layoutType,
      mappings: const {},
      createdAt: now,
      updatedAt: now,
    );

    setState(() {
      _isLoading = true;
      _errorMessage = null;
    });

    final result = await _profileService.saveProfile(newProfile);

    if (result.success) {
      await _loadProfiles();
      await _loadProfile(newProfile.id);

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Profile "${newProfile.name}" created')),
        );
      }
    } else {
      setState(() {
        _isLoading = false;
        _errorMessage = result.errorMessage ?? 'Failed to create profile';
      });
    }
  }

  Future<String?> _showCreateProfileDialog() async {
    final controller = TextEditingController();

    return showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Create New Profile'),
        content: TextField(
          controller: controller,
          autofocus: true,
          decoration: const InputDecoration(
            labelText: 'Profile Name',
            hintText: 'Enter a name for this profile',
          ),
          onSubmitted: (value) => Navigator.of(context).pop(value),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(controller.text),
            child: const Text('Create'),
          ),
        ],
      ),
    );
  }

  Future<LayoutType?> _showLayoutTypeSelector() async {
    return showDialog<LayoutType>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Select Layout Type'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(Icons.grid_3x3),
              title: const Text('Matrix'),
              subtitle: const Text('For macro pads and button grids'),
              onTap: () => Navigator.of(context).pop(LayoutType.matrix),
            ),
            ListTile(
              leading: const Icon(Icons.keyboard),
              title: const Text('Standard'),
              subtitle: const Text('For full-size keyboards'),
              onTap: () => Navigator.of(context).pop(LayoutType.standard),
            ),
            ListTile(
              leading: const Icon(Icons.keyboard_alt),
              title: const Text('Split'),
              subtitle: const Text('For split keyboards'),
              onTap: () => Navigator.of(context).pop(LayoutType.split),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    // Verify build is called with new code
    debugPrint('Building VisualEditorPage with layout fixes');

    return Scaffold(
      appBar: AppBar(
        title: const Text('Profiles'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: 'Refresh Profiles',
            onPressed: _loadProfiles,
          ),
        ],
      ),
      body: SafeArea(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // Profile selector toolbar
            SizedBox(
              height: 72, // Constrained height to prevent overflow
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 16),
                color: Theme.of(context).colorScheme.surfaceContainerHighest,
                child: Row(
                  children: [
                    const Icon(Icons.person_outline),
                    const SizedBox(width: 12),
                    const Text(
                      'Profile:',
                      style: TextStyle(fontWeight: FontWeight.w500),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: DropdownButton<String>(
                        value: _selectedProfileId,
                        hint: const Text('Select a profile'),
                        isExpanded: true,
                        underline: Container(),
                        items: _profileIds.map((id) {
                          return DropdownMenuItem(
                            value: id,
                            child: FutureBuilder<Profile?>(
                              future: _profileService.getProfile(id),
                              builder: (context, snapshot) {
                                if (snapshot.hasData && snapshot.data != null) {
                                  return Text(
                                    snapshot.data!.name,
                                    overflow: TextOverflow.ellipsis,
                                  );
                                }
                                return Text(
                                  id,
                                  overflow: TextOverflow.ellipsis,
                                );
                              },
                            ),
                          );
                        }).toList(),
                        onChanged: _isLoading
                            ? null
                            : (value) {
                                if (value != null) {
                                  _loadProfile(value);
                                }
                              },
                      ),
                    ),
                    const SizedBox(width: 12),
                    ElevatedButton.icon(
                      onPressed: _isLoading ? null : _createNewProfile,
                      icon: const Icon(Icons.add),
                      label: const Text('New Profile'),
                    ),
                  ],
                ),
              ),
            ),

            // Error message
            if (_errorMessage != null)
              Container(
                padding: const EdgeInsets.all(16),
                color: Theme.of(context).colorScheme.errorContainer,
                constraints: const BoxConstraints(maxHeight: 200),
                child: Row(
                  children: [
                    Icon(
                      Icons.error_outline,
                      color: Theme.of(context).colorScheme.onErrorContainer,
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: SingleChildScrollView(
                        child: Text(
                          _errorMessage!,
                          style: TextStyle(
                            color: Theme.of(
                              context,
                            ).colorScheme.onErrorContainer,
                          ),
                        ),
                      ),
                    ),
                    IconButton(
                      icon: const Icon(Icons.close),
                      onPressed: () => setState(() => _errorMessage = null),
                      color: Theme.of(context).colorScheme.onErrorContainer,
                    ),
                  ],
                ),
              ),

            // Main content
            Expanded(flex: 1, child: ClipRect(child: _buildContent())),
          ],
        ),
      ),
    );
  }

  Widget _buildContent() {
    if (_isLoading) {
      return const Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 16),
            Text('Loading...'),
          ],
        ),
      );
    }

    if (_currentProfile == null) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              Icons.description_outlined,
              size: 64,
              color: Theme.of(context).disabledColor,
            ),
            const SizedBox(height: 16),
            Text(
              'No profile selected',
              style: Theme.of(context).textTheme.titleLarge?.copyWith(
                color: Theme.of(context).disabledColor,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              _profileIds.isEmpty
                  ? 'Create a new profile to get started'
                  : 'Select a profile from the dropdown above',
              style: TextStyle(color: Theme.of(context).disabledColor),
            ),
            const SizedBox(height: 24),
            ElevatedButton.icon(
              onPressed: _createNewProfile,
              icon: const Icon(Icons.add),
              label: const Text('Create New Profile'),
            ),
          ],
        ),
      );
    }

    final layoutInfo =
        _currentLayoutInfo ?? _getLayoutInfoForProfile(_currentProfile!);

    return LayoutBuilder(
      builder: (context, constraints) {
        final mapperHeight = math.max(360.0, constraints.maxHeight * 0.55);

        return Padding(
          padding: const EdgeInsets.all(16),
          child: SingleChildScrollView(
            child: ConstrainedBox(
              constraints: BoxConstraints(minHeight: constraints.maxHeight),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  _buildLayoutEditorCard(layoutInfo),
                  const SizedBox(height: 16),
                  SizedBox(
                    height: mapperHeight,
                    child: DragDropMapper(
                      layoutInfo: layoutInfo,
                      profile: _currentProfile!,
                      profileRegistryService: _profileService,
                      onProfileUpdated: _handleProfileUpdated,
                      onSaveError: _handleSaveError,
                    ),
                  ),
                ],
              ),
            ),
          ),
        );
      },
    );
  }

  /// Build layout editor section for adjusting rows/columns before mapping.
  Widget _buildLayoutEditorCard(LayoutInfo layoutInfo) {
    final isMatrixLayout = _currentProfile?.layoutType == LayoutType.matrix;
    final totalKeys = _layoutRows.isNotEmpty
        ? _layoutRows.fold<int>(0, (sum, value) => sum + value)
        : layoutInfo.rows * layoutInfo.cols;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Wrap(
              spacing: 12,
              runSpacing: 8,
              crossAxisAlignment: WrapCrossAlignment.center,
              alignment: WrapAlignment.spaceBetween,
              children: [
                ConstrainedBox(
                  constraints: const BoxConstraints(
                    minWidth: 200,
                    maxWidth: 460,
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text(
                        'Profile Layout Editor',
                        style: TextStyle(fontWeight: FontWeight.w600),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        'Shape your physical grid before mapping keys.',
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: Theme.of(context).colorScheme.onSurfaceVariant,
                        ),
                      ),
                    ],
                  ),
                ),
                TextButton.icon(
                  onPressed: _resetLayoutToDefault,
                  icon: const Icon(Icons.restore),
                  label: const Text('Reset layout'),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              'Configure rows and per-row columns to mirror your physical board. '
              'We keep the preview wide enough to avoid cramped stacking.',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 12),
            if (isMatrixLayout)
              Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  _buildRowEditorList(),
                  if (_layoutValidationMessage != null)
                    InlineMessage(
                      message: _layoutValidationMessage!,
                      variant: InlineMessageVariant.error,
                    ),
                ],
              )
            else
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.surfaceContainerHighest,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Text(
                  'Layout editing is focused on matrix devices. '
                  'Using default ${layoutInfo.rows} rows × ${layoutInfo.cols} columns for ${layoutInfo.type.label}.',
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
              ),
            const SizedBox(height: 12),
            Wrap(
              spacing: 16,
              runSpacing: 8,
              children: [
                Chip(
                  avatar: const Icon(Icons.grid_on, size: 16),
                  label: Text(
                    '${layoutInfo.rows} rows × ${layoutInfo.cols} cols',
                  ),
                ),
                Chip(
                  avatar: const Icon(Icons.numbers, size: 16),
                  label: Text('$totalKeys keys defined'),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Text('Preview', style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 6),
            Text(
              'Preview enforces a minimum canvas so rows and columns stay readable.',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 12),
            _buildLayoutPreview(layoutInfo),
          ],
        ),
      ),
    );
  }

  /// Editable row controls for matrix layout.
  Widget _buildRowEditorList() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        ListView.separated(
          shrinkWrap: true,
          physics: const NeverScrollableScrollPhysics(),
          itemCount: _layoutRows.length,
          separatorBuilder: (_, __) => const SizedBox(height: 8),
          itemBuilder: (context, index) {
            final cols = _layoutRows[index];
            return Row(
              children: [
                Text(
                  'Row ${index + 1}',
                  style: const TextStyle(fontWeight: FontWeight.bold),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Slider(
                    value: cols.toDouble(),
                    min: 1,
                    max: _maxColsPerRow.toDouble(),
                    divisions: _maxColsPerRow - 1,
                    label: '$cols keys',
                    onChanged: (value) {
                      setState(() {
                        _layoutRows[index] = value.toInt();
                      });
                      _refreshLayoutInfoFromRows();
                    },
                  ),
                ),
                SizedBox(
                  width: 48,
                  child: Text(
                    '$cols',
                    textAlign: TextAlign.center,
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                ),
                IconButton(
                  icon: const Icon(Icons.delete_outline),
                  onPressed: _layoutRows.length > 1
                      ? () {
                          setState(() {
                            _layoutRows.removeAt(index);
                          });
                          _refreshLayoutInfoFromRows();
                        }
                      : null,
                  tooltip: 'Remove row',
                ),
              ],
            );
          },
        ),
        const SizedBox(height: 12),
        Row(
          children: [
            OutlinedButton.icon(
              onPressed: _handleAddRow,
              icon: const Icon(Icons.add),
              label: const Text('Add Row'),
            ),
            const SizedBox(width: 12),
            Text(
              '${_layoutRows.length} rows configured (max $_maxMatrixRows)',
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
          ],
        ),
      ],
    );
  }

  void _handleAddRow() {
    if (_layoutRows.length >= _maxMatrixRows) {
      setState(() {
        _layoutValidationMessage =
            'Limit layouts to $_maxMatrixRows rows to keep the preview readable.';
      });
      return;
    }

    setState(() {
      _layoutRows.add(_layoutRows.isNotEmpty ? _layoutRows.last : 5);
      _layoutValidationMessage = null;
    });
    _refreshLayoutInfoFromRows();
  }

  /// Preview of the current layout configuration.
  Widget _buildLayoutPreview(LayoutInfo layoutInfo) {
    final rows = _layoutRows.isNotEmpty ? _layoutRows.length : layoutInfo.rows;
    final cols = _layoutRows.isNotEmpty
        ? _layoutRows.fold<int>(1, (maxCols, value) => math.max(maxCols, value))
        : layoutInfo.cols;

    final previewLayout = layoutInfo.type == LayoutType.matrix
        ? LayoutInfo(
            rows: rows,
            cols: cols,
            type: layoutInfo.type,
            colsPerRow: List<int>.from(
              _layoutRows.isNotEmpty
                  ? _layoutRows
                  : _buildRowsFromLayout(layoutInfo),
            ),
          )
        : layoutInfo;

    return LayoutBuilder(
      builder: (context, constraints) {
        final availableWidth = constraints.maxWidth.isFinite
            ? constraints.maxWidth
            : _minPreviewWidth;
        final constrainedWidth = math.max(_minPreviewWidth, availableWidth);
        final widestRow =
            previewLayout.colsPerRow?.reduce(math.max) ?? previewLayout.cols;
        final computedKeySize =
            ((constrainedWidth - 48) / math.max(1, widestRow)).clamp(
              28.0,
              64.0,
            );
        final keySpacing = computedKeySize < 36 ? 4.0 : 6.0;

        final preview = ConstrainedBox(
          constraints: BoxConstraints(
            minWidth: constrainedWidth,
            minHeight: _minPreviewHeight,
          ),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 180),
            curve: Curves.easeOut,
            decoration: BoxDecoration(
              border: Border.all(
                color: Theme.of(context).colorScheme.outlineVariant,
              ),
              borderRadius: BorderRadius.circular(12),
            ),
            padding: const EdgeInsets.all(12),
            child: Center(
              child: LayoutGrid(
                layoutInfo: previewLayout,
                keySize: computedKeySize.toDouble(),
                keySpacing: keySpacing,
              ),
            ),
          ),
        );

        return SingleChildScrollView(
          scrollDirection: Axis.horizontal,
          child: preview,
        );
      },
    );
  }

  /// Update current profile state when DragDropMapper saves successfully.
  void _handleProfileUpdated(Profile updatedProfile) {
    setState(() {
      _currentProfile = updatedProfile;
      _selectedProfileId = updatedProfile.id;
      _layoutOverrides.putIfAbsent(
        updatedProfile.id,
        () => _currentLayoutInfo ?? _getLayoutInfoForProfile(updatedProfile),
      );
    });

    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Profile "${updatedProfile.name}" saved')),
      );
    }
  }

  /// Show save error surfaced from DragDropMapper.
  void _handleSaveError(String errorMessage) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(errorMessage),
        backgroundColor: Theme.of(context).colorScheme.error,
      ),
    );
  }

  /// Recalculate layout info from editable row data.
  String? _validateLayoutRows(List<int> rows) {
    if (rows.isEmpty) return 'Add at least one row to preview.';
    if (rows.length > _maxMatrixRows) {
      return 'Keep layouts to $_maxMatrixRows rows or fewer for readability.';
    }
    final invalidIndex = rows.indexWhere(
      (value) => value < 1 || value > _maxColsPerRow,
    );
    if (invalidIndex != -1) {
      return 'Row ${invalidIndex + 1} must have between 1 and $_maxColsPerRow columns.';
    }
    return null;
  }

  void _refreshLayoutInfoFromRows() {
    if (_currentProfile == null) return;

    final sanitizedRows = _layoutRows
        .map((value) => value.clamp(1, _maxColsPerRow))
        .map((value) => value.toInt())
        .toList();
    final validation = _validateLayoutRows(sanitizedRows);
    if (validation != null) {
      setState(() {
        _layoutRows = sanitizedRows;
        _layoutValidationMessage = validation;
      });
      return;
    }

    final rows = sanitizedRows.isNotEmpty ? sanitizedRows.length : 1;
    final cols = sanitizedRows.isNotEmpty
        ? sanitizedRows.fold<int>(
            1,
            (maxCols, value) => math.max(maxCols, value),
          )
        : 1;

    final updatedLayout = LayoutInfo(
      rows: rows,
      cols: cols,
      type: _currentProfile!.layoutType,
      colsPerRow: List<int>.from(
        sanitizedRows.isNotEmpty ? sanitizedRows : [cols],
      ),
    );

    setState(() {
      _layoutRows = sanitizedRows;
      _layoutValidationMessage = null;
      _currentLayoutInfo = updatedLayout;
      _layoutOverrides[_currentProfile!.id] = updatedLayout;
    });
  }

  /// Build editable rows list from layout info, falling back to uniform grid.
  List<int> _buildRowsFromLayout(LayoutInfo layoutInfo) {
    if (layoutInfo.colsPerRow != null && layoutInfo.colsPerRow!.isNotEmpty) {
      return layoutInfo.colsPerRow!
          .map((value) => value.clamp(1, _maxColsPerRow).toInt())
          .take(_maxMatrixRows)
          .toList();
    }
    final rows = math.min(layoutInfo.rows, _maxMatrixRows);
    final cols = math.min(layoutInfo.cols, _maxColsPerRow);
    return List<int>.filled(math.max(1, rows), math.max(1, cols));
  }

  /// Reset layout editor to defaults for the current profile.
  void _resetLayoutToDefault() {
    if (_currentProfile == null) return;
    final defaultLayout = _getLayoutInfoForProfile(_currentProfile!);
    setState(() {
      _layoutRows = _buildRowsFromLayout(defaultLayout);
      _currentLayoutInfo = defaultLayout;
      _layoutOverrides[_currentProfile!.id] = defaultLayout;
      _layoutValidationMessage = _validateLayoutRows(_layoutRows);
    });
  }

  /// Get layout info for a profile based on its layout type.
  ///
  /// TODO: This should eventually come from device definitions or be
  /// configurable when creating profiles.
  LayoutInfo _getLayoutInfoForProfile(Profile profile) {
    switch (profile.layoutType) {
      case LayoutType.matrix:
        // Default to 5x5 matrix for now
        return const LayoutInfo(
          rows: 5,
          cols: 5,
          type: LayoutType.matrix,
          colsPerRow: [5, 5, 5, 5, 5],
        );
      case LayoutType.standard:
        // Standard keyboard layout (simplified)
        return const LayoutInfo(rows: 6, cols: 15, type: LayoutType.standard);
      case LayoutType.split:
        // Split keyboard layout
        return const LayoutInfo(rows: 5, cols: 14, type: LayoutType.split);
    }
  }
}

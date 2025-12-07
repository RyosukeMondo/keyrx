/// Dedicated Mapping page for per-key assignment with search and density controls.
library;

import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../models/layout_type.dart';
import '../models/profile.dart';
import '../services/profile_autosave_service.dart';
import '../services/profile_registry_service.dart';
import '../services/service_registry.dart';
import '../widgets/layout_grid.dart';
import '../widgets/soft_keyboard.dart';
import 'visual_editor_widgets.dart' show InlineMessage, InlineMessageVariant;

const double _minPreviewWidth = 420;
const double _minPreviewHeight = 320;

/// Density options for the mapping grid.
enum MappingDensity { comfortable, compact }

/// Mapping page showing the profile grid with search highlights and an editor dialog.
class MappingPage extends StatefulWidget {
  const MappingPage({super.key});

  @override
  State<MappingPage> createState() => _MappingPageState();
}

class _MappingPageState extends State<MappingPage> {
  List<String> _profileIds = [];
  Profile? _currentProfile;
  LayoutInfo? _layoutInfo;
  String? _selectedProfileId;
  PhysicalPosition? _selectedPosition;
  String? _selectedOutputKey;
  Set<PhysicalPosition> _highlightedPositions = {};
  bool _isLoading = false;
  String? _errorMessage;
  String _searchQuery = '';
  MappingDensity _density = MappingDensity.comfortable;
  final TextEditingController _searchController = TextEditingController();
  AutosaveStatus _autosaveStatus = const AutosaveStatus(
    state: AutosaveState.idle,
  );
  StreamSubscription<AutosaveStatus>? _autosaveSubscription;

  ProfileRegistryService get _profileService {
    try {
      final registry = Provider.of<ServiceRegistry>(context, listen: false);
      return registry.profileRegistryService;
    } on ProviderNotFoundException {
      return Provider.of<ProfileRegistryService>(context, listen: false);
    }
  }

  ProfileAutosaveService get _autosaveService {
    try {
      final registry = Provider.of<ServiceRegistry>(context, listen: false);
      return registry.profileAutosaveService;
    } on ProviderNotFoundException {
      return Provider.of<ProfileAutosaveService>(context, listen: false);
    }
  }

  @override
  void initState() {
    super.initState();
    _listenToAutosave();
    _loadProfiles();
  }

  @override
  void dispose() {
    _autosaveSubscription?.cancel();
    _searchController.dispose();
    super.dispose();
  }

  Future<void> _loadProfiles() async {
    setState(() {
      _isLoading = true;
      _errorMessage = null;
    });

    try {
      final ids = await _profileService.listProfiles();
      _profileIds = ids;

      if (ids.isEmpty) {
        setState(() {
          _isLoading = false;
          _currentProfile = null;
          _layoutInfo = null;
          _selectedProfileId = null;
          _highlightedPositions = {};
        });
        return;
      }

      final targetId = _selectedProfileId ?? ids.first;
      await _loadProfile(targetId);
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
      _selectedProfileId = profileId;
    });

    try {
      final profile = await _profileService.getProfile(profileId);
      if (profile == null) {
        setState(() {
          _isLoading = false;
          _currentProfile = null;
          _layoutInfo = null;
          _errorMessage = 'Profile $profileId not found.';
        });
        return;
      }

      final layout = _buildLayoutInfo(profile);
      final highlights = _computeHighlights(profile, _searchQuery);

      setState(() {
        _currentProfile = profile;
        _layoutInfo = layout;
        _highlightedPositions = highlights;
        _isLoading = false;
      });
    } catch (e) {
      setState(() {
        _isLoading = false;
        _errorMessage = 'Failed to load profile: $e';
      });
    }
  }

  LayoutInfo _buildLayoutInfo(Profile profile) {
    switch (profile.layoutType) {
      case LayoutType.matrix:
        return const LayoutInfo(
          rows: 5,
          cols: 5,
          type: LayoutType.matrix,
          colsPerRow: [5, 5, 5, 5, 5],
        );
      case LayoutType.standard:
        return const LayoutInfo(rows: 6, cols: 15, type: LayoutType.standard);
      case LayoutType.split:
        return const LayoutInfo(rows: 5, cols: 14, type: LayoutType.split);
    }
  }

  Set<PhysicalPosition> _computeHighlights(Profile profile, String query) {
    if (query.isEmpty) return {};
    final normalized = query.toLowerCase();
    final matches = <PhysicalPosition>{};

    profile.mappings.forEach((key, action) {
      final label = keyActionLabel(action).toLowerCase();
      if (label.contains(normalized) ||
          key.toLowerCase().contains(normalized)) {
        final position = PhysicalPosition.fromKey(key);
        if (position != null) {
          matches.add(position);
        }
      }
    });

    return matches;
  }

  void _handleSearchChanged(String value) {
    final profile = _currentProfile;
    if (profile == null) return;

    setState(() {
      _searchQuery = value;
      _highlightedPositions = _computeHighlights(profile, value);
    });
  }

  void _handleDensityChanged(MappingDensity density) {
    setState(() {
      _density = density;
    });
  }

  void _handleKeyTap(int row, int col) {
    final position = PhysicalPosition(row: row, col: col);
    setState(() {
      _selectedPosition = position;
      _selectedOutputKey = null;
    });

    // Check if mapped
    final action = _currentProfile?.getAction(position);
    if (action != null) {
      action.when(
        key: (k) => setState(() => _selectedOutputKey = k),
        chord: (_) {},
        script: (_) {},
        block: () {},
        pass: () {},
      );
    }
  }

  Future<void> _handleOutputKeySelected(String key) async {
    setState(() {
      _selectedOutputKey = key;
    });
    await _applyMapping(key);
  }

  Future<void> _applyMapping(String keyVariant) async {
    final profile = _currentProfile;
    final position = _selectedPosition;
    if (profile == null || position == null) return;

    final updatedMappings = Map<String, KeyAction>.from(profile.mappings);
    updatedMappings[position.toKey()] = KeyAction.key(key: keyVariant);

    final updatedProfile = profile.copyWith(
      mappings: updatedMappings,
      updatedAt: DateTime.now().toUtc().toIso8601String(),
    );

    _autosaveService.queueSave(updatedProfile);

    if (mounted) {
      setState(() {
        _currentProfile = updatedProfile;
        _highlightedPositions = _computeHighlights(
          updatedProfile,
          _searchQuery,
        );
        // "Fold" the palette - clear selection
        _selectedPosition = null;
        _selectedOutputKey = null;
      });
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Mapping saved'),
          duration: Duration(milliseconds: 1000),
          behavior: SnackBarBehavior.floating,
        ),
      );
    }
  }

  Future<void> _clearMapping(PhysicalPosition position) async {
    final profile = _currentProfile;
    if (profile == null) return;

    final updatedMappings = Map<String, KeyAction>.from(profile.mappings)
      ..remove(position.toKey());
    final updatedProfile = profile.copyWith(
      mappings: updatedMappings,
      updatedAt: DateTime.now().toUtc().toIso8601String(),
    );

    _autosaveService.queueSave(updatedProfile);

    if (mounted) {
      setState(() {
        _currentProfile = updatedProfile;
        _highlightedPositions = _computeHighlights(
          updatedProfile,
          _searchQuery,
        );
      });
    }
  }

  void _listenToAutosave() {
    _autosaveStatus = _autosaveService.status;
    _autosaveSubscription = _autosaveService.statusStream.listen(
      _handleAutosaveStatus,
    );
  }

  void _handleAutosaveStatus(AutosaveStatus status) {
    final currentProfileId = _currentProfile?.id;
    if (status.profileId != null && status.profileId != currentProfileId) {
      return;
    }

    if (!mounted) return;

    final previousState = _autosaveStatus.state;
    setState(() {
      _autosaveStatus = status;
    });

    if (status.state == AutosaveState.error &&
        previousState != AutosaveState.error) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(status.errorMessage ?? 'Autosave failed.'),
          backgroundColor: Theme.of(context).colorScheme.error,
        ),
      );
    } else if (status.state == AutosaveState.success &&
        previousState != AutosaveState.success &&
        status.lastSavedAt != null) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Changes saved at ${_formatTime(status.lastSavedAt!)}'),
        ),
      );
    }
  }

  Widget _buildAutosaveBanner() {
    final status = _autosaveStatus;

    String? message;
    InlineMessageVariant variant = InlineMessageVariant.info;

    switch (status.state) {
      case AutosaveState.queued:
        message =
            'Autosave queued${status.profileId != null ? ' for ${status.profileId}' : ''}.';
        variant = InlineMessageVariant.info;
        break;
      case AutosaveState.saving:
        final attempt = status.attempt;
        final dir = status.targetDirectory ?? 'profiles directory';
        message = 'Saving changes (attempt $attempt) to $dir...';
        variant = InlineMessageVariant.info;
        break;
      case AutosaveState.success:
        final timestamp = status.lastSavedAt != null
            ? _formatTime(status.lastSavedAt!)
            : null;
        message = 'Last saved${timestamp != null ? ' at $timestamp' : ''}.';
        variant = InlineMessageVariant.success;
        break;
      case AutosaveState.error:
        message = status.errorMessage ?? 'Autosave failed.';
        variant = InlineMessageVariant.error;
        break;
      case AutosaveState.idle:
        if (status.lastSavedAt != null) {
          message = 'Last saved at ${_formatTime(status.lastSavedAt!)} (idle).';
          variant = InlineMessageVariant.success;
        }
        break;
    }

    if (message == null) {
      return const SizedBox.shrink();
    }

    return InlineMessage(
      message: message,
      variant: variant,
      icon: switch (variant) {
        InlineMessageVariant.success => Icons.check_circle_outline,
        InlineMessageVariant.error => Icons.warning_amber_rounded,
        _ => Icons.sync,
      },
    );
  }

  String _formatTime(DateTime timestamp) {
    final local = timestamp.toLocal();
    final hour = local.hour.toString().padLeft(2, '0');
    final minute = local.minute.toString().padLeft(2, '0');
    final second = local.second.toString().padLeft(2, '0');
    return '$hour:$minute:$second';
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Mapping')),
      body: SafeArea(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 16, 16, 0),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  _buildProfileToolbar(),
                  const SizedBox(height: 12),
                  _buildSearchAndDensity(),
                  if (_errorMessage != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 8),
                      child: InlineMessage(
                        message: _errorMessage!,
                        variant: InlineMessageVariant.error,
                      ),
                    ),
                  Padding(
                    padding: const EdgeInsets.only(top: 8),
                    child: _buildAutosaveBanner(),
                  ),
                ],
              ),
            ),
            Expanded(
              child: LayoutBuilder(
                builder: (context, constraints) {
                  if (_isLoading) {
                    return const Center(child: CircularProgressIndicator());
                  }
                  if (_currentProfile == null || _layoutInfo == null) {
                    return _buildEmptyState();
                  }

                  return _buildGridLayout(_layoutInfo!, constraints.maxWidth);
                },
              ),
            ),
            _buildPalettePanel(),
          ],
        ),
      ),
    );
  }

  Widget _buildProfileToolbar() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: Theme.of(context).colorScheme.outlineVariant),
      ),
      child: Row(
        children: [
          const Icon(Icons.person_outline),
          const SizedBox(width: 12),
          Text('Profile', style: Theme.of(context).textTheme.titleMedium),
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
                      return Text(id, overflow: TextOverflow.ellipsis);
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
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: 'Refresh profiles',
            onPressed: _isLoading ? null : _loadProfiles,
          ),
        ],
      ),
    );
  }

  Widget _buildSearchAndDensity() {
    final matchesLabel = _searchQuery.isEmpty
        ? 'Search mappings'
        : '${_highlightedPositions.length} match${_highlightedPositions.length == 1 ? '' : 'es'}';

    return Row(
      children: [
        Expanded(
          child: TextField(
            controller: _searchController,
            decoration: InputDecoration(
              hintText: 'Search by mapped key or position...',
              prefixIcon: const Icon(Icons.search),
              suffixIcon: _searchQuery.isNotEmpty
                  ? IconButton(
                      icon: const Icon(Icons.clear),
                      onPressed: () {
                        _searchController.clear();
                        _handleSearchChanged('');
                      },
                    )
                  : null,
              border: OutlineInputBorder(
                borderRadius: BorderRadius.circular(12),
              ),
            ),
            onChanged: _handleSearchChanged,
          ),
        ),
        const SizedBox(width: 12),
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(matchesLabel, style: Theme.of(context).textTheme.bodySmall),
            const SizedBox(height: 6),
            SegmentedButton<MappingDensity>(
              segments: const [
                ButtonSegment(
                  value: MappingDensity.comfortable,
                  icon: Icon(Icons.grid_view),
                  label: Text('Comfort'),
                ),
                ButtonSegment(
                  value: MappingDensity.compact,
                  icon: Icon(Icons.density_small),
                  label: Text('Compact'),
                ),
              ],
              selected: {_density},
              onSelectionChanged: (values) {
                _handleDensityChanged(values.first);
              },
              showSelectedIcon: false,
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildGridLayout(LayoutInfo layoutInfo, double maxWidth) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: LayoutBuilder(
        builder: (context, constraints) {
          final availableWidth = constraints.maxWidth.isFinite
              ? constraints.maxWidth
              : maxWidth;
          final widestRow =
              layoutInfo.colsPerRow?.reduce(math.max) ?? layoutInfo.cols;
          final baseKeySize =
              (availableWidth - 64) / math.max(1, widestRow.toDouble());
          final keySize = _clampedKeySize(baseKeySize);
          final keySpacing = _density == MappingDensity.comfortable ? 8.0 : 4.0;

          return Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Chip(
                    avatar: const Icon(Icons.grid_on, size: 16),
                    label: Text('${layoutInfo.rows} rows'),
                    visualDensity: VisualDensity.compact,
                  ),
                  const SizedBox(width: 8),
                  Chip(
                    avatar: const Icon(Icons.numbers, size: 16),
                    label: Text('$widestRow max cols'),
                    visualDensity: VisualDensity.compact,
                  ),
                  const SizedBox(width: 8),
                  Chip(
                    avatar: const Icon(Icons.apps, size: 16),
                    label: Text('${_currentProfile?.mappingCount ?? 0} mapped'),
                    visualDensity: VisualDensity.compact,
                  ),
                ],
              ),
              const SizedBox(height: 12),
              Expanded(
                child: Center(
                  child: SingleChildScrollView(
                    scrollDirection: Axis.horizontal,
                    child: ConstrainedBox(
                      constraints: BoxConstraints(
                        minWidth: math.max(_minPreviewWidth, availableWidth),
                        minHeight: _minPreviewHeight,
                      ),
                      child: LayoutGrid(
                        layoutInfo: layoutInfo,
                        profile: _currentProfile,
                        onKeyTap: _handleKeyTap,
                        selectedPosition: _selectedPosition,
                        highlightedPositions: _highlightedPositions,
                        keySize: keySize,
                        keySpacing: keySpacing,
                      ),
                    ),
                  ),
                ),
              ),
            ],
          );
        },
      ),
    );
  }

  double _clampedKeySize(double base) {
    final double minSize;
    final double maxSize;
    if (_density == MappingDensity.comfortable) {
      minSize = 48;
      maxSize = 76;
    } else {
      minSize = 36;
      maxSize = 58;
    }
    return base.clamp(minSize, maxSize);
  }

  Widget _buildPalettePanel() {
    final show = _selectedPosition != null;
    final theme = Theme.of(context);
    final action = _selectedPosition != null
        ? _currentProfile?.getAction(_selectedPosition!)
        : null;

    return AnimatedContainer(
      duration: const Duration(milliseconds: 300),
      height: show ? 400 : 0,
      curve: Curves.easeInOutCubicEmphasized,
      decoration: BoxDecoration(
        color: theme.colorScheme.surface,
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.1),
            blurRadius: 8,
            offset: const Offset(0, -2),
          ),
        ],
        borderRadius: const BorderRadius.vertical(top: Radius.circular(16)),
      ),
      child: ClipRRect(
        borderRadius: const BorderRadius.vertical(top: Radius.circular(16)),
        child: show
            ? Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Container(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 16,
                      vertical: 8,
                    ),
                    color: theme.colorScheme.surfaceContainerHighest,
                    child: Row(
                      children: [
                        Icon(
                          Icons.keyboard,
                          size: 18,
                          color: theme.colorScheme.primary,
                        ),
                        const SizedBox(width: 8),
                        Text(
                          'Select mapping for ${_selectedPosition?.toKey()}',
                          style: theme.textTheme.titleSmall,
                        ),
                        const SizedBox(width: 12),
                        if (action != null)
                          Chip(
                            label: Text(keyActionLabel(action)),
                            backgroundColor: theme.colorScheme.primaryContainer,
                            labelStyle: TextStyle(
                              color: theme.colorScheme.onPrimaryContainer,
                              fontSize: 11,
                            ),
                            visualDensity: VisualDensity.compact,
                            padding: EdgeInsets.zero,
                            onDeleted: () => _clearMapping(_selectedPosition!),
                          ),
                        const Spacer(),
                        IconButton(
                          icon: const Icon(Icons.close, size: 20),
                          onPressed: () {
                            setState(() {
                              _selectedPosition = null;
                              _selectedOutputKey = null;
                            });
                          },
                          tooltip: 'Close',
                        ),
                      ],
                    ),
                  ),
                  Expanded(
                    child: SoftKeyboard(
                      onKeySelected: _handleOutputKeySelected,
                      selectedKey: _selectedOutputKey,
                    ),
                  ),
                ],
              )
            : const SizedBox.shrink(),
      ),
    );
  }

  Widget _buildEmptyState() {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            Icons.layers_clear,
            size: 64,
            color: Theme.of(context).disabledColor,
          ),
          const SizedBox(height: 12),
          Text(
            'No profiles available',
            style: Theme.of(context).textTheme.titleLarge?.copyWith(
              color: Theme.of(context).disabledColor,
            ),
          ),
          const SizedBox(height: 8),
          const Text('Create a profile in the Profiles page to start mapping.'),
          const SizedBox(height: 16),
          OutlinedButton.icon(
            onPressed: _isLoading ? null : _loadProfiles,
            icon: const Icon(Icons.refresh),
            label: const Text('Refresh'),
          ),
        ],
      ),
    );
  }
}

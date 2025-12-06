/// Dedicated Mapping page for per-key assignment with search and density controls.
library;

import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../models/layout_type.dart';
import '../models/profile.dart';
import '../services/profile_registry_service.dart';
import '../services/service_registry.dart';
import '../widgets/drag_drop_mapper.dart';
import '../widgets/layout_grid.dart';
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
  Set<PhysicalPosition> _highlightedPositions = {};
  bool _isLoading = false;
  String? _errorMessage;
  String _searchQuery = '';
  MappingDensity _density = MappingDensity.comfortable;
  final TextEditingController _searchController = TextEditingController();

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

  @override
  void dispose() {
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
    });
    _openMappingEditor();
  }

  Future<void> _openMappingEditor() async {
    final profile = _currentProfile;
    final layout = _layoutInfo;
    final position = _selectedPosition;

    if (profile == null || layout == null || position == null) return;

    final updatedProfile = await showModalBottomSheet<Profile>(
      context: context,
      isScrollControlled: true,
      builder: (ctx) {
        return Padding(
          padding: EdgeInsets.only(
            bottom: MediaQuery.of(ctx).viewInsets.bottom,
            top: 16,
          ),
          child: FractionallySizedBox(
            heightFactor: 0.9,
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: DragDropMapper(
                layoutInfo: layout,
                profile: profile,
                profileRegistryService: _profileService,
                initialSelectedPosition: position,
                onProfileUpdated: (updated) {
                  Navigator.of(ctx).pop(updated);
                },
                onSaveError: (message) {
                  ScaffoldMessenger.of(
                    ctx,
                  ).showSnackBar(SnackBar(content: Text(message)));
                },
              ),
            ),
          ),
        );
      },
    );

    if (updatedProfile != null && mounted) {
      setState(() {
        _currentProfile = updatedProfile;
        _highlightedPositions = _computeHighlights(
          updatedProfile,
          _searchQuery,
        );
      });
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('Mapping updated')));
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

    final result = await _profileService.saveProfile(updatedProfile);
    if (!result.success) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(result.errorMessage ?? 'Failed to clear mapping.'),
            backgroundColor: Theme.of(context).colorScheme.errorContainer,
          ),
        );
      }
      return;
    }

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

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Mapping')),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(16),
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
              const SizedBox(height: 12),
              Expanded(
                child: LayoutBuilder(
                  builder: (context, constraints) {
                    if (_isLoading) {
                      return const Center(child: CircularProgressIndicator());
                    }
                    if (_currentProfile == null || _layoutInfo == null) {
                      return _buildEmptyState();
                    }

                    return Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        Expanded(
                          child: _buildGridCard(
                            _layoutInfo!,
                            constraints.maxWidth,
                          ),
                        ),
                        const SizedBox(height: 12),
                        _buildSelectionDetails(),
                      ],
                    );
                  },
                ),
              ),
            ],
          ),
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

  Widget _buildGridCard(LayoutInfo layoutInfo, double maxWidth) {
    return Card(
      child: Padding(
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
            final keySpacing = _density == MappingDensity.comfortable
                ? 8.0
                : 4.0;

            return Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Row(
                  children: [
                    Chip(
                      avatar: const Icon(Icons.grid_on, size: 16),
                      label: Text('${layoutInfo.rows} rows'),
                    ),
                    const SizedBox(width: 8),
                    Chip(
                      avatar: const Icon(Icons.numbers, size: 16),
                      label: Text('$widestRow max cols'),
                    ),
                    const SizedBox(width: 8),
                    Chip(
                      avatar: const Icon(Icons.apps, size: 16),
                      label: Text(
                        '${_currentProfile?.mappingCount ?? 0} mapped',
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 12),
                Expanded(
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
              ],
            );
          },
        ),
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

  Widget _buildSelectionDetails() {
    if (_selectedPosition == null) {
      return const InlineMessage(
        message: 'Tap a key to inspect and edit its mapping.',
      );
    }

    final action = _currentProfile?.getAction(_selectedPosition!);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            Icon(Icons.push_pin, color: Theme.of(context).colorScheme.primary),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'Key ${_selectedPosition!.toKey()}',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  const SizedBox(height: 4),
                  Text(
                    action == null
                        ? 'No mapping assigned yet.'
                        : 'Mapped to ${keyActionLabel(action)}',
                    style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                      color: Theme.of(context).colorScheme.onSurfaceVariant,
                    ),
                  ),
                ],
              ),
            ),
            if (action != null) ...[
              TextButton.icon(
                onPressed: () => _clearMapping(_selectedPosition!),
                icon: const Icon(Icons.clear),
                label: const Text('Clear'),
              ),
              const SizedBox(width: 8),
            ],
            FilledButton.icon(
              onPressed: _openMappingEditor,
              icon: const Icon(Icons.edit),
              label: const Text('Edit mapping'),
            ),
          ],
        ),
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

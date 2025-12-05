/// Visual keymap editor page with profile-based mapping.
///
/// Provides a visual interface for creating and editing key mapping profiles
/// using the DragDropMapper widget with dynamic layout rendering.
library;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:uuid/uuid.dart';

import '../models/layout_type.dart';
import '../models/profile.dart';
import '../services/profile_registry_service.dart';
import '../widgets/drag_drop_mapper.dart';
import '../widgets/layout_grid.dart';

const _uuid = Uuid();

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

  ProfileRegistryService get _profileService =>
      Provider.of<ProfileRegistryService>(context, listen: false);

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
      setState(() {
        _selectedProfileId = profileId;
        _currentProfile = profile;
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

    final now = DateTime.now().toIso8601String();
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

  void _handleProfileUpdated(Profile updatedProfile) {
    setState(() {
      _currentProfile = updatedProfile;
    });
  }

  void _handleSaveError(String errorMessage) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(errorMessage),
        backgroundColor: Theme.of(context).colorScheme.error,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Visual Editor'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: 'Refresh Profiles',
            onPressed: _loadProfiles,
          ),
        ],
      ),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Profile selector toolbar
          Container(
            padding: const EdgeInsets.all(16),
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
                    items: _profileIds.map((id) {
                      return DropdownMenuItem(
                        value: id,
                        child: FutureBuilder<Profile?>(
                          future: _profileService.getProfile(id),
                          builder: (context, snapshot) {
                            if (snapshot.hasData && snapshot.data != null) {
                              return Text(snapshot.data!.name);
                            }
                            return Text(id);
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

          // Error message
          if (_errorMessage != null)
            Container(
              padding: const EdgeInsets.all(16),
              color: Theme.of(context).colorScheme.errorContainer,
              child: Row(
                children: [
                  Icon(
                    Icons.error_outline,
                    color: Theme.of(context).colorScheme.onErrorContainer,
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Text(
                      _errorMessage!,
                      style: TextStyle(
                        color: Theme.of(context).colorScheme.onErrorContainer,
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
          Expanded(
            child: _buildContent(),
          ),
        ],
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

    // Determine layout info from profile
    final layoutInfo = _getLayoutInfoForProfile(_currentProfile!);

    return Padding(
      padding: const EdgeInsets.all(16),
      child: DragDropMapper(
        layoutInfo: layoutInfo,
        profile: _currentProfile!,
        profileRegistryService: _profileService,
        onProfileUpdated: _handleProfileUpdated,
        onSaveError: _handleSaveError,
      ),
    );
  }

  /// Get layout info for a profile based on its layout type.
  ///
  /// TODO: This should eventually come from device definitions or be
  /// configurable when creating profiles.
  LayoutInfo _getLayoutInfoForProfile(Profile profile) {
    switch (profile.layoutType) {
      case LayoutType.matrix:
        // Default to 5x5 matrix for now
        return const LayoutInfo(rows: 5, cols: 5, type: LayoutType.matrix);
      case LayoutType.standard:
        // Standard keyboard layout (simplified)
        return const LayoutInfo(rows: 6, cols: 15, type: LayoutType.standard);
      case LayoutType.split:
        // Split keyboard layout
        return const LayoutInfo(rows: 5, cols: 14, type: LayoutType.split);
    }
  }
}

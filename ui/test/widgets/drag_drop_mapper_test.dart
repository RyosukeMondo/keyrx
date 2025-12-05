import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/layout_type.dart';
import 'package:keyrx_ui/models/profile.dart';
import 'package:keyrx_ui/services/profile_registry_service.dart';
import 'package:keyrx_ui/widgets/drag_drop_mapper.dart';
import 'package:keyrx_ui/widgets/layout_grid.dart';
import 'package:keyrx_ui/widgets/soft_keyboard.dart';

/// Mock ProfileRegistryService for testing
class MockProfileRegistryService implements ProfileRegistryService {
  final List<Profile> _profiles = [];
  bool shouldFailSave = false;
  String? lastSavedProfileId;
  int saveCallCount = 0;

  @override
  Future<List<String>> listProfiles() async {
    return _profiles.map((p) => p.id).toList();
  }

  @override
  Future<Profile?> getProfile(String profileId) async {
    try {
      return _profiles.firstWhere((p) => p.id == profileId);
    } catch (_) {
      return null;
    }
  }

  @override
  Future<ProfileRegistryOperationResult> saveProfile(Profile profile) async {
    saveCallCount++;
    lastSavedProfileId = profile.id;

    if (shouldFailSave) {
      return ProfileRegistryOperationResult.error('Mock save error');
    }

    // Remove old profile with same ID if exists
    _profiles.removeWhere((p) => p.id == profile.id);
    _profiles.add(profile);

    return ProfileRegistryOperationResult.success();
  }

  @override
  Future<ProfileRegistryOperationResult> deleteProfile(String profileId) async {
    _profiles.removeWhere((p) => p.id == profileId);
    return ProfileRegistryOperationResult.success();
  }

  @override
  Future<List<Profile>> findCompatibleProfiles(LayoutType layoutType) async {
    return _profiles
        .where((p) => p.layoutType == layoutType)
        .toList();
  }

  @override
  Future<List<String>> refresh() async {
    return listProfiles();
  }

  @override
  Future<void> dispose() async {
    _profiles.clear();
  }
}

void main() {
  group('DragDropMapper Widget Tests', () {
    // Helper function to wrap widget in MaterialApp for testing
    Widget makeTestableWidget(Widget child) {
      return MaterialApp(
        home: Scaffold(
          body: child,
        ),
      );
    }

    // Sample profile for testing
    Profile createTestProfile({
      String id = 'test-profile-1',
      LayoutType layoutType = LayoutType.matrix,
      Map<String, KeyAction>? mappings,
    }) {
      return Profile(
        id: id,
        name: 'Test Profile',
        layoutType: layoutType,
        mappings: mappings ?? {},
        createdAt: DateTime.now().toIso8601String(),
        updatedAt: DateTime.now().toIso8601String(),
      );
    }

    group('Basic Rendering', () {
      testWidgets('renders both LayoutGrid and status bar',
          (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );
        final profile = createTestProfile();
        final service = MockProfileRegistryService();

        await tester.pumpWidget(
          makeTestableWidget(
            DragDropMapper(
              layoutInfo: layoutInfo,
              profile: profile,
              profileRegistryService: service,
            ),
          ),
        );

        // Should find the LayoutGrid widget
        expect(find.byType(LayoutGrid), findsOneWidget);

        // Should find status bar text
        expect(find.text('Step 1: Select a physical key from the layout'),
            findsOneWidget);

        // Should find section headers
        expect(find.text('Device Layout'), findsOneWidget);
        expect(find.text('Output Keys'), findsOneWidget);
      });

      testWidgets('shows disabled palette initially',
          (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );
        final profile = createTestProfile();
        final service = MockProfileRegistryService();

        await tester.pumpWidget(
          makeTestableWidget(
            DragDropMapper(
              layoutInfo: layoutInfo,
              profile: profile,
              profileRegistryService: service,
            ),
          ),
        );

        // Should NOT show SoftKeyboard initially
        expect(find.byType(SoftKeyboard), findsNothing);

        // Should show disabled state message
        expect(find.text('Select a physical key first'), findsWidgets);
      });
    });

    group('Two-Step Workflow', () {
      testWidgets('transitions to output selection after physical key tap',
          (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );
        final profile = createTestProfile();
        final service = MockProfileRegistryService();

        await tester.pumpWidget(
          makeTestableWidget(
            DragDropMapper(
              layoutInfo: layoutInfo,
              profile: profile,
              profileRegistryService: service,
            ),
          ),
        );

        // Initially in Step 1
        expect(find.text('Step 1: Select a physical key from the layout'),
            findsOneWidget);

        // Tap a physical key (position 0,0)
        final keyWidgets = find.byType(InkWell);
        await tester.tap(keyWidgets.first);
        await tester.pumpAndSettle();

        // Should transition to Step 2
        expect(find.text('Step 2: Select an output key from the palette'),
            findsOneWidget);

        // Should show selected position
        expect(find.text('Selected: 0,0'), findsOneWidget);

        // Should now show SoftKeyboard
        expect(find.byType(SoftKeyboard), findsOneWidget);
      });

      testWidgets('creates mapping and auto-saves when output key selected',
          (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );
        final profile = createTestProfile();
        final service = MockProfileRegistryService();
        Profile? updatedProfile;

        await tester.pumpWidget(
          makeTestableWidget(
            DragDropMapper(
              layoutInfo: layoutInfo,
              profile: profile,
              profileRegistryService: service,
              onProfileUpdated: (p) => updatedProfile = p,
            ),
          ),
        );

        // Step 1: Select physical key
        final keyWidgets = find.byType(InkWell);
        await tester.tap(keyWidgets.first);
        await tester.pumpAndSettle();

        // Step 2: Select output key
        // Find the SoftKeyboard and tap a key (we'll tap 'A')
        final softKeyboard = tester.widget<SoftKeyboard>(find.byType(SoftKeyboard));
        softKeyboard.onKeySelected!('A');
        await tester.pumpAndSettle();

        // Should have saved the profile
        expect(service.saveCallCount, 1);
        expect(service.lastSavedProfileId, profile.id);

        // Should have created the mapping
        expect(updatedProfile, isNotNull);
        expect(updatedProfile!.mappings.containsKey('0,0'), true);
        expect(updatedProfile!.mappings['0,0'],
            const KeyAction.key(key: 'A'));

        // Should reset to Step 1
        expect(find.text('Step 1: Select a physical key from the layout'),
            findsOneWidget);
      });

      testWidgets('completes save operation successfully',
          (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );
        final profile = createTestProfile();
        final service = MockProfileRegistryService();

        await tester.pumpWidget(
          makeTestableWidget(
            DragDropMapper(
              layoutInfo: layoutInfo,
              profile: profile,
              profileRegistryService: service,
            ),
          ),
        );

        // Select physical key
        final keyWidgets = find.byType(InkWell);
        await tester.tap(keyWidgets.first);
        await tester.pumpAndSettle();

        // Select output key
        final softKeyboard = tester.widget<SoftKeyboard>(find.byType(SoftKeyboard));
        softKeyboard.onKeySelected!('A');

        // Complete the save
        await tester.pumpAndSettle();

        // Should have saved
        expect(service.saveCallCount, 1);

        // Should reset to step 1 after save
        expect(find.text('Step 1: Select a physical key from the layout'),
            findsOneWidget);
      });
    });

    group('Editing Existing Mappings', () {
      testWidgets('pre-selects existing mapping when tapping mapped key',
          (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );
        final profile = createTestProfile(
          mappings: {
            '0,0': const KeyAction.key(key: 'A'),
          },
        );
        final service = MockProfileRegistryService();

        await tester.pumpWidget(
          makeTestableWidget(
            DragDropMapper(
              layoutInfo: layoutInfo,
              profile: profile,
              profileRegistryService: service,
            ),
          ),
        );

        // Tap the mapped key
        final keyWidgets = find.byType(InkWell);
        await tester.tap(keyWidgets.first);
        await tester.pumpAndSettle();

        // Should show SoftKeyboard with pre-selected key
        expect(find.byType(SoftKeyboard), findsOneWidget);
        final softKeyboard = tester.widget<SoftKeyboard>(find.byType(SoftKeyboard));
        expect(softKeyboard.selectedKey, 'A');
      });

      testWidgets('shows remove mapping button for mapped keys',
          (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );
        final profile = createTestProfile(
          mappings: {
            '0,0': const KeyAction.key(key: 'A'),
          },
        );
        final service = MockProfileRegistryService();

        await tester.pumpWidget(
          makeTestableWidget(
            DragDropMapper(
              layoutInfo: layoutInfo,
              profile: profile,
              profileRegistryService: service,
            ),
          ),
        );

        // Initially no remove button
        expect(find.text('Remove Mapping'), findsNothing);

        // Tap the mapped key
        final keyWidgets = find.byType(InkWell);
        await tester.tap(keyWidgets.first);
        await tester.pumpAndSettle();

        // Should show remove mapping button
        expect(find.text('Remove Mapping'), findsOneWidget);
      });

      testWidgets('removes mapping when remove button pressed',
          (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );
        final profile = createTestProfile(
          mappings: {
            '0,0': const KeyAction.key(key: 'A'),
          },
        );
        final service = MockProfileRegistryService();
        Profile? updatedProfile;

        await tester.pumpWidget(
          makeTestableWidget(
            DragDropMapper(
              layoutInfo: layoutInfo,
              profile: profile,
              profileRegistryService: service,
              onProfileUpdated: (p) => updatedProfile = p,
            ),
          ),
        );

        // Tap the mapped key
        final keyWidgets = find.byType(InkWell);
        await tester.tap(keyWidgets.first);
        await tester.pumpAndSettle();

        // Tap remove button
        await tester.tap(find.text('Remove Mapping'));
        await tester.pumpAndSettle();

        // Should have saved the profile
        expect(service.saveCallCount, 1);

        // Mapping should be removed
        expect(updatedProfile, isNotNull);
        expect(updatedProfile!.mappings.containsKey('0,0'), false);
      });
    });

    group('Error Handling', () {
      testWidgets('reverts changes on save error',
          (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );
        final profile = createTestProfile();
        final service = MockProfileRegistryService()..shouldFailSave = true;
        String? errorMessage;

        await tester.pumpWidget(
          makeTestableWidget(
            DragDropMapper(
              layoutInfo: layoutInfo,
              profile: profile,
              profileRegistryService: service,
              onSaveError: (msg) => errorMessage = msg,
            ),
          ),
        );

        // Select physical key
        final keyWidgets = find.byType(InkWell);
        await tester.tap(keyWidgets.first);
        await tester.pumpAndSettle();

        // Select output key
        final softKeyboard = tester.widget<SoftKeyboard>(find.byType(SoftKeyboard));
        softKeyboard.onKeySelected!('A');
        await tester.pumpAndSettle();

        // Should have called save
        expect(service.saveCallCount, 1);

        // Should have triggered error callback
        expect(errorMessage, isNotNull);
        expect(errorMessage, contains('Mock save error'));

        // Profile should not be updated (no mapping created)
        expect(profile.mappings.isEmpty, true);
      });

      testWidgets('remove button works correctly',
          (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );
        final profile = createTestProfile(
          mappings: {
            '0,0': const KeyAction.key(key: 'A'),
          },
        );
        final service = MockProfileRegistryService();

        await tester.pumpWidget(
          makeTestableWidget(
            DragDropMapper(
              layoutInfo: layoutInfo,
              profile: profile,
              profileRegistryService: service,
            ),
          ),
        );

        // Tap the mapped key to show remove button
        final keyWidgets = find.byType(InkWell);
        await tester.tap(keyWidgets.first);
        await tester.pumpAndSettle();

        // Find and tap remove button
        final removeButton = find.text('Remove Mapping');
        expect(removeButton, findsOneWidget);

        await tester.tap(removeButton);
        await tester.pumpAndSettle();

        // Should have saved the profile
        expect(service.saveCallCount, 1);
      });
    });

    group('Profile Updates', () {
      testWidgets('updates when external profile changes',
          (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );
        final initialProfile = createTestProfile();
        final service = MockProfileRegistryService();

        await tester.pumpWidget(
          makeTestableWidget(
            DragDropMapper(
              layoutInfo: layoutInfo,
              profile: initialProfile,
              profileRegistryService: service,
            ),
          ),
        );

        // Update with a new profile
        final updatedProfile = createTestProfile(
          mappings: {
            '1,1': const KeyAction.key(key: 'B'),
          },
        );

        await tester.pumpWidget(
          makeTestableWidget(
            DragDropMapper(
              layoutInfo: layoutInfo,
              profile: updatedProfile,
              profileRegistryService: service,
            ),
          ),
        );
        await tester.pumpAndSettle();

        // The widget should reflect the new profile
        // (This is tested by verifying the LayoutGrid gets the updated profile)
        final layoutGrid = tester.widget<LayoutGrid>(find.byType(LayoutGrid));
        expect(layoutGrid.profile, updatedProfile);
      });
    });

    group('Callback Tests', () {
      testWidgets('calls onProfileUpdated when mapping created',
          (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );
        final profile = createTestProfile();
        final service = MockProfileRegistryService();
        int updateCallCount = 0;
        Profile? lastUpdatedProfile;

        await tester.pumpWidget(
          makeTestableWidget(
            DragDropMapper(
              layoutInfo: layoutInfo,
              profile: profile,
              profileRegistryService: service,
              onProfileUpdated: (p) {
                updateCallCount++;
                lastUpdatedProfile = p;
              },
            ),
          ),
        );

        // Create a mapping
        final keyWidgets = find.byType(InkWell);
        await tester.tap(keyWidgets.first);
        await tester.pumpAndSettle();

        final softKeyboard = tester.widget<SoftKeyboard>(find.byType(SoftKeyboard));
        softKeyboard.onKeySelected!('A');
        await tester.pumpAndSettle();

        // Should have called callback once
        expect(updateCallCount, 1);
        expect(lastUpdatedProfile, isNotNull);
        expect(lastUpdatedProfile!.mappings['0,0'],
            const KeyAction.key(key: 'A'));
      });

      testWidgets('calls onSaveError when save fails',
          (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );
        final profile = createTestProfile();
        final service = MockProfileRegistryService()..shouldFailSave = true;
        int errorCallCount = 0;
        String? lastErrorMessage;

        await tester.pumpWidget(
          makeTestableWidget(
            DragDropMapper(
              layoutInfo: layoutInfo,
              profile: profile,
              profileRegistryService: service,
              onSaveError: (msg) {
                errorCallCount++;
                lastErrorMessage = msg;
              },
            ),
          ),
        );

        // Attempt to create a mapping
        final keyWidgets = find.byType(InkWell);
        await tester.tap(keyWidgets.first);
        await tester.pumpAndSettle();

        final softKeyboard = tester.widget<SoftKeyboard>(find.byType(SoftKeyboard));
        softKeyboard.onKeySelected!('A');
        await tester.pumpAndSettle();

        // Should have called error callback once
        expect(errorCallCount, 1);
        expect(lastErrorMessage, isNotNull);
        expect(lastErrorMessage, contains('Mock save error'));
      });
    });
  });
}

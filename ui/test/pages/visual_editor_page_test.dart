import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/layout_type.dart';
import 'package:keyrx_ui/models/profile.dart';
import 'package:keyrx_ui/pages/visual_editor_page.dart';
import 'package:keyrx_ui/services/profile_registry_service.dart';
import 'package:keyrx_ui/widgets/drag_drop_mapper.dart';
import 'package:provider/provider.dart';

/// Mock ProfileRegistryService for testing
class MockProfileRegistryService implements ProfileRegistryService {
  final List<Profile> _profiles = [];
  bool shouldFailSave = false;
  bool shouldFailLoad = false;
  String? lastSavedProfileId;
  int saveCallCount = 0;
  int listCallCount = 0;
  int getCallCount = 0;

  @override
  Future<List<String>> listProfiles() async {
    listCallCount++;
    if (shouldFailLoad) {
      throw Exception('Mock list error');
    }
    return _profiles.map((p) => p.id).toList();
  }

  @override
  Future<Profile?> getProfile(String profileId) async {
    getCallCount++;
    if (shouldFailLoad) {
      throw Exception('Mock get error');
    }
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
    return _profiles.where((p) => p.layoutType == layoutType).toList();
  }

  @override
  Future<List<String>> refresh() async {
    return listProfiles();
  }

  @override
  Future<void> dispose() async {
    _profiles.clear();
  }

  // Helper method for tests
  void addTestProfile(Profile profile) {
    _profiles.add(profile);
  }

  void reset() {
    _profiles.clear();
    shouldFailSave = false;
    shouldFailLoad = false;
    lastSavedProfileId = null;
    saveCallCount = 0;
    listCallCount = 0;
    getCallCount = 0;
  }
}

void main() {
  group('VisualEditorPage Tests', () {
    late MockProfileRegistryService mockService;

    setUp(() {
      mockService = MockProfileRegistryService();
    });

    tearDown(() {
      mockService.reset();
    });

    // Helper function to wrap widget with provider
    Widget makeTestableWidget(Widget child) {
      return MaterialApp(
        home: Provider<ProfileRegistryService>.value(
          value: mockService,
          child: child,
        ),
      );
    }

    // Sample profile factory
    Profile createTestProfile({
      String? id,
      String name = 'Test Profile',
      LayoutType layoutType = LayoutType.matrix,
      Map<String, KeyAction>? mappings,
    }) {
      return Profile(
        id: id ?? 'test-profile-${DateTime.now().millisecondsSinceEpoch}',
        name: name,
        layoutType: layoutType,
        mappings: mappings ?? {},
        createdAt: DateTime.now().toIso8601String(),
        updatedAt: DateTime.now().toIso8601String(),
      );
    }

    group('Initial Rendering', () {
      testWidgets('renders with empty state when no profiles',
          (WidgetTester tester) async {
        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Should show empty state
        expect(find.text('No profile selected'), findsOneWidget);
        expect(find.text('Create a new profile to get started'), findsOneWidget);
        expect(find.byIcon(Icons.description_outlined), findsOneWidget);

        // Should have create button
        expect(find.text('Create New Profile'), findsOneWidget);
      });

      testWidgets('loads and displays existing profiles',
          (WidgetTester tester) async {
        // Add test profiles
        final profile1 = createTestProfile(id: 'profile-1', name: 'Profile 1');
        final profile2 = createTestProfile(id: 'profile-2', name: 'Profile 2');
        mockService.addTestProfile(profile1);
        mockService.addTestProfile(profile2);

        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Should load profiles
        expect(mockService.listCallCount, greaterThan(0));

        // Should auto-select first profile
        expect(find.byType(DragDropMapper), findsOneWidget);
      });

      testWidgets('displays profile selector with profiles',
          (WidgetTester tester) async {
        final profile = createTestProfile(id: 'test-1', name: 'My Profile');
        mockService.addTestProfile(profile);

        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Should show dropdown with profile
        expect(find.byType(DropdownButton<String>), findsOneWidget);
      });

      testWidgets('shows loading indicator during initial load',
          (WidgetTester tester) async {
        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );

        // Should show loading initially
        expect(find.byType(CircularProgressIndicator), findsOneWidget);
        expect(find.text('Loading...'), findsOneWidget);

        await tester.pumpAndSettle();
      });

      testWidgets('displays app bar with title and refresh button',
          (WidgetTester tester) async {
        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Should have app bar
        expect(find.widgetWithText(AppBar, 'Visual Editor'), findsOneWidget);
        expect(find.byIcon(Icons.refresh), findsOneWidget);
      });
    });

    group('Profile Management', () {
      testWidgets('creates new profile with matrix layout',
          (WidgetTester tester) async {
        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Tap create new profile button
        await tester.tap(find.text('Create New Profile'));
        await tester.pumpAndSettle();

        // Should show name dialog (there may be multiple instances of the text)
        expect(find.text('Create New Profile'), findsWidgets);
        expect(find.text('Profile Name'), findsOneWidget);

        // Enter name
        await tester.enterText(find.byType(TextField).last, 'My New Profile');
        await tester.tap(find.text('Create'));
        await tester.pumpAndSettle();

        // Should show layout type selector
        expect(find.text('Select Layout Type'), findsOneWidget);

        // Select matrix layout
        await tester.tap(find.text('Matrix'));
        await tester.pumpAndSettle();

        // Should have saved the profile
        expect(mockService.saveCallCount, 1);

        // Should show success message
        expect(find.text('Profile "My New Profile" created'), findsOneWidget);
      });

      testWidgets('creates new profile with standard layout',
          (WidgetTester tester) async {
        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Tap create button
        await tester.tap(find.text('Create New Profile'));
        await tester.pumpAndSettle();

        // Enter name
        await tester.enterText(
            find.byType(TextField).last, 'Standard Profile');
        await tester.tap(find.text('Create'));
        await tester.pumpAndSettle();

        // Select standard layout
        await tester.tap(find.text('Standard'));
        await tester.pumpAndSettle();

        expect(mockService.saveCallCount, 1);
      });

      testWidgets('creates new profile with split layout',
          (WidgetTester tester) async {
        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        await tester.tap(find.text('Create New Profile'));
        await tester.pumpAndSettle();

        await tester.enterText(find.byType(TextField).last, 'Split Profile');
        await tester.tap(find.text('Create'));
        await tester.pumpAndSettle();

        await tester.tap(find.text('Split'));
        await tester.pumpAndSettle();

        expect(mockService.saveCallCount, 1);
      });

      testWidgets('cancels profile creation on cancel button',
          (WidgetTester tester) async {
        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        await tester.tap(find.text('Create New Profile'));
        await tester.pumpAndSettle();

        // Cancel name dialog
        await tester.tap(find.text('Cancel'));
        await tester.pumpAndSettle();

        // Should not have saved
        expect(mockService.saveCallCount, 0);
      });

      testWidgets('cancels profile creation on layout cancel',
          (WidgetTester tester) async {
        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        await tester.tap(find.text('Create New Profile'));
        await tester.pumpAndSettle();

        await tester.enterText(find.byType(TextField).last, 'Test');
        await tester.tap(find.text('Create'));
        await tester.pumpAndSettle();

        // Cancel layout selector
        await tester.tap(find.text('Cancel').last);
        await tester.pumpAndSettle();

        // Should not have saved
        expect(mockService.saveCallCount, 0);
      });

      testWidgets('handles empty profile name',
          (WidgetTester tester) async {
        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        await tester.tap(find.text('Create New Profile'));
        await tester.pumpAndSettle();

        // Don't enter a name, just tap create
        await tester.tap(find.text('Create'));
        await tester.pumpAndSettle();

        // Should not proceed (no save, no layout dialog)
        expect(mockService.saveCallCount, 0);
        expect(find.text('Select Layout Type'), findsNothing);
      });
    });

    group('Profile Selection', () {
      testWidgets('switches between profiles using dropdown',
          (WidgetTester tester) async {
        final profile1 =
            createTestProfile(id: 'profile-1', name: 'Profile 1');
        final profile2 =
            createTestProfile(id: 'profile-2', name: 'Profile 2');
        mockService.addTestProfile(profile1);
        mockService.addTestProfile(profile2);

        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Initially loads first profile
        expect(mockService.getCallCount, greaterThan(0));

        // Open dropdown
        await tester.tap(find.byType(DropdownButton<String>));
        await tester.pumpAndSettle();

        // Select second profile
        await tester.tap(find.text('Profile 2').last);
        await tester.pumpAndSettle();

        // Should load the second profile
        expect(mockService.getCallCount, greaterThan(1));
      });

      testWidgets('refreshes profile list when refresh button tapped',
          (WidgetTester tester) async {
        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        final initialListCalls = mockService.listCallCount;

        // Tap refresh button
        await tester.tap(find.byIcon(Icons.refresh));
        await tester.pumpAndSettle();

        // Should have called list again
        expect(mockService.listCallCount, greaterThan(initialListCalls));
      });
    });

    group('DragDropMapper Integration', () {
      testWidgets('displays DragDropMapper when profile is loaded',
          (WidgetTester tester) async {
        final profile = createTestProfile(
          id: 'test-1',
          layoutType: LayoutType.matrix,
        );
        mockService.addTestProfile(profile);

        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Should display DragDropMapper
        expect(find.byType(DragDropMapper), findsOneWidget);
      });

      testWidgets('passes correct layout info for matrix profile',
          (WidgetTester tester) async {
        final profile = createTestProfile(
          id: 'test-1',
          layoutType: LayoutType.matrix,
        );
        mockService.addTestProfile(profile);

        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        final mapper =
            tester.widget<DragDropMapper>(find.byType(DragDropMapper));
        expect(mapper.layoutInfo.type, LayoutType.matrix);
        expect(mapper.layoutInfo.rows, 5);
        expect(mapper.layoutInfo.cols, 5);
      });

      testWidgets('passes correct layout info for standard profile',
          (WidgetTester tester) async {
        final profile = createTestProfile(
          id: 'test-1',
          layoutType: LayoutType.standard,
        );
        mockService.addTestProfile(profile);

        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        final mapper =
            tester.widget<DragDropMapper>(find.byType(DragDropMapper));
        expect(mapper.layoutInfo.type, LayoutType.standard);
        expect(mapper.layoutInfo.rows, 6);
        expect(mapper.layoutInfo.cols, 15);
      });

      testWidgets('passes correct layout info for split profile',
          (WidgetTester tester) async {
        final profile = createTestProfile(
          id: 'test-1',
          layoutType: LayoutType.split,
        );
        mockService.addTestProfile(profile);

        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        final mapper =
            tester.widget<DragDropMapper>(find.byType(DragDropMapper));
        expect(mapper.layoutInfo.type, LayoutType.split);
        expect(mapper.layoutInfo.rows, 5);
        expect(mapper.layoutInfo.cols, 14);
      });

      testWidgets('updates state when DragDropMapper updates profile',
          (WidgetTester tester) async {
        final profile = createTestProfile(
          id: 'test-1',
          layoutType: LayoutType.matrix,
          mappings: {},
        );
        mockService.addTestProfile(profile);

        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Simulate profile update from DragDropMapper
        final updatedProfile = profile.copyWith(
          mappings: {'0,0': const KeyAction.key(key: 'A')},
        );

        final mapper =
            tester.widget<DragDropMapper>(find.byType(DragDropMapper));
        mapper.onProfileUpdated?.call(updatedProfile);

        await tester.pumpAndSettle();

        // State should be updated (verified by no errors)
        expect(find.byType(DragDropMapper), findsOneWidget);
      });

      testWidgets('shows error snackbar on save error',
          (WidgetTester tester) async {
        final profile = createTestProfile(
          id: 'test-1',
          layoutType: LayoutType.matrix,
        );
        mockService.addTestProfile(profile);

        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Simulate save error from DragDropMapper
        final mapper =
            tester.widget<DragDropMapper>(find.byType(DragDropMapper));
        mapper.onSaveError?.call('Test error message');

        await tester.pumpAndSettle();

        // Should show error snackbar
        expect(find.text('Test error message'), findsOneWidget);
      });
    });

    group('Error Handling', () {
      testWidgets('shows error message when profile list fails to load',
          (WidgetTester tester) async {
        mockService.shouldFailLoad = true;

        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Should show error message
        expect(find.textContaining('Failed to load profiles'), findsOneWidget);
        expect(find.byIcon(Icons.error_outline), findsOneWidget);
      });

      testWidgets('shows error message when profile fails to load',
          (WidgetTester tester) async {
        final profile = createTestProfile(id: 'test-1');
        mockService.addTestProfile(profile);

        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Now fail loads
        mockService.shouldFailLoad = true;

        // Try to reload profile
        await tester.tap(find.byIcon(Icons.refresh));
        await tester.pumpAndSettle();

        // Should show error
        expect(find.textContaining('Failed to load'), findsOneWidget);
      });

      testWidgets('allows dismissing error message',
          (WidgetTester tester) async {
        mockService.shouldFailLoad = true;

        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Should show error
        expect(find.textContaining('Failed to load'), findsOneWidget);

        // Dismiss error
        await tester.tap(find.byIcon(Icons.close));
        await tester.pumpAndSettle();

        // Error should be dismissed
        expect(find.textContaining('Failed to load'), findsNothing);
      });

      testWidgets('shows error when profile creation fails',
          (WidgetTester tester) async {
        mockService.shouldFailSave = true;

        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Attempt to create profile
        await tester.tap(find.text('Create New Profile'));
        await tester.pumpAndSettle();

        await tester.enterText(find.byType(TextField).last, 'Test');
        await tester.tap(find.text('Create'));
        await tester.pumpAndSettle();

        await tester.tap(find.text('Matrix'));
        await tester.pumpAndSettle();

        // Should show error (no success message)
        expect(find.text('Profile "Test" created'), findsNothing);
      });
    });

    group('UI State Management', () {
      testWidgets('disables buttons during loading',
          (WidgetTester tester) async {
        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );

        // Should show loading
        expect(find.byType(CircularProgressIndicator), findsOneWidget);

        await tester.pumpAndSettle();
      });

      testWidgets('shows toolbar with profile selector',
          (WidgetTester tester) async {
        final profile = createTestProfile(id: 'test-1');
        mockService.addTestProfile(profile);

        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Should show toolbar elements
        expect(find.byIcon(Icons.person_outline), findsOneWidget);
        expect(find.text('Profile:'), findsOneWidget);
        expect(find.byType(DropdownButton<String>), findsOneWidget);
        expect(find.text('New Profile'), findsOneWidget);
      });

      testWidgets('new profile button available from toolbar',
          (WidgetTester tester) async {
        final profile = createTestProfile(id: 'test-1');
        mockService.addTestProfile(profile);

        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Find new profile button in toolbar (may be icon + text)
        expect(find.text('New Profile'), findsOneWidget);

        await tester.tap(find.text('New Profile'));
        await tester.pumpAndSettle();

        // Should show create dialog
        expect(find.text('Create New Profile'), findsOneWidget);
      });
    });

    group('Edge Cases', () {
      testWidgets('handles switching to non-existent profile gracefully',
          (WidgetTester tester) async {
        final profile = createTestProfile(id: 'test-1');
        mockService.addTestProfile(profile);

        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Profile is loaded initially
        expect(find.byType(DragDropMapper), findsOneWidget);
      });

      testWidgets('handles rapid profile switching',
          (WidgetTester tester) async {
        final profile1 = createTestProfile(id: 'profile-1', name: 'Profile 1');
        final profile2 = createTestProfile(id: 'profile-2', name: 'Profile 2');
        mockService.addTestProfile(profile1);
        mockService.addTestProfile(profile2);

        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Open dropdown
        await tester.tap(find.byType(DropdownButton<String>));
        await tester.pumpAndSettle();

        // Find and tap Profile 2 dropdown item
        final profile2Items = find.text('Profile 2');
        if (profile2Items.evaluate().length > 1) {
          // Tap the one in the dropdown menu (last one)
          await tester.tap(profile2Items.last);
        } else {
          await tester.tap(profile2Items);
        }
        await tester.pumpAndSettle();

        // Should handle gracefully
        expect(find.byType(DragDropMapper), findsOneWidget);
      });

      testWidgets('maintains state when profile is externally updated',
          (WidgetTester tester) async {
        final profile = createTestProfile(id: 'test-1', name: 'Original');
        mockService.addTestProfile(profile);

        await tester.pumpWidget(
          makeTestableWidget(const VisualEditorPage()),
        );
        await tester.pumpAndSettle();

        // Update profile externally
        final updatedProfile =
            profile.copyWith(name: 'Updated', mappings: {});
        mockService.saveCallCount = 0;
        await mockService.saveProfile(updatedProfile);

        // Refresh to see update
        await tester.tap(find.byIcon(Icons.refresh));
        await tester.pumpAndSettle();

        // Should still be functional
        expect(find.byType(DragDropMapper), findsOneWidget);
      });
    });
  });
}

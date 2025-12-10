import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/profile.dart';
import 'package:keyrx_ui/models/layout_type.dart';
import 'package:keyrx_ui/services/profile_registry_service.dart';
import 'package:keyrx_ui/widgets/profile_selector.dart';

/// Mock ProfileRegistryService for testing.
class MockProfileRegistryService implements ProfileRegistryService {
  final List<String> _profiles;
  final bool _shouldThrow;

  MockProfileRegistryService({List<String>? profiles, bool shouldThrow = false})
    : _profiles = profiles ?? [],
      _shouldThrow = shouldThrow;

  @override
  Future<List<String>> listProfiles() async {
    if (_shouldThrow) {
      throw Exception('Failed to load profiles');
    }
    return _profiles;
  }

  @override
  Future<Profile?> getProfile(String profileId) async {
    return null;
  }

  @override
  Future<ProfileRegistryOperationResult> saveProfile(Profile profile) async {
    return ProfileRegistryOperationResult.success();
  }

  @override
  Future<ProfileRegistryOperationResult> deleteProfile(String profileId) async {
    return ProfileRegistryOperationResult.success();
  }

  @override
  Future<List<Profile>> findCompatibleProfiles(LayoutType layoutType) async {
    return const [];
  }

  @override
  Future<List<String>> refresh() async {
    return _profiles;
  }

  @override
  Future<void> dispose() async {}
}

void main() {
  group('ProfileSelector', () {
    testWidgets('displays loading state initially', (
      WidgetTester tester,
    ) async {
      final service = MockProfileRegistryService(profiles: ['profile-1']);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: ProfileSelector(profileService: service)),
        ),
      );

      // Before pump, should show loading
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      expect(find.text('Loading profiles...'), findsOneWidget);
    });

    testWidgets('displays dropdown with profiles after loading', (
      WidgetTester tester,
    ) async {
      final service = MockProfileRegistryService(
        profiles: ['profile-1', 'profile-2', 'profile-3'],
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: ProfileSelector(profileService: service)),
        ),
      );

      // Wait for loading to complete
      await tester.pumpAndSettle();

      // Should display dropdown
      expect(find.byType(DropdownButton<String?>), findsOneWidget);
      expect(find.byType(CircularProgressIndicator), findsNothing);
    });

    testWidgets('dropdown includes "No Profile" option', (
      WidgetTester tester,
    ) async {
      final service = MockProfileRegistryService(profiles: ['profile-1']);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: ProfileSelector(profileService: service)),
        ),
      );

      await tester.pumpAndSettle();

      // Tap to open dropdown
      await tester.tap(find.byType(DropdownButton<String?>));
      await tester.pumpAndSettle();

      // Should have "No Profile" option
      expect(find.text('No Profile'), findsOneWidget);
    });

    testWidgets('dropdown includes all profile IDs', (
      WidgetTester tester,
    ) async {
      final service = MockProfileRegistryService(
        profiles: ['alpha', 'beta', 'gamma'],
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: ProfileSelector(profileService: service)),
        ),
      );

      await tester.pumpAndSettle();

      // Tap to open dropdown
      await tester.tap(find.byType(DropdownButton<String?>));
      await tester.pump(); // Trigger the overlay to appear
      await tester.pump(const Duration(seconds: 1)); // Wait for animation

      // All profiles should be present in the overlay
      expect(find.text('alpha', skipOffstage: false), findsWidgets);
      expect(find.text('beta', skipOffstage: false), findsWidgets);
      expect(find.text('gamma', skipOffstage: false), findsWidgets);
    });

    testWidgets('shows selected profile', (WidgetTester tester) async {
      final service = MockProfileRegistryService(
        profiles: ['profile-1', 'profile-2'],
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ProfileSelector(
              profileService: service,
              selectedProfileId: 'profile-2',
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final dropdown = tester.widget<DropdownButton<String?>>(
        find.byType(DropdownButton<String?>),
      );
      expect(dropdown.value, 'profile-2');
    });

    testWidgets('calls onChanged when selection changes', (
      WidgetTester tester,
    ) async {
      final service = MockProfileRegistryService(
        profiles: ['profile-1', 'profile-2'],
      );

      String? selectedProfile;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ProfileSelector(
              profileService: service,
              onChanged: (value) => selectedProfile = value,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Tap to open dropdown
      await tester.tap(find.byType(DropdownButton<String?>));
      await tester.pumpAndSettle();

      // Select a profile
      await tester.tap(find.text('profile-1').last);
      await tester.pumpAndSettle();

      expect(selectedProfile, 'profile-1');
    });

    testWidgets('calls onChanged with null when "No Profile" selected', (
      WidgetTester tester,
    ) async {
      final service = MockProfileRegistryService(profiles: ['profile-1']);

      String? selectedProfile = 'profile-1';

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ProfileSelector(
              profileService: service,
              selectedProfileId: selectedProfile,
              onChanged: (value) => selectedProfile = value,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Tap to open dropdown
      await tester.tap(find.byType(DropdownButton<String?>));
      await tester.pumpAndSettle();

      // Select "No Profile"
      await tester.tap(find.text('No Profile').last);
      await tester.pumpAndSettle();

      expect(selectedProfile, isNull);
    });

    testWidgets('displays error state when loading fails', (
      WidgetTester tester,
    ) async {
      final service = MockProfileRegistryService(shouldThrow: true);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: ProfileSelector(profileService: service)),
        ),
      );

      await tester.pumpAndSettle();

      // Should show error message
      expect(find.text('Failed to load profiles'), findsOneWidget);
      expect(find.byIcon(Icons.error_outline), findsOneWidget);
      expect(find.byType(DropdownButton<String?>), findsNothing);
    });

    testWidgets('displays empty state when no profiles available', (
      WidgetTester tester,
    ) async {
      final service = MockProfileRegistryService(profiles: []);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: ProfileSelector(profileService: service)),
        ),
      );

      await tester.pumpAndSettle();

      // Should show empty message
      expect(find.text('No profiles available'), findsOneWidget);
      expect(find.byIcon(Icons.info_outline), findsOneWidget);
      expect(find.byType(DropdownButton<String?>), findsNothing);
    });

    testWidgets('disables dropdown when enabled is false', (
      WidgetTester tester,
    ) async {
      final service = MockProfileRegistryService(profiles: ['profile-1']);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ProfileSelector(profileService: service, enabled: false),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final dropdown = tester.widget<DropdownButton<String?>>(
        find.byType(DropdownButton<String?>),
      );
      expect(dropdown.onChanged, isNull);
    });

    testWidgets('enables dropdown when enabled is true', (
      WidgetTester tester,
    ) async {
      final service = MockProfileRegistryService(profiles: ['profile-1']);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ProfileSelector(
              profileService: service,
              enabled: true,
              onChanged: (_) {},
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final dropdown = tester.widget<DropdownButton<String?>>(
        find.byType(DropdownButton<String?>),
      );
      expect(dropdown.onChanged, isNotNull);
    });
  });
}

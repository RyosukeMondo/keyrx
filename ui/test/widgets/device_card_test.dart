import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/device_identity.dart';
import 'package:keyrx_ui/models/device_state.dart';
import 'package:keyrx_ui/models/profile.dart';
import 'package:keyrx_ui/models/layout_type.dart';
import 'package:keyrx_ui/services/device_registry_service.dart';
import 'package:keyrx_ui/services/profile_registry_service.dart';
import 'package:keyrx_ui/widgets/device_card.dart';
import 'package:keyrx_ui/widgets/remap_toggle.dart';
import 'package:keyrx_ui/widgets/profile_selector.dart';

/// Mock DeviceRegistryService for testing.
class MockDeviceRegistryService implements DeviceRegistryService {
  final List<DeviceState> _devices;
  final bool _shouldFail;
  String? lastToggledDeviceKey;
  bool? lastToggledValue;
  String? lastAssignedDeviceKey;
  String? lastAssignedProfileId;

  MockDeviceRegistryService({
    List<DeviceState>? devices,
    bool shouldFail = false,
  })  : _devices = devices ?? [],
        _shouldFail = shouldFail;

  @override
  Future<List<DeviceState>> getDevices() async {
    return _devices;
  }

  @override
  Future<DeviceRegistryOperationResult> toggleRemap(
    String deviceKey,
    bool enabled,
  ) async {
    lastToggledDeviceKey = deviceKey;
    lastToggledValue = enabled;
    if (_shouldFail) {
      return DeviceRegistryOperationResult.error('Failed to toggle remap');
    }
    return DeviceRegistryOperationResult.success();
  }

  @override
  Future<DeviceRegistryOperationResult> assignProfile(
    String deviceKey,
    String profileId,
  ) async {
    lastAssignedDeviceKey = deviceKey;
    lastAssignedProfileId = profileId;
    if (_shouldFail) {
      return DeviceRegistryOperationResult.error('Failed to assign profile');
    }
    return DeviceRegistryOperationResult.success();
  }

  @override
  Future<DeviceRegistryOperationResult> setUserLabel(
    String deviceKey,
    String? label,
  ) async {
    if (_shouldFail) {
      return DeviceRegistryOperationResult.error('Failed to set label');
    }
    return DeviceRegistryOperationResult.success();
  }

  @override
  Future<List<DeviceState>> refresh() async {
    return _devices;
  }

  @override
  Future<void> dispose() async {}
}

/// Mock ProfileRegistryService for testing.
class MockProfileRegistryService implements ProfileRegistryService {
  final List<String> _profiles;

  MockProfileRegistryService({
    List<String>? profiles,
  }) : _profiles = profiles ?? [];

  @override
  Future<List<String>> listProfiles() async {
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
  group('DeviceCard', () {
    late DeviceState deviceState;
    late MockDeviceRegistryService deviceService;
    late MockProfileRegistryService profileService;

    setUp(() {
      deviceState = const DeviceState(
        identity: DeviceIdentity(
          vendorId: 0x046d,
          productId: 0xc52b,
          serialNumber: 'ABC123',
        ),
        remapEnabled: true,
        profileId: 'test-profile',
        connectedAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      );
      deviceService = MockDeviceRegistryService();
      profileService = MockProfileRegistryService(
        profiles: ['test-profile', 'another-profile'],
      );
    });

    testWidgets('displays device display name', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: deviceState,
              deviceService: deviceService,
              profileService: profileService,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Should display VID:PID since no user label
      expect(find.text('046d:c52b'), findsOneWidget);
    });

    testWidgets('displays user label when set', (WidgetTester tester) async {
      final deviceWithLabel = DeviceState(
        identity: const DeviceIdentity(
          vendorId: 0x046d,
          productId: 0xc52b,
          serialNumber: 'ABC123',
          userLabel: 'My Keyboard',
        ),
        remapEnabled: true,
        profileId: 'test-profile',
        connectedAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: deviceWithLabel,
              deviceService: deviceService,
              profileService: profileService,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.text('My Keyboard'), findsOneWidget);
    });

    testWidgets('displays device identity key', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: deviceState,
              deviceService: deviceService,
              profileService: profileService,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.text('046d:c52b:ABC123'), findsOneWidget);
    });

    testWidgets('displays Active status when remap enabled and profile assigned',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: deviceState,
              deviceService: deviceService,
              profileService: profileService,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.text('Active'), findsOneWidget);
    });

    testWidgets('displays Passthrough status when remap disabled',
        (WidgetTester tester) async {
      final passthroughDevice = deviceState.copyWith(remapEnabled: false);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: passthroughDevice,
              deviceService: deviceService,
              profileService: profileService,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.text('Passthrough'), findsOneWidget);
    });

    testWidgets('displays No Profile status when no profile assigned',
        (WidgetTester tester) async {
      final noProfileDevice = deviceState.copyWith(
        profileId: null,
        remapEnabled: true,
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: noProfileDevice,
              deviceService: deviceService,
              profileService: profileService,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // "No Profile" appears in both the status badge and the dropdown
      expect(find.text('No Profile'), findsWidgets);
    });

    testWidgets('includes RemapToggle widget', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: deviceState,
              deviceService: deviceService,
              profileService: profileService,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.byType(RemapToggle), findsOneWidget);
    });

    testWidgets('includes ProfileSelector widget',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: deviceState,
              deviceService: deviceService,
              profileService: profileService,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.byType(ProfileSelector), findsOneWidget);
    });

    testWidgets('calls deviceService.toggleRemap when toggle is changed',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: deviceState,
              deviceService: deviceService,
              profileService: profileService,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Tap the toggle
      await tester.tap(find.byType(Switch));
      await tester.pumpAndSettle();

      expect(deviceService.lastToggledDeviceKey, '046d:c52b:ABC123');
      expect(deviceService.lastToggledValue, false);
    });

    testWidgets('calls deviceService.assignProfile when profile is selected',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: deviceState,
              deviceService: deviceService,
              profileService: profileService,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Tap the dropdown to open it
      await tester.tap(find.byType(DropdownButton<String?>));
      await tester.pumpAndSettle();

      // Select another profile
      await tester.tap(find.text('another-profile').last);
      await tester.pumpAndSettle();

      expect(deviceService.lastAssignedDeviceKey, '046d:c52b:ABC123');
      expect(deviceService.lastAssignedProfileId, 'another-profile');
    });

    testWidgets('displays Edit Label button', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: deviceState,
              deviceService: deviceService,
              profileService: profileService,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.text('Edit Label'), findsOneWidget);
    });

    testWidgets('displays Manage Profiles button',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: deviceState,
              deviceService: deviceService,
              profileService: profileService,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.text('Manage Profiles'), findsOneWidget);
    });

    testWidgets('calls onEditLabel when Edit Label button is pressed',
        (WidgetTester tester) async {
      bool editLabelCalled = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: deviceState,
              deviceService: deviceService,
              profileService: profileService,
              onEditLabel: () => editLabelCalled = true,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      await tester.tap(find.text('Edit Label'));
      await tester.pumpAndSettle();

      expect(editLabelCalled, isTrue);
    });

    testWidgets('calls onManageProfiles when Manage Profiles button is pressed',
        (WidgetTester tester) async {
      bool manageProfilesCalled = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: deviceState,
              deviceService: deviceService,
              profileService: profileService,
              onManageProfiles: () => manageProfilesCalled = true,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      await tester.tap(find.text('Manage Profiles'));
      await tester.pumpAndSettle();

      expect(manageProfilesCalled, isTrue);
    });

    testWidgets('displays as Card widget', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: deviceState,
              deviceService: deviceService,
              profileService: profileService,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.byType(Card), findsOneWidget);
    });

    testWidgets('RemapToggle reflects device remap state',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: deviceState,
              deviceService: deviceService,
              profileService: profileService,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final remapToggle = tester.widget<RemapToggle>(find.byType(RemapToggle));
      expect(remapToggle.enabled, isTrue);
    });

    testWidgets('ProfileSelector reflects device profile assignment',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: deviceState,
              deviceService: deviceService,
              profileService: profileService,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final profileSelector =
          tester.widget<ProfileSelector>(find.byType(ProfileSelector));
      expect(profileSelector.selectedProfileId, 'test-profile');
    });

    testWidgets('rolls back remap toggle on failure and shows error snackbar',
        (WidgetTester tester) async {
      deviceService = MockDeviceRegistryService(shouldFail: true);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: deviceState,
              deviceService: deviceService,
              profileService: profileService,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      await tester.tap(find.byType(Switch));
      await tester.pumpAndSettle();

      final remapToggle = tester.widget<RemapToggle>(find.byType(RemapToggle));
      expect(remapToggle.enabled, isTrue);
      expect(find.textContaining('Failed to toggle remap'), findsOneWidget);
    });

    testWidgets('rolls back profile selection on failure and shows error snackbar',
        (WidgetTester tester) async {
      deviceService = MockDeviceRegistryService(shouldFail: true);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DeviceCard(
              deviceState: deviceState,
              deviceService: deviceService,
              profileService: profileService,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      await tester.tap(find.byType(DropdownButton<String?>));
      await tester.pumpAndSettle();

      await tester.tap(find.text('another-profile').last);
      await tester.pumpAndSettle();

      final profileSelector =
          tester.widget<ProfileSelector>(find.byType(ProfileSelector));
      expect(profileSelector.selectedProfileId, 'test-profile');
      expect(find.textContaining('Failed to assign profile'), findsOneWidget);
    });
  });
}

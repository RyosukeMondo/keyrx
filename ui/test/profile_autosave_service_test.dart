import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/layout_type.dart';
import 'package:keyrx_ui/models/profile.dart';
import 'package:keyrx_ui/services/profile_autosave_service.dart';
import 'package:keyrx_ui/services/profile_registry_service.dart';
import 'package:keyrx_ui/services/storage_path_resolver.dart';

class _StubProfileRegistryService implements ProfileRegistryService {
  @override
  Future<void> dispose() async {}

  @override
  Future<List<Profile>> findCompatibleProfiles(LayoutType layoutType) async =>
      const <Profile>[];

  @override
  Future<Profile?> getProfile(String profileId) async => null;

  @override
  Future<List<String>> listProfiles() async => const <String>[];

  @override
  Future<List<String>> refresh() async => const <String>[];

  @override
  Future<ProfileRegistryOperationResult> deleteProfile(String profileId) async =>
      const ProfileRegistryOperationResult(success: true);

  @override
  Future<ProfileRegistryOperationResult> saveProfile(Profile profile) async =>
      const ProfileRegistryOperationResult(success: true);
}

class _FakeStoragePathResolver extends StoragePathResolver {
  _FakeStoragePathResolver(this.path) : super(environment: const {});

  final String path;

  @override
  Future<String> ensureProfilesDirectory() async => path;
}

Profile _profile(String id) {
  const timestamp = '2024-01-01T00:00:00Z';
  return Profile(
    id: id,
    name: 'Profile $id',
    layoutType: LayoutType.standard,
    createdAt: timestamp,
    updatedAt: timestamp,
  );
}

void main() {
  group('ProfileAutosaveService', () {
    test('debounces multiple saves and keeps the latest profile', () {
      fakeAsync((async) {
        final savedProfiles = <Profile>[];
        final statuses = <AutosaveStatus>[];
        final service = ProfileAutosaveService(
          profileRegistryService: _StubProfileRegistryService(),
          storagePathResolver: _FakeStoragePathResolver('/tmp/.keyrx'),
          debounceDuration: const Duration(milliseconds: 100),
          initialBackoff: const Duration(milliseconds: 10),
          maxRetries: 1,
          delayFn: (duration) {
            async.elapse(duration);
            return Future.value();
          },
          now: () => DateTime(2024, 1, 1, 12, 0, 0),
          saveOperation: (profile) async {
            savedProfiles.add(profile);
            return const ProfileRegistryOperationResult(success: true);
          },
        );

        final subscription = service.statusStream.listen(statuses.add);
        addTearDown(() async {
          await subscription.cancel();
          await service.dispose();
        });

        final first = _profile('first');
        final latest = _profile('latest');

        service.queueSave(first);
        async.elapse(const Duration(milliseconds: 50));
        service.queueSave(latest);

        async.elapse(const Duration(milliseconds: 100));
        async.flushMicrotasks();

        expect(savedProfiles.length, 1);
        expect(savedProfiles.single.id, 'latest');

        final savingAttempts = statuses
            .where((status) => status.state == AutosaveState.saving)
            .toList();
        expect(savingAttempts.length, 1);
        expect(savingAttempts.single.profileId, 'latest');

        final successStatuses = statuses
            .where((status) => status.state == AutosaveState.success)
            .toList();
        expect(successStatuses.length, 1);
        expect(successStatuses.single.profileId, 'latest');
      });
    });

    test('retries transient errors with backoff and succeeds', () {
      fakeAsync((async) {
        final attempts = <int>[];
        final observedBackoffs = <Duration>[];
        final statuses = <AutosaveStatus>[];
        final resolver = _FakeStoragePathResolver('/tmp/.keyrx');

        final errors = <String>[
          'IO error: write failed',
          'temporary unavailable',
        ];

        final service = ProfileAutosaveService(
          profileRegistryService: _StubProfileRegistryService(),
          storagePathResolver: resolver,
          debounceDuration: const Duration(milliseconds: 20),
          initialBackoff: const Duration(milliseconds: 50),
          maxRetries: 2,
          delayFn: (duration) {
            observedBackoffs.add(duration);
            return Future.value();
          },
          now: () => DateTime(2024, 2, 1, 10, 0, 0),
          saveOperation: (profile) async {
            attempts.add(attempts.length + 1);
            if (errors.isNotEmpty) {
              return ProfileRegistryOperationResult.error(errors.removeAt(0));
            }
            return const ProfileRegistryOperationResult(success: true);
          },
        );

        final subscription = service.statusStream.listen(statuses.add);
        addTearDown(() async {
          await subscription.cancel();
          await service.dispose();
        });
        final profile = _profile('retry-me');

        service.queueSave(profile);

        async.elapse(const Duration(milliseconds: 20));
        async.flushMicrotasks();

        final savingAttempts = statuses
            .where((status) => status.state == AutosaveState.saving)
            .map((status) => status.attempt)
            .toList();

        expect(savingAttempts, [1, 2, 3]);
        expect(observedBackoffs, [
          const Duration(milliseconds: 50),
          const Duration(milliseconds: 100),
        ]);
        expect(attempts, [1, 2, 3]);

        final successStatuses = statuses
            .where((status) => status.state == AutosaveState.success)
            .toList();
        expect(successStatuses.length, 1);
        expect(successStatuses.single.profileId, 'retry-me');
        expect(successStatuses.single.targetDirectory, resolver.path);
      });
    });
  });
}

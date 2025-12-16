/// Unit tests for daemon Riverpod providers.
library;

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:keyrx_ui/services/daemon_service.dart';
import 'package:keyrx_ui/state/daemon_providers.dart';

void main() {
  group('Daemon Providers', () {
    test('daemonServiceProvider creates singleton instance', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final service1 = container.read(daemonServiceProvider);
      final service2 = container.read(daemonServiceProvider);

      expect(service1, same(service2));
      expect(service1, isA<DaemonService>());
    });

    test(
      'daemonServiceProvider disposes service on container dispose',
      () async {
        final container = ProviderContainer();

        final service = container.read(daemonServiceProvider);
        expect(service, isNotNull);

        // Dispose container
        container.dispose();

        // Service should be disposed (state = disconnected)
        expect(service.state, DaemonConnectionState.disconnected);
      },
    );

    test('daemonConnectionStateProvider provides stream', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final stateAsync = container.read(daemonConnectionStateProvider);

      expect(stateAsync, isA<AsyncValue<DaemonConnectionState>>());
    });

    test('daemonEventsProvider provides stream', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final eventsAsync = container.read(daemonEventsProvider);

      expect(eventsAsync, isA<AsyncValue>());
    });

    test('multiple containers create independent service instances', () {
      final container1 = ProviderContainer();
      final container2 = ProviderContainer();

      addTearDown(() {
        container1.dispose();
        container2.dispose();
      });

      final service1 = container1.read(daemonServiceProvider);
      final service2 = container2.read(daemonServiceProvider);

      expect(service1, isNot(same(service2)));
    });

    test('provider dependencies are correctly wired', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      // Reading stream providers should not throw
      expect(
        () => container.read(daemonConnectionStateProvider),
        returnsNormally,
      );

      expect(() => container.read(daemonEventsProvider), returnsNormally);
    });
  });
}

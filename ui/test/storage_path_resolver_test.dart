import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/services/storage_path_resolver.dart';
import 'package:path/path.dart' as p;

void main() {
  group('StoragePathResolver', () {
    test('resolves linux home path', () async {
      final tempHome = await Directory.systemTemp.createTemp(
        'keyrx_home_linux',
      );
      addTearDown(() async => tempHome.delete(recursive: true));

      final resolver = StoragePathResolver(
        environment: {'HOME': tempHome.path, 'USERPROFILE': '/should/not/use'},
      );

      final resolved = resolver.resolveProfilesPath();

      expect(resolved, p.join(tempHome.path, '.keyrx'));
    }, testOn: 'linux');

    test('resolves windows home path', () async {
      final tempHome = await Directory.systemTemp.createTemp('keyrx_home_win');
      addTearDown(() async => tempHome.delete(recursive: true));
      final userProfile = p.join(tempHome.path, 'Users', 'tester');

      final resolver = StoragePathResolver(
        environment: {'USERPROFILE': userProfile, 'HOME': '/should/not/use'},
      );

      final resolved = resolver.resolveProfilesPath();

      expect(resolved, p.join(userProfile, '.keyrx'));
    }, testOn: 'windows');

    test('ensureProfilesDirectory creates directory', () async {
      final tempHome = await Directory.systemTemp.createTemp(
        'keyrx_home_create',
      );
      addTearDown(() async => tempHome.delete(recursive: true));

      final resolver = StoragePathResolver(
        environment: {'HOME': tempHome.path},
      );

      final resolved = await resolver.ensureProfilesDirectory();

      expect(resolved, p.join(tempHome.path, '.keyrx'));
      expect(await Directory(resolved).exists(), isTrue);
    });
  });
}

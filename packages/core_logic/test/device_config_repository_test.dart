import 'dart:io';
import 'package:core_logic/models/device_config.dart';
import 'package:core_logic/models/device_identity.dart';
import 'package:core_logic/repositories/device_config_repository.dart';
import 'package:path/path.dart' as p;
import 'package:test/test.dart';

void main() {
  late Directory tempDir;
  late DeviceConfigRepository repository;

  setUp(() {
    tempDir = Directory.systemTemp.createTempSync('device_config_test');
    repository = DeviceConfigRepository(configDir: tempDir.path);
  });

  tearDown(() {
    if (tempDir.existsSync()) {
      tempDir.deleteSync(recursive: true);
    }
  });

  group('DeviceConfigRepository', () {
    test('load() returns empty config if file does not exist', () async {
      final config = await repository.load();
      expect(config.devices, isEmpty);
    });

    test('save() creates file and load() retrieves it', () async {
      final config = DeviceConfig(
        devices: [
          const DeviceIdentity(
            vendorId: 0x1234,
            productId: 0x5678,
            serialNumber: 'SN123',
            userLabel: 'My Device',
          ),
        ],
      );

      await repository.save(config);
      
      final loaded = await repository.load();
      expect(loaded.devices, hasLength(1));
      expect(loaded.devices.first.vendorId, 0x1234);
      expect(loaded.devices.first.userLabel, 'My Device');
    });

    test('atomic write pattern works', () async {
      final config = DeviceConfig(
        devices: [
          const DeviceIdentity(
            vendorId: 0x1234,
            productId: 0x5678,
            serialNumber: 'SN123',
          ),
        ],
      );

      await repository.save(config);
      expect(File(repository.filePath).existsSync(), isTrue);
      expect(File('${repository.filePath}.tmp').existsSync(), isFalse);
    });

    test('load() handles corrupt JSON by creating backup', () async {
      final file = File(repository.filePath);
      await file.create(recursive: true);
      await file.writeAsString('{ invalid json }');

      final config = await repository.load();
      expect(config.devices, isEmpty);

      // Check for backup file
      final dir = Directory(tempDir.path);
      final files = dir.listSync();
      expect(files.any((f) => f.path.contains('.bak')), isTrue);
    });
    
    test('default path logic works (smoke test)', () {
      final repo = DeviceConfigRepository();
      expect(repo.filePath, contains('keyrx'));
      expect(repo.filePath, endsWith('devices.json'));
    });
  });
}

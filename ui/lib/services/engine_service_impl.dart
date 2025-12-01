import '../ffi/bridge.dart';
import 'engine_service.dart';

/// Real EngineService that wraps the KeyrxBridge.
class EngineServiceImpl implements EngineService {
  EngineServiceImpl({KeyrxBridge? bridge})
      : _bridge = bridge ?? KeyrxBridge.instance;

  final KeyrxBridge _bridge;

  @override
  bool get isInitialized => _bridge.isInitialized;

  @override
  String get version => _bridge.version;

  @override
  Future<bool> initialize() async {
    return _bridge.initialize();
  }

  @override
  Future<bool> loadScript(String path) async {
    if (!isInitialized) return false;
    return _bridge.loadScript(path);
  }

  @override
  Future<void> dispose() async {}
}

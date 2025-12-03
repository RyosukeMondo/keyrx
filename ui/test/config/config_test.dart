import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/config/config.dart';

void main() {
  group('TimingConfig', () {
    test('animationDurationMs has correct value', () {
      expect(TimingConfig.animationDurationMs, equals(150));
    });

    test('pulseAnimationMs has correct value', () {
      expect(TimingConfig.pulseAnimationMs, equals(300));
    });

    test('debounceMs has correct value', () {
      expect(TimingConfig.debounceMs, equals(500));
    });

    test('keyAnimationMs has correct value', () {
      expect(TimingConfig.keyAnimationMs, equals(100));
    });

    test('typingTimeLimitSec has correct value', () {
      expect(TimingConfig.typingTimeLimitSec, equals(30));
    });

    test('trainingAnimationMs has correct value', () {
      expect(TimingConfig.trainingAnimationMs, equals(400));
    });

    test('tooltipDelayMs has correct value', () {
      expect(TimingConfig.tooltipDelayMs, equals(150));
    });

    test('all timing values are positive', () {
      expect(TimingConfig.animationDurationMs, greaterThan(0));
      expect(TimingConfig.pulseAnimationMs, greaterThan(0));
      expect(TimingConfig.debounceMs, greaterThan(0));
      expect(TimingConfig.keyAnimationMs, greaterThan(0));
      expect(TimingConfig.typingTimeLimitSec, greaterThan(0));
      expect(TimingConfig.trainingAnimationMs, greaterThan(0));
      expect(TimingConfig.tooltipDelayMs, greaterThan(0));
    });
  });

  group('UiConstants', () {
    test('defaultPadding has correct value', () {
      expect(UiConstants.defaultPadding, equals(16.0));
    });

    test('smallPadding has correct value', () {
      expect(UiConstants.smallPadding, equals(8.0));
    });

    test('tinyPadding has correct value', () {
      expect(UiConstants.tinyPadding, equals(4.0));
    });

    test('defaultElevation has correct value', () {
      expect(UiConstants.defaultElevation, equals(6.0));
    });

    test('minKeyboardScale has correct value', () {
      expect(UiConstants.minKeyboardScale, equals(0.5));
    });

    test('maxKeyboardScale has correct value', () {
      expect(UiConstants.maxKeyboardScale, equals(1.0));
    });

    test('defaultIconSize has correct value', () {
      expect(UiConstants.defaultIconSize, equals(24.0));
    });

    test('defaultBorderRadius has correct value', () {
      expect(UiConstants.defaultBorderRadius, equals(4.0));
    });

    test('keyboard scale range is valid', () {
      expect(UiConstants.minKeyboardScale, lessThan(UiConstants.maxKeyboardScale));
      expect(UiConstants.minKeyboardScale, greaterThan(0));
      expect(UiConstants.maxKeyboardScale, greaterThan(0));
    });

    test('all dimension values are positive', () {
      expect(UiConstants.defaultPadding, greaterThan(0));
      expect(UiConstants.smallPadding, greaterThan(0));
      expect(UiConstants.tinyPadding, greaterThan(0));
      expect(UiConstants.defaultElevation, greaterThan(0));
      expect(UiConstants.defaultIconSize, greaterThan(0));
      expect(UiConstants.defaultBorderRadius, greaterThanOrEqualTo(0));
    });
  });

  group('StorageKeys', () {
    test('developerModeKey has correct value', () {
      expect(StorageKeys.developerModeKey, equals('developer_mode'));
    });

    test('trainingProgressKey has correct value', () {
      expect(StorageKeys.trainingProgressKey, equals('keyrx_training_progress'));
    });

    test('storage keys are not empty', () {
      expect(StorageKeys.developerModeKey, isNotEmpty);
      expect(StorageKeys.trainingProgressKey, isNotEmpty);
    });

    test('storage keys are unique', () {
      final keys = [
        StorageKeys.developerModeKey,
        StorageKeys.trainingProgressKey,
      ];
      expect(keys.toSet().length, equals(keys.length));
    });
  });

  group('FfiFunctions', () {
    test('init has correct value', () {
      expect(FfiFunctions.init, equals('keyrx_init'));
    });

    test('version has correct value', () {
      expect(FfiFunctions.version, equals('keyrx_version'));
    });

    test('loadScript has correct value', () {
      expect(FfiFunctions.loadScript, equals('keyrx_load_script'));
    });

    test('all function names start with keyrx_', () {
      expect(FfiFunctions.init, startsWith('keyrx_'));
      expect(FfiFunctions.version, startsWith('keyrx_'));
      expect(FfiFunctions.loadScript, startsWith('keyrx_'));
      expect(FfiFunctions.eval, startsWith('keyrx_'));
      expect(FfiFunctions.simulate, startsWith('keyrx_'));
      expect(FfiFunctions.runBenchmark, startsWith('keyrx_'));
      expect(FfiFunctions.validateScript, startsWith('keyrx_'));
      expect(FfiFunctions.listKeys, startsWith('keyrx_'));
      expect(FfiFunctions.checkScript, startsWith('keyrx_'));
      expect(FfiFunctions.discoverTests, startsWith('keyrx_'));
      expect(FfiFunctions.runTests, startsWith('keyrx_'));
    });

    test('all function names are not empty', () {
      expect(FfiFunctions.init, isNotEmpty);
      expect(FfiFunctions.version, isNotEmpty);
      expect(FfiFunctions.loadScript, isNotEmpty);
      expect(FfiFunctions.freeString, isNotEmpty);
      expect(FfiFunctions.eval, isNotEmpty);
      expect(FfiFunctions.simulate, isNotEmpty);
      expect(FfiFunctions.runBenchmark, isNotEmpty);
      expect(FfiFunctions.validateScript, isNotEmpty);
      expect(FfiFunctions.listKeys, isNotEmpty);
      expect(FfiFunctions.startRecording, isNotEmpty);
      expect(FfiFunctions.stopRecording, isNotEmpty);
    });
  });

  group('JsonKeys', () {
    test('success key has correct value', () {
      expect(JsonKeys.success, equals('success'));
    });

    test('error key has correct value', () {
      expect(JsonKeys.error, equals('error'));
    });

    test('totalEvents has correct value', () {
      expect(JsonKeys.totalEvents, equals('totalEvents'));
    });

    test('avgLatencyUs has correct value', () {
      expect(JsonKeys.avgLatencyUs, equals('avgLatencyUs'));
    });

    test('all JSON keys are not empty', () {
      expect(JsonKeys.success, isNotEmpty);
      expect(JsonKeys.error, isNotEmpty);
      expect(JsonKeys.totalEvents, isNotEmpty);
      expect(JsonKeys.avgLatencyUs, isNotEmpty);
      expect(JsonKeys.minLatencyUs, isNotEmpty);
      expect(JsonKeys.maxLatencyUs, isNotEmpty);
      expect(JsonKeys.path, isNotEmpty);
      expect(JsonKeys.name, isNotEmpty);
      expect(JsonKeys.latencyUs, isNotEmpty);
    });
  });

  group('ResponsePrefixes', () {
    test('ok prefix has correct value', () {
      expect(ResponsePrefixes.ok, equals('ok:'));
    });

    test('error prefix has correct value', () {
      expect(ResponsePrefixes.error, equals('error:'));
    });

    test('prefixes end with colon', () {
      expect(ResponsePrefixes.ok, endsWith(':'));
      expect(ResponsePrefixes.error, endsWith(':'));
    });

    test('prefixes are not empty', () {
      expect(ResponsePrefixes.ok, isNotEmpty);
      expect(ResponsePrefixes.error, isNotEmpty);
    });
  });

  group('ThresholdConstants', () {
    test('latencyWarningUs has correct value', () {
      expect(ThresholdConstants.latencyWarningUs, equals(20000));
    });

    test('latencyCautionUs has correct value', () {
      expect(ThresholdConstants.latencyCautionUs, equals(10000));
    });

    test('warningThresholdNs has correct value', () {
      expect(ThresholdConstants.warningThresholdNs, equals(1000000));
    });

    test('minKeystrokes has correct value', () {
      expect(ThresholdConstants.minKeystrokes, equals(10));
    });

    test('pauseThresholdMs has correct value', () {
      expect(ThresholdConstants.pauseThresholdMs, equals(2000));
    });

    test('maxEventsHistory has correct value', () {
      expect(ThresholdConstants.maxEventsHistory, equals(300));
    });

    test('latency thresholds are ordered correctly', () {
      expect(
        ThresholdConstants.latencyCautionUs,
        lessThan(ThresholdConstants.latencyWarningUs),
      );
    });

    test('all threshold values are positive', () {
      expect(ThresholdConstants.latencyWarningUs, greaterThan(0));
      expect(ThresholdConstants.latencyCautionUs, greaterThan(0));
      expect(ThresholdConstants.warningThresholdNs, greaterThan(0));
      expect(ThresholdConstants.minKeystrokes, greaterThan(0));
      expect(ThresholdConstants.pauseThresholdMs, greaterThan(0));
      expect(ThresholdConstants.maxEventsHistory, greaterThan(0));
    });
  });

  group('PathConstants', () {
    test('defaultScriptPath has correct value', () {
      expect(PathConstants.defaultScriptPath, equals('scripts/generated.rhai'));
    });

    test('tempValidationPath has correct value', () {
      expect(PathConstants.tempValidationPath, equals('/tmp/keyrx_validation.rhai'));
    });

    test('scriptsDir has correct value', () {
      expect(PathConstants.scriptsDir, equals('scripts/'));
    });

    test('defaultConfigFileName has correct value', () {
      expect(PathConstants.defaultConfigFileName, equals('config.rhai'));
    });

    test('all paths are not empty', () {
      expect(PathConstants.defaultScriptPath, isNotEmpty);
      expect(PathConstants.tempValidationPath, isNotEmpty);
      expect(PathConstants.scriptsDir, isNotEmpty);
      expect(PathConstants.defaultConfigFileName, isNotEmpty);
    });

    test('scriptsDir ends with slash', () {
      expect(PathConstants.scriptsDir, endsWith('/'));
    });

    test('script files have .rhai extension', () {
      expect(PathConstants.defaultScriptPath, endsWith('.rhai'));
      expect(PathConstants.tempValidationPath, endsWith('.rhai'));
      expect(PathConstants.defaultConfigFileName, endsWith('.rhai'));
    });
  });

  group('Config Module Integration', () {
    test('all modules are exported from barrel file', () {
      // This test verifies that importing from config.dart works
      // If any export is missing, compilation would fail
      expect(TimingConfig.animationDurationMs, isA<int>());
      expect(UiConstants.defaultPadding, isA<double>());
      expect(StorageKeys.developerModeKey, isA<String>());
      expect(FfiFunctions.init, isA<String>());
      expect(JsonKeys.success, isA<String>());
      expect(ResponsePrefixes.ok, isA<String>());
      expect(ThresholdConstants.latencyWarningUs, isA<int>());
      expect(PathConstants.defaultScriptPath, isA<String>());
    });

    test('no runtime errors when accessing constants', () {
      // Verify that all constants can be accessed without errors
      expect(() => TimingConfig.animationDurationMs, returnsNormally);
      expect(() => UiConstants.defaultPadding, returnsNormally);
      expect(() => StorageKeys.developerModeKey, returnsNormally);
      expect(() => FfiFunctions.init, returnsNormally);
      expect(() => JsonKeys.success, returnsNormally);
      expect(() => ResponsePrefixes.ok, returnsNormally);
      expect(() => ThresholdConstants.latencyWarningUs, returnsNormally);
      expect(() => PathConstants.defaultScriptPath, returnsNormally);
    });

    test('constants maintain expected types', () {
      expect(TimingConfig.animationDurationMs, isA<int>());
      expect(TimingConfig.pulseAnimationMs, isA<int>());
      expect(UiConstants.defaultPadding, isA<double>());
      expect(UiConstants.smallPadding, isA<double>());
      expect(StorageKeys.developerModeKey, isA<String>());
      expect(ThresholdConstants.latencyWarningUs, isA<int>());
      expect(PathConstants.defaultScriptPath, isA<String>());
    });
  });
}

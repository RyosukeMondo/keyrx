import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

import 'package:keyrx_ui/pages/developer/test_runner_page.dart';
import 'package:keyrx_ui/pages/developer/benchmark_page.dart';
import 'package:keyrx_ui/pages/developer/doctor_page.dart';
import 'package:keyrx_ui/services/test_service.dart';
import 'package:keyrx_ui/services/benchmark_service.dart';
import 'package:keyrx_ui/services/doctor_service.dart';

// Mock services
class MockTestService extends Mock implements TestService {}

class MockBenchmarkService extends Mock implements BenchmarkService {}

class MockDoctorService extends Mock implements DoctorService {}

void main() {
  group('TestRunnerPage', () {
    late MockTestService mockTestService;

    setUp(() {
      mockTestService = MockTestService();
    });

    testWidgets('shows empty state initially', (tester) async {
      await tester.pumpWidget(
        MaterialApp(home: TestRunnerPage(testService: mockTestService)),
      );

      expect(find.text('No tests discovered'), findsOneWidget);
      expect(find.text('Discover'), findsOneWidget);
    });

    testWidgets('shows error when script path is empty', (tester) async {
      await tester.pumpWidget(
        MaterialApp(home: TestRunnerPage(testService: mockTestService)),
      );

      await tester.tap(find.text('Discover'));
      await tester.pumpAndSettle();

      expect(find.text('Please enter a script path'), findsOneWidget);
    });

    testWidgets('discovers tests when path provided', (tester) async {
      when(() => mockTestService.discoverTests(any())).thenAnswer(
        (_) async => TestDiscoveryServiceResult(
          tests: [
            const TestCase(name: 'test_example', file: 'script.rhai', line: 10),
          ],
        ),
      );

      await tester.pumpWidget(
        MaterialApp(home: TestRunnerPage(testService: mockTestService)),
      );

      await tester.enterText(find.byType(TextField).first, 'script.rhai');
      await tester.tap(find.text('Discover'));
      await tester.pumpAndSettle();

      expect(find.text('test_example'), findsOneWidget);
      verify(() => mockTestService.discoverTests('script.rhai')).called(1);
    });
  });

  group('BenchmarkPage', () {
    late MockBenchmarkService mockBenchmarkService;

    setUp(() {
      mockBenchmarkService = MockBenchmarkService();
    });

    testWidgets('shows configuration card', (tester) async {
      await tester.pumpWidget(
        MaterialApp(home: BenchmarkPage(benchmarkService: mockBenchmarkService)),
      );

      expect(find.text('Configuration'), findsOneWidget);
      expect(find.text('Run Benchmark'), findsOneWidget);
      expect(find.byType(Slider), findsOneWidget);
    });

    testWidgets('shows placeholder when no results', (tester) async {
      await tester.pumpWidget(
        MaterialApp(home: BenchmarkPage(benchmarkService: mockBenchmarkService)),
      );

      expect(find.text('Run a benchmark to see results'), findsOneWidget);
    });

    testWidgets('runs benchmark and shows results', (tester) async {
      when(() => mockBenchmarkService.runBenchmark(any())).thenAnswer(
        (_) async => BenchmarkServiceResult(
          data: const BenchmarkData(
            minNs: 100,
            maxNs: 500,
            meanNs: 250,
            p99Ns: 450,
            iterations: 10000,
            hasWarning: false,
          ),
        ),
      );

      await tester.pumpWidget(
        MaterialApp(home: BenchmarkPage(benchmarkService: mockBenchmarkService)),
      );

      await tester.tap(find.text('Run Benchmark'));
      await tester.pumpAndSettle();

      expect(find.text('Results'), findsOneWidget);
      expect(find.text('Min'), findsOneWidget);
      expect(find.text('Mean'), findsOneWidget);
      expect(find.text('P99'), findsOneWidget);
      expect(find.text('Max'), findsOneWidget);
    });

    testWidgets('shows warning banner when latency exceeds threshold',
        (tester) async {
      when(() => mockBenchmarkService.runBenchmark(any())).thenAnswer(
        (_) async => BenchmarkServiceResult(
          data: const BenchmarkData(
            minNs: 100000,
            maxNs: 2000000, // 2ms
            meanNs: 1500000,
            p99Ns: 1800000,
            iterations: 10000,
            hasWarning: true,
            warning: 'Latency exceeds 1ms',
          ),
        ),
      );

      await tester.pumpWidget(
        MaterialApp(home: BenchmarkPage(benchmarkService: mockBenchmarkService)),
      );

      await tester.tap(find.text('Run Benchmark'));
      await tester.pumpAndSettle();

      expect(find.text('Latency exceeds 1ms'), findsOneWidget);
    });
  });

  group('DoctorPage', () {
    late MockDoctorService mockDoctorService;

    setUp(() {
      mockDoctorService = MockDoctorService();
    });

    testWidgets('auto-runs diagnostics on open', (tester) async {
      when(() => mockDoctorService.runDiagnostics()).thenAnswer(
        (_) async => DoctorServiceResult(
          report: const DiagnosticReport(
            checks: [
              DiagnosticCheckData(name: 'Check 1', status: 'pass'),
              DiagnosticCheckData(name: 'Check 2', status: 'pass'),
            ],
            passed: 2,
            failed: 0,
            warned: 0,
          ),
        ),
      );

      await tester.pumpWidget(
        MaterialApp(home: DoctorPage(doctorService: mockDoctorService)),
      );

      await tester.pumpAndSettle();

      verify(() => mockDoctorService.runDiagnostics()).called(1);
      expect(find.text('All checks passed!'), findsOneWidget);
    });

    testWidgets('shows loading state while running', (tester) async {
      final completer = Completer<DoctorServiceResult>();
      when(() => mockDoctorService.runDiagnostics())
          .thenAnswer((_) => completer.future);

      await tester.pumpWidget(
        MaterialApp(home: DoctorPage(doctorService: mockDoctorService)),
      );

      expect(find.text('Running diagnostics...'), findsOneWidget);
      expect(find.byType(CircularProgressIndicator), findsOneWidget);

      completer.complete(DoctorServiceResult(
        report: const DiagnosticReport(
          checks: [],
          passed: 0,
          failed: 0,
          warned: 0,
        ),
      ));
      await tester.pumpAndSettle();
    });

    testWidgets('shows check list with status icons', (tester) async {
      when(() => mockDoctorService.runDiagnostics()).thenAnswer(
        (_) async => DoctorServiceResult(
          report: const DiagnosticReport(
            checks: [
              DiagnosticCheckData(name: 'Passed Check', status: 'pass'),
              DiagnosticCheckData(
                name: 'Failed Check',
                status: 'fail',
                details: 'Something went wrong',
                remediation: 'Try restarting',
              ),
              DiagnosticCheckData(name: 'Warning Check', status: 'warn'),
            ],
            passed: 1,
            failed: 1,
            warned: 1,
          ),
        ),
      );

      await tester.pumpWidget(
        MaterialApp(home: DoctorPage(doctorService: mockDoctorService)),
      );

      await tester.pumpAndSettle();

      expect(find.text('Passed Check'), findsOneWidget);
      expect(find.text('Failed Check'), findsOneWidget);
      expect(find.text('Warning Check'), findsOneWidget);
      expect(find.text('Some issues found'), findsOneWidget);
    });

    testWidgets('shows error state on failure', (tester) async {
      when(() => mockDoctorService.runDiagnostics()).thenAnswer(
        (_) async => DoctorServiceResult.error('Connection failed'),
      );

      await tester.pumpWidget(
        MaterialApp(home: DoctorPage(doctorService: mockDoctorService)),
      );

      await tester.pumpAndSettle();

      expect(find.text('Diagnostics failed'), findsOneWidget);
      expect(find.text('Connection failed'), findsOneWidget);
    });
  });
}

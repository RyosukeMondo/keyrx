/// Test Runner page for discovering and executing Rhai script tests.
///
/// Provides test list with status indicators, filtering, and expandable
/// error details for failed tests.
library;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../services/test_service.dart';
import '../../services/facade/keyrx_facade.dart';

/// Test Runner page for discovering and executing tests.
class TestRunnerPage extends StatefulWidget {
  const TestRunnerPage({super.key});

  @override
  State<TestRunnerPage> createState() => _TestRunnerPageState();
}

class _TestRunnerPageState extends State<TestRunnerPage> {
  final _filterController = TextEditingController();
  final _scriptPathController = TextEditingController();

  List<TestCase> _discoveredTests = [];
  Map<String, TestCaseResult?> _testResults = {};
  bool _isDiscovering = false;
  bool _isRunning = false;
  String? _error;
  int _passed = 0;
  int _failed = 0;

  @override
  void dispose() {
    _filterController.dispose();
    _scriptPathController.dispose();
    super.dispose();
  }

  Future<void> _discoverTests() async {
    final path = _scriptPathController.text.trim();
    if (path.isEmpty) {
      setState(() => _error = 'Please enter a script path');
      return;
    }

    setState(() {
      _isDiscovering = true;
      _error = null;
      _discoveredTests = [];
      _testResults = {};
      _passed = 0;
      _failed = 0;
    });

    final facade = context.read<KeyrxFacade>();
    final facadeResult = await facade.discoverTests(path);

    if (!mounted) return;

    await facadeResult.when(
      ok: (result) async {
        setState(() {
          _isDiscovering = false;
          _discoveredTests = result.tests;
          _testResults = {for (final t in result.tests) t.name: null};
        });
      },
      err: (error) async {
        setState(() {
          _isDiscovering = false;
          _error = error.userMessage;
        });
      },
    );
  }

  Future<void> _runAllTests() async {
    final path = _scriptPathController.text.trim();
    if (path.isEmpty || _discoveredTests.isEmpty) return;

    setState(() {
      _isRunning = true;
      _error = null;
      _testResults = {for (final t in _discoveredTests) t.name: null};
      _passed = 0;
      _failed = 0;
    });

    final filter = _filterController.text.trim();
    final facade = context.read<KeyrxFacade>();
    final facadeResult = await facade.runTests(
      path,
      filter: filter.isNotEmpty ? filter : null,
    );

    if (!mounted) return;

    await facadeResult.when(
      ok: (result) async {
        setState(() {
          _isRunning = false;
          for (final r in result.results) {
            _testResults[r.name] = r;
          }
          _passed = result.passed;
          _failed = result.failed;
        });
      },
      err: (error) async {
        setState(() {
          _isRunning = false;
          _error = error.userMessage;
        });
      },
    );
  }

  Future<void> _runSingleTest(TestCase test) async {
    final path = _scriptPathController.text.trim();
    if (path.isEmpty) return;

    setState(() {
      _testResults[test.name] = null;
      _isRunning = true;
    });

    final facade = context.read<KeyrxFacade>();
    final facadeResult = await facade.runTests(path, filter: test.name);

    if (!mounted) return;

    await facadeResult.when(
      ok: (result) async {
        setState(() {
          _isRunning = false;
          if (result.results.isNotEmpty) {
            _testResults[test.name] = result.results.first;
            _passed = _testResults.values.where((r) => r?.passed == true).length;
            _failed = _testResults.values.where((r) => r?.passed == false).length;
          }
        });
      },
      err: (_) async {
        setState(() {
          _isRunning = false;
        });
      },
    );
  }

  List<TestCase> get _filteredTests {
    final filter = _filterController.text.toLowerCase();
    if (filter.isEmpty) return _discoveredTests;
    return _discoveredTests
        .where((t) => t.name.toLowerCase().contains(filter))
        .toList();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Test Runner'),
        actions: [
          if (_discoveredTests.isNotEmpty)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: _buildStatusChips(),
            ),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            _buildScriptInput(),
            const SizedBox(height: 16),
            if (_discoveredTests.isNotEmpty) ...[
              _buildFilterAndActions(),
              const SizedBox(height: 16),
            ],
            if (_error != null) _buildErrorBanner(),
            Expanded(child: _buildTestList()),
          ],
        ),
      ),
    );
  }

  Widget _buildScriptInput() {
    return Row(
      children: [
        Expanded(
          child: TextField(
            controller: _scriptPathController,
            decoration: const InputDecoration(
              labelText: 'Script Path',
              hintText: 'path/to/script.rhai',
              border: OutlineInputBorder(),
              prefixIcon: Icon(Icons.description),
            ),
            onSubmitted: (_) => _discoverTests(),
          ),
        ),
        const SizedBox(width: 8),
        FilledButton.icon(
          onPressed: _isDiscovering ? null : _discoverTests,
          icon: _isDiscovering
              ? const SizedBox(
                  width: 16,
                  height: 16,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Icon(Icons.search),
          label: const Text('Discover'),
        ),
      ],
    );
  }

  Widget _buildFilterAndActions() {
    return Row(
      children: [
        Expanded(
          child: TextField(
            controller: _filterController,
            decoration: const InputDecoration(
              labelText: 'Filter tests',
              hintText: 'Type to filter...',
              border: OutlineInputBorder(),
              prefixIcon: Icon(Icons.filter_list),
              isDense: true,
            ),
            onChanged: (_) => setState(() {}),
          ),
        ),
        const SizedBox(width: 8),
        FilledButton.icon(
          onPressed: _isRunning ? null : _runAllTests,
          icon: _isRunning
              ? const SizedBox(
                  width: 16,
                  height: 16,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Icon(Icons.play_arrow),
          label: const Text('Run All'),
        ),
      ],
    );
  }

  Widget _buildStatusChips() {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Chip(
          avatar: const Icon(Icons.check_circle, color: Colors.green, size: 18),
          label: Text('$_passed passed'),
          visualDensity: VisualDensity.compact,
        ),
        const SizedBox(width: 8),
        Chip(
          avatar: const Icon(Icons.cancel, color: Colors.red, size: 18),
          label: Text('$_failed failed'),
          visualDensity: VisualDensity.compact,
        ),
      ],
    );
  }

  Widget _buildErrorBanner() {
    return Card(
      color: Theme.of(context).colorScheme.errorContainer,
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Row(
          children: [
            Icon(Icons.error, color: Theme.of(context).colorScheme.error),
            const SizedBox(width: 12),
            Expanded(
              child: Text(
                _error!,
                style: TextStyle(
                  color: Theme.of(context).colorScheme.onErrorContainer,
                ),
              ),
            ),
            IconButton(
              icon: const Icon(Icons.close),
              onPressed: () => setState(() => _error = null),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildTestList() {
    final tests = _filteredTests;

    if (_discoveredTests.isEmpty && !_isDiscovering) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.science_outlined, size: 64, color: Colors.grey[600]),
            const SizedBox(height: 16),
            Text(
              'No tests discovered',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            Text(
              'Enter a script path and click Discover to find tests',
              style: TextStyle(color: Colors.grey[500]),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      itemCount: tests.length,
      itemBuilder: (context, index) => _buildTestTile(tests[index]),
    );
  }

  Widget _buildTestTile(TestCase test) {
    final result = _testResults[test.name];
    final hasRun = result != null;
    final passed = result?.passed ?? false;

    return Card(
      child: ExpansionTile(
        leading: _buildStatusIcon(hasRun, passed),
        title: Text(test.name),
        subtitle: Text(
          test.line != null ? '${test.file}:${test.line}' : test.file,
          style: Theme.of(context).textTheme.bodySmall,
        ),
        trailing: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (result != null)
              Text(
                '${result.durationMs.toStringAsFixed(1)}ms',
                style: Theme.of(context).textTheme.bodySmall,
              ),
            const SizedBox(width: 8),
            IconButton(
              icon: const Icon(Icons.play_arrow),
              onPressed: _isRunning ? null : () => _runSingleTest(test),
              tooltip: 'Run this test',
            ),
          ],
        ),
        children: [
          if (result?.error != null)
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(16),
              color: Theme.of(context).colorScheme.errorContainer.withAlpha(50),
              child: SelectableText(
                result!.error!,
                style: TextStyle(
                  fontFamily: 'monospace',
                  fontSize: 12,
                  color: Theme.of(context).colorScheme.error,
                ),
              ),
            ),
          if (result != null && result.error == null && result.passed)
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(16),
              child: Text(
                'Test passed successfully',
                style: TextStyle(color: Colors.green[400]),
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildStatusIcon(bool hasRun, bool passed) {
    if (!hasRun) {
      return const Icon(Icons.circle_outlined, color: Colors.grey);
    }
    return passed
        ? const Icon(Icons.check_circle, color: Colors.green)
        : const Icon(Icons.cancel, color: Colors.red);
  }
}

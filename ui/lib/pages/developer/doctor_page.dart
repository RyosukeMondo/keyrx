/// Doctor page for system diagnostics with remediation.
///
/// Provides auto-run diagnostics, check list with status icons,
/// expandable details, remediation steps, and summary counts.
library;

import 'package:flutter/material.dart';

import '../../services/doctor_service.dart';

/// Doctor page for running system diagnostics.
class DoctorPage extends StatefulWidget {
  const DoctorPage({super.key, required this.doctorService});

  final DoctorService doctorService;

  @override
  State<DoctorPage> createState() => _DoctorPageState();
}

class _DoctorPageState extends State<DoctorPage> {
  DiagnosticReport? _report;
  bool _isRunning = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _runDiagnostics();
  }

  Future<void> _runDiagnostics() async {
    setState(() {
      _isRunning = true;
      _error = null;
    });

    final result = await widget.doctorService.runDiagnostics();

    setState(() {
      _isRunning = false;
      if (result.hasError) {
        _error = result.errorMessage;
      } else {
        _report = result.report;
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('System Diagnostics'),
        actions: [
          if (_report != null) _buildSummaryChips(),
          const SizedBox(width: 8),
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _isRunning ? null : _runDiagnostics,
            tooltip: 'Re-run diagnostics',
          ),
        ],
      ),
      body: _buildBody(),
    );
  }

  Widget _buildSummaryChips() {
    final report = _report!;
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        _buildCountChip(report.passed, Colors.green, Icons.check_circle),
        const SizedBox(width: 4),
        if (report.warned > 0) ...[
          _buildCountChip(report.warned, Colors.orange, Icons.warning),
          const SizedBox(width: 4),
        ],
        if (report.failed > 0)
          _buildCountChip(report.failed, Colors.red, Icons.cancel),
      ],
    );
  }

  Widget _buildCountChip(int count, Color color, IconData icon) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withAlpha(30),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 16, color: color),
          const SizedBox(width: 4),
          Text(
            '$count',
            style: TextStyle(color: color, fontWeight: FontWeight.bold),
          ),
        ],
      ),
    );
  }

  Widget _buildBody() {
    if (_isRunning) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 16),
            Text('Running diagnostics...'),
          ],
        ),
      );
    }

    if (_error != null) {
      return _buildErrorState();
    }

    if (_report == null) {
      return const Center(child: Text('No diagnostics available'));
    }

    return _buildCheckList();
  }

  Widget _buildErrorState() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.error_outline, size: 64, color: Colors.red[300]),
          const SizedBox(height: 16),
          Text(
            'Diagnostics failed',
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const SizedBox(height: 8),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 32),
            child: Text(
              _error!,
              textAlign: TextAlign.center,
              style: TextStyle(color: Colors.grey[500]),
            ),
          ),
          const SizedBox(height: 24),
          FilledButton.icon(
            onPressed: _runDiagnostics,
            icon: const Icon(Icons.refresh),
            label: const Text('Retry'),
          ),
        ],
      ),
    );
  }

  Widget _buildCheckList() {
    final checks = _report!.checks;

    return ListView.builder(
      padding: const EdgeInsets.all(16),
      itemCount: checks.length + 1,
      itemBuilder: (context, index) {
        if (index == 0) {
          return _buildSummaryCard();
        }
        return _buildCheckTile(checks[index - 1]);
      },
    );
  }

  Widget _buildSummaryCard() {
    final report = _report!;
    final allGood = report.allPassed && !report.hasWarnings;

    return Card(
      color: allGood ? Colors.green.shade50 : null,
      margin: const EdgeInsets.only(bottom: 16),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            Icon(
              allGood ? Icons.check_circle : Icons.info,
              size: 48,
              color: allGood ? Colors.green : Colors.blue,
            ),
            const SizedBox(width: 16),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    allGood ? 'All checks passed!' : 'Some issues found',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  const SizedBox(height: 4),
                  Text(
                    '${report.passed} passed, ${report.warned} warnings, ${report.failed} failed',
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildCheckTile(DiagnosticCheckData check) {
    final hasDetails = check.details != null || check.remediation != null;

    return Card(
      margin: const EdgeInsets.only(bottom: 8),
      child: hasDetails
          ? ExpansionTile(
              leading: _buildStatusIcon(check),
              title: Text(check.name),
              subtitle: _buildStatusText(check),
              children: [_buildExpandedContent(check)],
            )
          : ListTile(
              leading: _buildStatusIcon(check),
              title: Text(check.name),
              subtitle: _buildStatusText(check),
            ),
    );
  }

  Widget _buildStatusIcon(DiagnosticCheckData check) {
    if (check.passed) {
      return const Icon(Icons.check_circle, color: Colors.green);
    } else if (check.warned) {
      return const Icon(Icons.warning, color: Colors.orange);
    } else {
      return const Icon(Icons.cancel, color: Colors.red);
    }
  }

  Widget? _buildStatusText(DiagnosticCheckData check) {
    if (check.passed) return null;
    if (check.warned) {
      return const Text('Warning', style: TextStyle(color: Colors.orange));
    }
    return const Text('Failed', style: TextStyle(color: Colors.red));
  }

  Widget _buildExpandedContent(DiagnosticCheckData check) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (check.details != null) ...[
            Text('Details', style: Theme.of(context).textTheme.labelMedium),
            const SizedBox(height: 4),
            Text(check.details!, style: Theme.of(context).textTheme.bodySmall),
          ],
          if (check.remediation != null) ...[
            const SizedBox(height: 12),
            Text(
              'How to fix',
              style: Theme.of(
                context,
              ).textTheme.labelMedium?.copyWith(color: Colors.blue),
            ),
            const SizedBox(height: 4),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.blue.shade50,
                borderRadius: BorderRadius.circular(8),
              ),
              child: Text(
                check.remediation!,
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ),
          ],
        ],
      ),
    );
  }
}

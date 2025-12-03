/// Analyzer page for session analysis with statistics and timeline.
///
/// Provides session file picking, statistics cards, decision breakdown
/// pie chart, and timeline view with event details.
library;

import 'package:fl_chart/fl_chart.dart';
import 'package:flutter/material.dart';

import '../../services/session_service.dart';

/// Session analyzer page for viewing and analyzing recorded sessions.
class AnalyzerPage extends StatefulWidget {
  const AnalyzerPage({super.key, required this.sessionService});

  final SessionService sessionService;

  @override
  State<AnalyzerPage> createState() => _AnalyzerPageState();
}

class _AnalyzerPageState extends State<AnalyzerPage> {
  final _sessionPathController = TextEditingController();

  List<SessionRecord> _sessions = [];
  SessionRecord? _selectedSession;
  SessionAnalysisData? _analysis;
  bool _isLoading = false;
  bool _isAnalyzing = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadSessions();
  }

  @override
  void dispose() {
    _sessionPathController.dispose();
    super.dispose();
  }

  Future<void> _loadSessions() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    final result = await widget.sessionService.listSessions('sessions/');

    setState(() {
      _isLoading = false;
      if (result.hasError) {
        _error = result.errorMessage;
      } else {
        _sessions = result.sessions;
      }
    });
  }

  Future<void> _analyzeSession(SessionRecord session) async {
    setState(() {
      _selectedSession = session;
      _isAnalyzing = true;
      _error = null;
      _analysis = null;
    });

    final result = await widget.sessionService.analyze(session.path);

    setState(() {
      _isAnalyzing = false;
      if (result.hasError) {
        _error = result.errorMessage;
      } else {
        _analysis = result.analysis;
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Session Analyzer'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _isLoading ? null : _loadSessions,
            tooltip: 'Refresh sessions',
          ),
        ],
      ),
      body: Row(
        children: [
          // Session list panel
          SizedBox(
            width: 280,
            child: _buildSessionList(),
          ),
          const VerticalDivider(width: 1),
          // Analysis panel
          Expanded(
            child: _buildAnalysisPanel(),
          ),
        ],
      ),
    );
  }

  Widget _buildSessionList() {
    if (_isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_error != null && _sessions.isEmpty) {
      return _buildEmptyState(
        icon: Icons.error_outline,
        title: 'Error loading sessions',
        subtitle: _error!,
      );
    }

    if (_sessions.isEmpty) {
      return _buildEmptyState(
        icon: Icons.folder_open,
        title: 'No sessions found',
        subtitle: 'Record a session to analyze it here',
      );
    }

    return ListView.builder(
      itemCount: _sessions.length,
      itemBuilder: (context, index) {
        final session = _sessions[index];
        final isSelected = _selectedSession?.path == session.path;

        return ListTile(
          selected: isSelected,
          leading: const Icon(Icons.description),
          title: Text(session.name, maxLines: 1, overflow: TextOverflow.ellipsis),
          subtitle: Text(
            '${session.eventCount} events • ${_formatDuration(session.durationMs)}',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          trailing: Text(
            _formatDate(session.created),
            style: Theme.of(context).textTheme.labelSmall,
          ),
          onTap: () => _analyzeSession(session),
        );
      },
    );
  }

  Widget _buildEmptyState({
    required IconData icon,
    required String title,
    required String subtitle,
  }) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(icon, size: 48, color: Colors.grey),
          const SizedBox(height: 16),
          Text(title, style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 8),
          Text(subtitle, style: TextStyle(color: Colors.grey[500])),
        ],
      ),
    );
  }

  Widget _buildAnalysisPanel() {
    if (_selectedSession == null) {
      return _buildEmptyState(
        icon: Icons.analytics_outlined,
        title: 'Select a session',
        subtitle: 'Choose a session from the list to analyze',
      );
    }

    if (_isAnalyzing) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_error != null) {
      return _buildEmptyState(
        icon: Icons.error_outline,
        title: 'Analysis failed',
        subtitle: _error!,
      );
    }

    if (_analysis == null) {
      return _buildEmptyState(
        icon: Icons.hourglass_empty,
        title: 'No analysis data',
        subtitle: 'Something went wrong during analysis',
      );
    }

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _buildStatisticsRow(),
          const SizedBox(height: 24),
          _buildDecisionBreakdown(),
          const SizedBox(height: 24),
          _buildLatencyCard(),
        ],
      ),
    );
  }

  Widget _buildStatisticsRow() {
    final analysis = _analysis!;

    return Row(
      children: [
        Expanded(child: _buildStatCard('Events', '${analysis.eventCount}', Icons.event)),
        const SizedBox(width: 16),
        Expanded(
          child: _buildStatCard(
            'Duration',
            _formatDuration(analysis.durationMs),
            Icons.timer,
          ),
        ),
        const SizedBox(width: 16),
        Expanded(
          child: _buildStatCard(
            'Avg Latency',
            '${analysis.avgLatencyUs}µs',
            Icons.speed,
          ),
        ),
      ],
    );
  }

  Widget _buildStatCard(String label, String value, IconData icon) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            Icon(icon, size: 32, color: Theme.of(context).colorScheme.primary),
            const SizedBox(height: 8),
            Text(
              value,
              style: Theme.of(context).textTheme.headlineSmall,
            ),
            Text(label, style: Theme.of(context).textTheme.bodySmall),
          ],
        ),
      ),
    );
  }

  Widget _buildDecisionBreakdown() {
    final breakdown = _analysis!.decisionBreakdown;
    final sections = <_PieSection>[
      _PieSection('Pass-through', breakdown.passThrough, Colors.green),
      _PieSection('Remap', breakdown.remap, Colors.blue),
      _PieSection('Block', breakdown.block, Colors.red),
      _PieSection('Tap', breakdown.tap, Colors.orange),
      _PieSection('Hold', breakdown.hold, Colors.purple),
      _PieSection('Combo', breakdown.combo, Colors.teal),
      _PieSection('Layer', breakdown.layer, Colors.amber),
      _PieSection('Modifier', breakdown.modifier, Colors.indigo),
    ].where((s) => s.value > 0).toList();

    final total = sections.fold(0, (sum, s) => sum + s.value);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Decision Breakdown', style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 16),
            SizedBox(
              height: 200,
              child: Row(
                children: [
                  Expanded(
                    child: PieChart(
                      PieChartData(
                        sectionsSpace: 2,
                        centerSpaceRadius: 40,
                        sections: sections.map((s) {
                          final percentage = total > 0 ? (s.value / total * 100) : 0.0;
                          return PieChartSectionData(
                            value: s.value.toDouble(),
                            color: s.color,
                            title: percentage >= 5 ? '${percentage.toStringAsFixed(0)}%' : '',
                            titleStyle: const TextStyle(
                              color: Colors.white,
                              fontWeight: FontWeight.bold,
                              fontSize: 12,
                            ),
                            radius: 60,
                          );
                        }).toList(),
                      ),
                    ),
                  ),
                  const SizedBox(width: 16),
                  Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: sections.map((s) => _buildLegendItem(s, total)).toList(),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildLegendItem(_PieSection section, int total) {
    final percentage = total > 0 ? (section.value / total * 100) : 0.0;

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            width: 12,
            height: 12,
            decoration: BoxDecoration(color: section.color, shape: BoxShape.circle),
          ),
          const SizedBox(width: 8),
          Text(
            '${section.label} (${section.value})',
            style: Theme.of(context).textTheme.bodySmall,
          ),
        ],
      ),
    );
  }

  Widget _buildLatencyCard() {
    final analysis = _analysis!;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Latency Statistics', style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 16),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceAround,
              children: [
                _buildLatencyMetric('Min', '${analysis.minLatencyUs}µs', Colors.green),
                _buildLatencyMetric('Avg', '${analysis.avgLatencyUs}µs', Colors.blue),
                _buildLatencyMetric('Max', '${analysis.maxLatencyUs}µs', Colors.orange),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildLatencyMetric(String label, String value, Color color) {
    return Column(
      children: [
        Text(label, style: Theme.of(context).textTheme.bodySmall),
        const SizedBox(height: 4),
        Text(
          value,
          style: Theme.of(context).textTheme.titleLarge?.copyWith(color: color),
        ),
      ],
    );
  }

  String _formatDuration(double ms) {
    if (ms < 1000) return '${ms.toStringAsFixed(0)}ms';
    final seconds = ms / 1000;
    if (seconds < 60) return '${seconds.toStringAsFixed(1)}s';
    final minutes = seconds / 60;
    return '${minutes.toStringAsFixed(1)}m';
  }

  String _formatDate(String isoDate) {
    try {
      final date = DateTime.parse(isoDate);
      return '${date.month}/${date.day}';
    } catch (_) {
      return isoDate;
    }
  }
}

/// Helper class for pie chart sections.
class _PieSection {
  const _PieSection(this.label, this.value, this.color);

  final String label;
  final int value;
  final Color color;
}

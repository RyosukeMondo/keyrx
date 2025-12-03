/// Replay page for session replay with playback controls.
///
/// Provides session list, metadata display, play/pause/stop controls,
/// speed slider, verify mode toggle, and mismatch highlighting.
library;

import 'package:flutter/material.dart';

import '../../services/session_service.dart';

/// Replay page for replaying and verifying recorded sessions.
class ReplayPage extends StatefulWidget {
  const ReplayPage({super.key, required this.sessionService});

  final SessionService sessionService;

  @override
  State<ReplayPage> createState() => _ReplayPageState();
}

class _ReplayPageState extends State<ReplayPage> {
  List<SessionRecord> _sessions = [];
  SessionRecord? _selectedSession;
  SessionReplayData? _replayResult;
  bool _isLoading = false;
  bool _isReplaying = false;
  bool _verifyMode = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadSessions();
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

  Future<void> _replaySession() async {
    if (_selectedSession == null) return;

    setState(() {
      _isReplaying = true;
      _error = null;
      _replayResult = null;
    });

    final result = await widget.sessionService.replay(
      _selectedSession!.path,
      verify: _verifyMode,
    );

    setState(() {
      _isReplaying = false;
      if (result.hasError) {
        _error = result.errorMessage;
      } else {
        _replayResult = result.data;
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Session Replay'),
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
          // Replay panel
          Expanded(
            child: _buildReplayPanel(),
          ),
        ],
      ),
    );
  }

  Widget _buildSessionList() {
    if (_isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_sessions.isEmpty) {
      return _buildEmptyState(
        icon: Icons.folder_open,
        title: 'No sessions found',
        subtitle: 'Record a session to replay it here',
      );
    }

    return ListView.builder(
      itemCount: _sessions.length,
      itemBuilder: (context, index) {
        final session = _sessions[index];
        final isSelected = _selectedSession?.path == session.path;

        return ListTile(
          selected: isSelected,
          leading: const Icon(Icons.play_circle_outline),
          title: Text(session.name, maxLines: 1, overflow: TextOverflow.ellipsis),
          subtitle: Text(
            '${session.eventCount} events • ${_formatDuration(session.durationMs)}',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          onTap: () => setState(() {
            _selectedSession = session;
            _replayResult = null;
            _error = null;
          }),
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

  Widget _buildReplayPanel() {
    if (_selectedSession == null) {
      return _buildEmptyState(
        icon: Icons.play_circle_outline,
        title: 'Select a session',
        subtitle: 'Choose a session from the list to replay',
      );
    }

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _buildSessionInfoCard(),
          const SizedBox(height: 16),
          _buildControlsCard(),
          const SizedBox(height: 16),
          if (_error != null) _buildErrorBanner(),
          if (_replayResult != null) _buildResultCard(),
        ],
      ),
    );
  }

  Widget _buildSessionInfoCard() {
    final session = _selectedSession!;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Session Info', style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 12),
            _buildInfoRow('Name', session.name),
            _buildInfoRow('Events', '${session.eventCount}'),
            _buildInfoRow('Duration', _formatDuration(session.durationMs)),
            _buildInfoRow('Created', _formatDate(session.created)),
          ],
        ),
      ),
    );
  }

  Widget _buildInfoRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: Theme.of(context).textTheme.bodySmall),
          Text(value, style: const TextStyle(fontWeight: FontWeight.w500)),
        ],
      ),
    );
  }

  Widget _buildControlsCard() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Playback Controls', style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 16),
            SwitchListTile(
              title: const Text('Verify Mode'),
              subtitle: const Text('Compare replay output against recorded decisions'),
              value: _verifyMode,
              onChanged: _isReplaying ? null : (value) => setState(() => _verifyMode = value),
            ),
            const SizedBox(height: 16),
            SizedBox(
              width: double.infinity,
              child: FilledButton.icon(
                onPressed: _isReplaying ? null : _replaySession,
                icon: _isReplaying
                    ? const SizedBox(
                        width: 16,
                        height: 16,
                        child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white),
                      )
                    : const Icon(Icons.play_arrow),
                label: Text(_isReplaying ? 'Replaying...' : 'Replay Session'),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildErrorBanner() {
    return Card(
      color: Theme.of(context).colorScheme.errorContainer,
      margin: const EdgeInsets.only(bottom: 16),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Row(
          children: [
            Icon(Icons.error, color: Theme.of(context).colorScheme.error),
            const SizedBox(width: 12),
            Expanded(
              child: Text(
                _error!,
                style: TextStyle(color: Theme.of(context).colorScheme.onErrorContainer),
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

  Widget _buildResultCard() {
    final result = _replayResult!;
    final success = result.success;

    return Card(
      color: success ? Colors.green.shade50 : Colors.red.shade50,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(
                  success ? Icons.check_circle : Icons.cancel,
                  color: success ? Colors.green : Colors.red,
                  size: 32,
                ),
                const SizedBox(width: 12),
                Text(
                  success ? 'Replay Successful' : 'Mismatches Detected',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ],
            ),
            const SizedBox(height: 16),
            _buildResultRow('Total Events', '${result.totalEvents}'),
            _buildResultRow('Matched', '${result.matched}', color: Colors.green),
            _buildResultRow('Mismatched', '${result.mismatched}', color: result.mismatched > 0 ? Colors.red : null),
            if (result.mismatches.isNotEmpty) ...[
              const Divider(height: 24),
              Text('Mismatches', style: Theme.of(context).textTheme.titleSmall),
              const SizedBox(height: 8),
              ...result.mismatches.take(10).map(_buildMismatchTile),
              if (result.mismatches.length > 10)
                Padding(
                  padding: const EdgeInsets.only(top: 8),
                  child: Text(
                    '...and ${result.mismatches.length - 10} more',
                    style: TextStyle(color: Colors.grey[600], fontStyle: FontStyle.italic),
                  ),
                ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildResultRow(String label, String value, {Color? color}) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: Theme.of(context).textTheme.bodySmall),
          Text(
            value,
            style: TextStyle(fontWeight: FontWeight.w500, color: color),
          ),
        ],
      ),
    );
  }

  Widget _buildMismatchTile(SessionReplayMismatch mismatch) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.red.shade100,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('Event #${mismatch.seq}', style: const TextStyle(fontWeight: FontWeight.bold)),
          const SizedBox(height: 4),
          Text('Expected: ${mismatch.recorded}', style: Theme.of(context).textTheme.bodySmall),
          Text('Actual: ${mismatch.actual}', style: Theme.of(context).textTheme.bodySmall),
        ],
      ),
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
      return '${date.month}/${date.day} ${date.hour}:${date.minute.toString().padLeft(2, '0')}';
    } catch (_) {
      return isoDate;
    }
  }
}

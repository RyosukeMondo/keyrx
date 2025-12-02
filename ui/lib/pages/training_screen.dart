import 'dart:async';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../services/audio_service.dart';
import '../services/error_translator.dart';
import '../services/service_registry.dart';
import '../state/app_state.dart';
import '../ui/styles/surfaces.dart';
import '../ui/widgets/app_error_dialog.dart';
import '../ui/widgets/loading_overlay.dart';

/// Screen that wires the service layer into a simple training flow.
class TrainingScreen extends StatefulWidget {
  const TrainingScreen({super.key});

  @override
  State<TrainingScreen> createState() => _TrainingScreenState();
}

class _TrainingScreenState extends State<TrainingScreen> {
  late final TextEditingController _bpmController;
  late AudioService _audioService;
  late ServiceRegistry _registry;

  StreamSubscription<ClassificationResult>? _classificationSub;
  List<ClassificationResult> _recentResults = [];
  AudioState _state = AudioState.idle;
  bool _isLoading = false;
  String? _loadingMessage;

  @override
  void initState() {
    super.initState();
    _bpmController = TextEditingController(text: '120');
    _registry = Provider.of<ServiceRegistry>(context, listen: false);
    _audioService = _registry.audioService;
    _state = _audioService.state;
    _attachClassificationStream();
  }

  void _attachClassificationStream() {
    _classificationSub?.cancel();
    _classificationSub = _audioService.classificationStream.listen(
      (event) {
        if (!mounted) return;
        setState(() {
          _recentResults = [event, ..._recentResults].take(15).toList();
        });
      },
      onError: (error, stackTrace) {
        if (!mounted) return;
        final message = _registry.errorTranslator.translate(error);
        _showMessage(message);
      },
    );
  }

  @override
  void dispose() {
    _classificationSub?.cancel();
    // Ensure we stop audio if user navigates away.
    unawaited(_audioService.stop());
    _bpmController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final appState = context.watch<AppState>();
    final isRunning = _state == AudioState.running;
    final isBusy = _isLoading || _state == AudioState.starting;

    return LoadingOverlay(
      isLoading: _isLoading,
      message: _loadingMessage,
      child: Scaffold(
        appBar: AppBar(
          title: const Text('Training'),
          actions: [
            Icon(
              isRunning ? Icons.check_circle : Icons.stop_circle_outlined,
              color: isRunning ? Colors.greenAccent : Colors.grey,
            ),
            const SizedBox(width: 16),
          ],
        ),
        body: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            SurfaceContainer(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  if (!appState.initialized || appState.error != null)
                    Padding(
                      padding: const EdgeInsets.only(bottom: 8.0),
                      child: Row(
                        children: [
                          Icon(
                            appState.initialized ? Icons.error_outline : Icons.info_outline,
                            color: Colors.amber,
                          ),
                          const SizedBox(width: 8),
                          Expanded(
                            child: Text(
                              appState.error ??
                                  'Engine not initialized yet. Some actions may fail.',
                              style: Theme.of(context).textTheme.bodyMedium,
                            ),
                          ),
                        ],
                      ),
                    ),
                  _buildStatusRow(),
                  const SizedBox(height: 16),
                  _buildBpmField(),
                  const SizedBox(height: 12),
                  Wrap(
                    spacing: 12,
                    runSpacing: 8,
                    children: [
                      FilledButton.icon(
                        icon: const Icon(Icons.play_arrow),
                        label: const Text('Start'),
                        onPressed: isBusy || isRunning ? null : _startAudio,
                      ),
                      OutlinedButton.icon(
                        icon: const Icon(Icons.stop),
                        label: const Text('Stop'),
                        onPressed: isBusy || !isRunning ? null : _stopAudio,
                      ),
                      TextButton.icon(
                        icon: const Icon(Icons.speed),
                        label: const Text('Apply BPM'),
                        onPressed: isBusy ? null : _applyBpm,
                      ),
                    ],
                  ),
                ],
              ),
            ),
            SurfaceContainer(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text(
                        'Recent Classifications',
                        style: Theme.of(context).textTheme.titleMedium,
                      ),
                      if (_recentResults.isNotEmpty)
                        IconButton(
                          tooltip: 'Clear',
                          onPressed: _clearResults,
                          icon: const Icon(Icons.delete_sweep_outlined),
                        ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  if (_recentResults.isEmpty)
                    const Text(
                      'Start audio to see live classification results.',
                    )
                  else
                    ..._recentResults.map(
                      (result) => _buildClassificationTile(result),
                    ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatusRow() {
    Color color;
    IconData icon;
    switch (_state) {
      case AudioState.running:
        color = Colors.greenAccent;
        icon = Icons.graphic_eq;
        break;
      case AudioState.starting:
        color = Colors.amber;
        icon = Icons.hourglass_top_outlined;
        break;
      case AudioState.stopping:
        color = Colors.amber.shade200;
        icon = Icons.hourglass_bottom_outlined;
        break;
      case AudioState.idle:
      default:
        color = Colors.grey;
        icon = Icons.pause_circle_outline;
    }

    return Row(
      children: [
        Icon(icon, color: color),
        const SizedBox(width: 8),
        Text(
          'Audio state: ${_state.name}',
          style: Theme.of(context).textTheme.titleMedium,
        ),
        const Spacer(),
        IconButton(
          tooltip: 'Refresh state',
          onPressed: () => setState(() {
            _state = _audioService.state;
          }),
          icon: const Icon(Icons.refresh),
        ),
      ],
    );
  }

  Widget _buildBpmField() {
    return Row(
      children: [
        Expanded(
          child: TextField(
            controller: _bpmController,
            keyboardType: TextInputType.number,
            decoration: const InputDecoration(
              labelText: 'Target BPM',
              helperText: 'Positive integer (e.g., 120)',
            ),
            enabled: !_isLoading,
          ),
        ),
        const SizedBox(width: 12),
        Chip(
          label: Text('State: ${_state.name}'),
          avatar: const Icon(Icons.speed_outlined),
        ),
      ],
    );
  }

  Widget _buildClassificationTile(ClassificationResult result) {
    return ListTile(
      dense: true,
      leading: const Icon(Icons.analytics_outlined),
      title: Text(result.label),
      subtitle: Text(
        'Confidence: ${(result.confidence * 100).toStringAsFixed(1)}% • '
        '${result.timestamp.toLocal()}',
      ),
    );
  }

  Future<void> _startAudio() async {
    final bpm = int.tryParse(_bpmController.text);
    if (bpm == null || bpm <= 0) {
      _showMessage(
        const UserMessage(
          title: 'Invalid BPM',
          body: 'Enter a whole number greater than zero.',
          category: MessageCategory.warning,
        ),
      );
      return;
    }

    setState(() {
      _isLoading = true;
      _loadingMessage = 'Starting audio...';
      _state = AudioState.starting;
    });

    final result = await _audioService.start(bpm: bpm);
    if (!mounted) return;

    setState(() {
      _isLoading = false;
      _loadingMessage = null;
      _state = _audioService.state;
    });

    if (!result.success && result.userMessage != null) {
      _showMessage(result.userMessage!);
    }
  }

  Future<void> _stopAudio() async {
    setState(() {
      _isLoading = true;
      _loadingMessage = 'Stopping audio...';
      _state = AudioState.stopping;
    });

    final result = await _audioService.stop();
    if (!mounted) return;

    setState(() {
      _isLoading = false;
      _loadingMessage = null;
      _state = _audioService.state;
    });

    if (!result.success && result.userMessage != null) {
      _showMessage(result.userMessage!);
    }
  }

  Future<void> _applyBpm() async {
    final bpm = int.tryParse(_bpmController.text);
    if (bpm == null || bpm <= 0) {
      _showMessage(
        const UserMessage(
          title: 'Invalid BPM',
          body: 'Enter a whole number greater than zero.',
          category: MessageCategory.warning,
        ),
      );
      return;
    }

    final result = await _audioService.setBpm(bpm);
    if (!result.success && mounted && result.userMessage != null) {
      _showMessage(result.userMessage!);
    }
  }

  void _clearResults() {
    setState(() {
      _recentResults = [];
    });
  }

  void _showMessage(UserMessage message) {
    final icon = switch (message.category) {
      MessageCategory.info => Icons.info_outline,
      MessageCategory.warning => Icons.warning_amber_rounded,
      MessageCategory.error => Icons.error_outline,
    };

    AppErrorDialog.show(
      context,
      title: message.title,
      message: message.body,
      icon: icon,
    );
  }
}

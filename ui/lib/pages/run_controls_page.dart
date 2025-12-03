/// Central engine control panel for starting/stopping the engine.
///
/// Provides device selection, script selection, recording toggle,
/// and prominent start/stop controls with status indicators.
library;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../ffi/bridge.dart';
import '../services/device_service.dart';
import '../state/app_state.dart';
import 'run_controls_widgets.dart';

/// Engine running state.
enum EngineRunState { stopped, starting, running, stopping }

/// Central page for engine run controls.
class RunControlsPage extends StatefulWidget {
  const RunControlsPage({
    super.key,
    required this.deviceService,
    required this.bridge,
  });

  final DeviceService deviceService;
  final KeyrxBridge bridge;

  @override
  State<RunControlsPage> createState() => _RunControlsPageState();
}

class _RunControlsPageState extends State<RunControlsPage> {
  List<KeyboardDevice> _devices = [];
  KeyboardDevice? _selectedDevice;
  EngineRunState _runState = EngineRunState.stopped;
  bool _isRecording = false;
  String? _recordingPath;
  bool _isLoadingDevices = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadDevices();
  }

  Future<void> _loadDevices() async {
    setState(() {
      _isLoadingDevices = true;
      _error = null;
    });

    try {
      final devices = await widget.deviceService.listDevices();
      setState(() {
        _devices = devices;
        _isLoadingDevices = false;
        if (devices.isNotEmpty && _selectedDevice == null) {
          _selectedDevice = devices.first;
        }
      });
    } catch (e) {
      setState(() {
        _error = e.toString();
        _isLoadingDevices = false;
      });
    }
  }

  Future<void> _toggleEngine() async {
    if (_runState == EngineRunState.running) {
      await _stopEngine();
    } else if (_runState == EngineRunState.stopped) {
      await _startEngine();
    }
  }

  Future<void> _startEngine() async {
    if (_selectedDevice == null) {
      _showSnackBar('Please select a device first', isError: true);
      return;
    }

    final appState = context.read<AppState>();
    if (!appState.initialized) {
      _showSnackBar('Engine not initialized', isError: true);
      return;
    }

    setState(() => _runState = EngineRunState.starting);

    // Select the device
    final result = await widget.deviceService.selectDevice(_selectedDevice!.path);
    if (!result.success) {
      setState(() => _runState = EngineRunState.stopped);
      if (mounted) {
        _showSnackBar('Failed to select device: ${result.errorMessage}', isError: true);
      }
      return;
    }

    // For now, simulate engine start (actual engine run would be via FFI)
    await Future.delayed(const Duration(milliseconds: 500));

    if (mounted) {
      setState(() => _runState = EngineRunState.running);
      _showSnackBar('Engine started');
    }
  }

  Future<void> _stopEngine() async {
    setState(() => _runState = EngineRunState.stopping);

    // Stop recording if active
    if (_isRecording) {
      await _stopRecording();
    }

    // Simulate engine stop
    await Future.delayed(const Duration(milliseconds: 300));

    if (mounted) {
      setState(() => _runState = EngineRunState.stopped);
      _showSnackBar('Engine stopped');
    }
  }

  Future<void> _toggleRecording() async {
    if (_isRecording) {
      await _stopRecording();
    } else {
      await _startRecording();
    }
  }

  Future<void> _startRecording() async {
    final timestamp = DateTime.now().toIso8601String().replaceAll(':', '-');
    final path = 'sessions/session_$timestamp.krx';

    final result = widget.bridge.startRecording(path);
    if (result.hasError) {
      _showSnackBar('Failed to start recording: ${result.errorMessage}', isError: true);
      return;
    }

    setState(() {
      _isRecording = true;
      _recordingPath = result.outputPath ?? path;
    });
    _showSnackBar('Recording started');
  }

  Future<void> _stopRecording() async {
    final result = widget.bridge.stopRecording();
    if (result.hasError) {
      _showSnackBar('Failed to stop recording: ${result.errorMessage}', isError: true);
      return;
    }

    final path = result.path;
    final events = result.eventCount;
    setState(() {
      _isRecording = false;
      _recordingPath = null;
    });
    _showSnackBar('Recording saved: $events events to $path');
  }

  void _showSnackBar(String message, {bool isError = false}) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(message),
        backgroundColor: isError ? Colors.red : null,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final appState = context.watch<AppState>();
    final isRunning = _runState == EngineRunState.running;
    final isBusy = _runState == EngineRunState.starting ||
        _runState == EngineRunState.stopping;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Run Controls'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _loadDevices,
            tooltip: 'Refresh devices',
          ),
        ],
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            _buildStatusCard(appState, isRunning),
            const SizedBox(height: 24),
            _buildDeviceSelector(),
            const SizedBox(height: 16),
            _buildScriptInfo(appState),
            const SizedBox(height: 16),
            _buildRecordingToggle(),
            const SizedBox(height: 32),
            _buildStartStopButton(isRunning, isBusy),
          ],
        ),
      ),
    );
  }

  Widget _buildStatusCard(AppState appState, bool isRunning) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Status',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 12),
            StatusIndicator(
              icon: Icons.power_settings_new,
              label: 'Engine',
              value: _runState.name,
              isActive: isRunning,
            ),
            const SizedBox(height: 8),
            StatusIndicator(
              icon: Icons.keyboard,
              label: 'Device',
              value: _selectedDevice?.name ?? 'None selected',
              isActive: _selectedDevice != null,
            ),
            const SizedBox(height: 8),
            StatusIndicator(
              icon: Icons.description,
              label: 'Script',
              value: appState.loadedScript ?? 'None loaded',
              isActive: appState.loadedScript != null,
            ),
            const SizedBox(height: 8),
            StatusIndicator(
              icon: Icons.fiber_manual_record,
              label: 'Recording',
              value: _isRecording ? 'Active' : 'Off',
              isActive: _isRecording,
              activeColor: Colors.red,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildDeviceSelector() {
    if (_isLoadingDevices) {
      return const LoadingCard(message: 'Loading devices...');
    }

    if (_error != null) {
      return ErrorCard(error: _error!, onRetry: _loadDevices);
    }

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.keyboard, size: 20),
                const SizedBox(width: 8),
                Text(
                  'Device',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ],
            ),
            const SizedBox(height: 12),
            if (_devices.isEmpty)
              const Text(
                'No devices found. Connect a keyboard and refresh.',
                style: TextStyle(color: Colors.grey),
              )
            else
              DropdownButtonFormField<KeyboardDevice>(
                initialValue: _selectedDevice,
                decoration: const InputDecoration(
                  border: OutlineInputBorder(),
                  contentPadding: EdgeInsets.symmetric(
                    horizontal: 12,
                    vertical: 8,
                  ),
                ),
                items: _devices.map((device) {
                  return DropdownMenuItem(
                    value: device,
                    child: Row(
                      children: [
                        Expanded(child: Text(device.name)),
                        if (device.hasProfile)
                          Container(
                            margin: const EdgeInsets.only(left: 8),
                            padding: const EdgeInsets.symmetric(
                              horizontal: 6,
                              vertical: 2,
                            ),
                            decoration: BoxDecoration(
                              color: Colors.green.withValues(alpha: 0.15),
                              borderRadius: BorderRadius.circular(8),
                            ),
                            child: const Text(
                              'Profile',
                              style: TextStyle(
                                fontSize: 10,
                                color: Colors.green,
                              ),
                            ),
                          ),
                      ],
                    ),
                  );
                }).toList(),
                onChanged: (device) {
                  setState(() => _selectedDevice = device);
                },
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildScriptInfo(AppState appState) {
    final script = appState.loadedScript;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.description, size: 20),
                const SizedBox(width: 8),
                Text(
                  'Script',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ],
            ),
            const SizedBox(height: 12),
            if (script == null)
              const Text(
                'No script loaded. Use the Editor to load a script.',
                style: TextStyle(color: Colors.grey),
              )
            else
              Row(
                children: [
                  const Icon(Icons.check_circle, color: Colors.green, size: 16),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      script,
                      style: const TextStyle(fontFamily: 'monospace'),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                ],
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildRecordingToggle() {
    final isRunning = _runState == EngineRunState.running;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            Icon(
              Icons.fiber_manual_record,
              size: 20,
              color: _isRecording ? Colors.red : null,
            ),
            const SizedBox(width: 8),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'Recording',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  if (_isRecording && _recordingPath != null)
                    Text(
                      _recordingPath!,
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                            fontFamily: 'monospace',
                            color: Colors.grey,
                          ),
                    ),
                ],
              ),
            ),
            Switch(
              value: _isRecording,
              onChanged: isRunning ? (_) => _toggleRecording() : null,
              activeTrackColor: Colors.red.withValues(alpha: 0.5),
              activeThumbColor: Colors.red,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStartStopButton(bool isRunning, bool isBusy) {
    return StartStopButton(
      isRunning: isRunning,
      isBusy: isBusy,
      onPressed: _toggleEngine,
    );
  }
}

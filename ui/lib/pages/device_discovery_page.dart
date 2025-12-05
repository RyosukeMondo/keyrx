/// Wizard for creating a new device profile.
library;

import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../ffi/bridge.dart';
import '../models/discovery_progress.dart';
import '../services/facade/keyrx_facade.dart';
import '../services/facade/result.dart';
import '../services/service_registry.dart';

class DeviceDiscoveryPage extends StatefulWidget {
  const DeviceDiscoveryPage({
    super.key,
    required this.device,
    required this.facade,
    required this.services,
  });

  final KeyboardDevice device;
  final KeyrxFacade facade;
  final ServiceRegistry services;

  @override
  State<DeviceDiscoveryPage> createState() => _DeviceDiscoveryPageState();
}

class _DeviceDiscoveryPageState extends State<DeviceDiscoveryPage> {
  int _currentStep = 0;

  // Step 1: Layout Config
  List<int> _colsPerRow = [10]; // Default 1 row, 10 cols

  // Step 2: Mapping
  bool _isDiscovering = false;
  DiscoveryProgress? _progress;
  String? _discoveryError;
  final FocusNode _focusNode = FocusNode();

  // Step 3: Summary
  bool _isSaving = false;

  @override
  void dispose() {
    _focusNode.dispose();
    if (_isDiscovering) {
      widget.facade.cancelDiscovery();
    }
    super.dispose();
  }

  void _nextStep() {
    setState(() {
      _currentStep++;
    });
    if (_currentStep == 1) {
      _startDiscoverySession();
    }
  }

  void _prevStep() {
    if (_currentStep == 1) {
      widget.facade.cancelDiscovery();
      setState(() {
        _isDiscovering = false;
        _progress = null;
      });
    }
    setState(() {
      _currentStep--;
    });
  }

  Future<void> _startDiscoverySession() async {
    setState(() {
      _isDiscovering = true;
      _discoveryError = null;
      _progress = DiscoveryProgress(
        captured: 0,
        total: _colsPerRow.fold(0, (sum, c) => sum + c),
        nextKey: const DiscoveryPosition(row: 0, col: 0),
      );
    });

    final result = await widget.facade.startDiscovery(
      device: widget.device,
      rows: _colsPerRow.length,
      colsPerRow: _colsPerRow,
    );

    if (!mounted) return;

    result.when(
      ok: (_) {
        // Listen to progress
        widget.facade.discoveryProgress.listen((progress) {
          if (!mounted) return;
          setState(() {
            _progress = progress;
          });
          if (progress.isComplete) {
            // Auto-advance to summary
            setState(() {
              _currentStep = 2;
              _isDiscovering = false;
            });
          }
        });
        // Request focus to capture keys
        _focusNode.requestFocus();
      },
      err: (error) {
        setState(() {
          _discoveryError = error.userMessage;
          _isDiscovering = false;
        });
      },
    );
  }

  void _handleKeyEvent(KeyEvent event) {
    if (!_isDiscovering || event is! KeyDownEvent) return;

    // On Windows, scanCode is in physicalKey or logicalKey?
    // Flutter's KeyEvent unifies this, but we want the raw scan code if possible.
    // event.physicalKey.usbHidUsage might be useful, but on Windows we usually rely on scan code from platform.

    // Actually, we can use data.scanCode if we cast to specific platform events,
    // or assume the FFI expects what Flutter provides.
    // Core expects u16 scan code.

    int scanCode = 0;
    // We'll use a best-effort mapping or the raw code if available
    // In Flutter 3+, HardwareKeyboard provides cleaner events

    // Wait, `processDiscoveryEvent` takes `scanCode`.
    // On Windows, `RawKeyEvent.data` had `scanCode`.
    // `KeyEvent` (newer) might hide it.
    // Let's try to access it.

    if (event is KeyDownEvent) {
        // We need to get the scan code.
        // This might be tricky with just `KeyEvent`.
        // But wait, we can use `RawKeyboardListener` instead of `KeyboardListener` if we need raw access.
        // Or inspect `event.data` if we import `package:flutter/services.dart`.
        // `event` doesn't expose raw data easily in the base class.
        // But `KeyEvent` has `physicalKey`.
    }
  }

  // We use RawKeyboardListener for now to get platform-specific scan codes easily
  void _handleRawKeyEvent(RawKeyEvent event) {
    if (!_isDiscovering || event is! RawKeyDownEvent) return;

    int scanCode = 0;
    if (event.data is RawKeyEventDataWindows) {
      scanCode = (event.data as RawKeyEventDataWindows).scanCode;
    } else if (event.data is RawKeyEventDataLinux) {
      scanCode = (event.data as RawKeyEventDataLinux).scanCode;
    } else {
      // Fallback or skip
      return;
    }

    // Pass to FFI
    // Timestamp: microseconds
    final timestamp = DateTime.now().microsecondsSinceEpoch;
    widget.services.bridge.processDiscoveryEvent(scanCode, true, timestamp);
  }

  Future<void> _saveProfile() async {
    setState(() {
      _isSaving = true;
    });

    // The profile is currently held in the Rust session/summary state.
    // But wait, `save_device_profile` in FFI takes a JSON string.
    // Where do we get that JSON?
    // Ah, `DiscoverySummary` (from `capture_session` in CLI) contains it.
    // But in FFI, `process_discovery_event` returns 1 when finished.
    // Where is the result?

    // Looking at `core/src/ffi/domains/discovery.rs`, `process_discovery_event` returns 1 on completion.
    // And it publishes `SessionUpdate::Finished(summary)`.
    // The `discovery_sink` invokes `EventType.DiscoverySummary` callback with the summary.

    // I need to catch that summary JSON in the Facade or here!
    // `KeyrxFacadeImpl` only listens to `DiscoveryProgress`.
    // I should probably update Facade to also expose `discoveryResult` stream or Future.

    // For now, I can register a one-off callback here or in `KeyrxFacadeImpl`.
    // But `KeyrxFacadeImpl` is already designed.
    // Let's assume for now that once complete, the profile is READY in Rust memory?
    // No, `DiscoverySessionState` is cleared on finish.
    // But `SessionUpdate::Finished` carries the summary.

    // I need to capture that summary.
    // I'll add a listener for `discoverySummary` in `KeyrxFacadeImpl` and expose `Stream<DeviceProfile> discoveryComplete`.

    // Wait, I can't modify Facade right now without context switch cost.
    // I'll use `widget.services.bridge.registerEventCallback` directly here as a pragmatic solution
    // since `KeyrxFacade` doesn't expose the summary yet.

    // Wait, I missed `saveDeviceProfile` implementation details.
    // `saveDeviceProfile` takes a JSON.
    // If I get the JSON from the summary event, I can pass it to `saveDeviceProfile`.

    // So:
    // 1. Register for `EventType.discoverySummary`.
    // 2. When received, store the JSON.
    // 3. On "Save", call `saveDeviceProfile` with that JSON.

    // I'll do that in `_startDiscoverySession`.
  }

  String? _finalProfileJson;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Create Profile'),
      ),
      body: RawKeyboardListener(
        focusNode: _focusNode,
        onKey: _handleRawKeyEvent,
        autofocus: true,
        child: Stepper(
          type: StepperType.horizontal,
          currentStep: _currentStep,
          onStepContinue: _currentStep < 2 ? _nextStep : null,
          onStepCancel: _currentStep > 0 ? _prevStep : null,
          controlsBuilder: (context, details) {
            return Padding(
              padding: const EdgeInsets.only(top: 24.0),
              child: Row(
                children: [
                  if (_currentStep == 0)
                    FilledButton(
                      onPressed: details.onStepContinue,
                      child: const Text('Start Mapping'),
                    ),
                  if (_currentStep == 1)
                    FilledButton(
                      onPressed: null, // Disable manual continue, wait for discovery
                      child: const Text('Press Highlighted Key...'),
                    ),
                  if (_currentStep == 2)
                    FilledButton(
                      onPressed: _isSaving ? null : () async {
                        if (_finalProfileJson != null) {
                          setState(() => _isSaving = true);
                          final success = widget.services.bridge.saveDeviceProfile(_finalProfileJson!);
                          setState(() => _isSaving = false);

                          if (success) {
                            ScaffoldMessenger.of(context).showSnackBar(
                              const SnackBar(content: Text('Profile saved!')),
                            );
                            Navigator.of(context).pop(); // Return to devices list
                          } else {
                            ScaffoldMessenger.of(context).showSnackBar(
                              const SnackBar(content: Text('Failed to save profile'), backgroundColor: Colors.red),
                            );
                          }
                        }
                      },
                      child: _isSaving
                        ? const SizedBox(width: 20, height: 20, child: CircularProgressIndicator(strokeWidth: 2))
                        : const Text('Save Profile'),
                    ),

                  const SizedBox(width: 12),

                  if (_currentStep > 0 && _currentStep < 2)
                    TextButton(
                      onPressed: details.onStepCancel,
                      child: const Text('Cancel'),
                    ),
                ],
              ),
            );
          },
          steps: [
            Step(
              title: const Text('Layout'),
              content: _buildLayoutConfig(),
              isActive: _currentStep >= 0,
              state: _currentStep > 0 ? StepState.complete : StepState.indexed,
            ),
            Step(
              title: const Text('Map'),
              content: _buildMappingInterface(),
              isActive: _currentStep >= 1,
              state: _currentStep > 1 ? StepState.complete : (_isDiscovering ? StepState.editing : StepState.indexed),
            ),
            Step(
              title: const Text('Save'),
              content: _buildSaveSummary(),
              isActive: _currentStep >= 2,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildLayoutConfig() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'Configure your keyboard layout',
          style: Theme.of(context).textTheme.titleMedium,
        ),
        const SizedBox(height: 8),
        const Text('Add rows and specify how many keys are in each row.'),
        const SizedBox(height: 24),

        ListView.separated(
          shrinkWrap: true,
          physics: const NeverScrollableScrollPhysics(),
          itemCount: _colsPerRow.length,
          separatorBuilder: (_, __) => const SizedBox(height: 12),
          itemBuilder: (context, index) {
            return Row(
              children: [
                Text('Row ${index + 1}', style: const TextStyle(fontWeight: FontWeight.bold)),
                const SizedBox(width: 16),
                Expanded(
                  child: Column(
                    children: [
                      Slider(
                        value: _colsPerRow[index].toDouble(),
                        min: 1,
                        max: 30,
                        divisions: 29,
                        label: '${_colsPerRow[index]} keys',
                        onChanged: (val) {
                          setState(() {
                            _colsPerRow[index] = val.toInt();
                          });
                        },
                      ),
                    ],
                  ),
                ),
                const SizedBox(width: 16),
                SizedBox(
                  width: 40,
                  child: Text(
                    '${_colsPerRow[index]}',
                    textAlign: TextAlign.center,
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                ),
                IconButton(
                  icon: const Icon(Icons.delete_outline),
                  onPressed: _colsPerRow.length > 1
                    ? () => setState(() => _colsPerRow.removeAt(index))
                    : null,
                ),
              ],
            );
          },
        ),

        const SizedBox(height: 16),
        OutlinedButton.icon(
          onPressed: () => setState(() => _colsPerRow.add(10)),
          icon: const Icon(Icons.add),
          label: const Text('Add Row'),
        ),

        const SizedBox(height: 24),
        Text('Preview:', style: Theme.of(context).textTheme.titleSmall),
        const SizedBox(height: 8),
        _buildKeyboardPreview(),
      ],
    );
  }

  Widget _buildMappingInterface() {
    if (_discoveryError != null) {
      return Center(
        child: Column(
          children: [
            const Icon(Icons.error_outline, color: Colors.red, size: 48),
            const SizedBox(height: 16),
            Text('Discovery Error: $_discoveryError'),
          ],
        ),
      );
    }

    final next = _progress?.nextKey;
    final progress = _progress?.progress ?? 0.0;

    return Column(
      children: [
        Text(
          next != null
            ? 'Press the highlighted key'
            : 'Discovery complete!',
          style: Theme.of(context).textTheme.headlineSmall,
        ),
        const SizedBox(height: 8),
        if (next != null)
          Text(
            'Row ${next.row + 1}, Column ${next.col + 1}',
            style: Theme.of(context).textTheme.titleMedium?.copyWith(
              color: Theme.of(context).colorScheme.primary,
            ),
          ),
        const SizedBox(height: 24),
        LinearProgressIndicator(value: progress),
        const SizedBox(height: 8),
        Text('${(_progress?.captured ?? 0)} / ${(_progress?.total ?? 0)} keys mapped'),
        const SizedBox(height: 32),
        _buildKeyboardPreview(highlight: next),
      ],
    );
  }

  Widget _buildSaveSummary() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.check_circle_outline, color: Colors.green, size: 64),
          const SizedBox(height: 24),
          Text(
            'Profile Created!',
            style: Theme.of(context).textTheme.headlineMedium,
          ),
          const SizedBox(height: 16),
          Text(
            'Device: ${widget.device.name}',
            style: Theme.of(context).textTheme.titleMedium,
          ),
          Text(
            'Layout: ${_colsPerRow.length} rows, ${_colsPerRow.fold(0, (a, b) => a + b)} keys',
          ),
          const SizedBox(height: 32),
          const Text('Click Save to write this profile to disk.'),
        ],
      ),
    );
  }

  Widget _buildKeyboardPreview({DiscoveryPosition? highlight}) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        border: Border.all(color: Colors.grey.withOpacity(0.3)),
        borderRadius: BorderRadius.circular(8),
        color: Colors.grey.withOpacity(0.05),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          for (int r = 0; r < _colsPerRow.length; r++)
            Padding(
              padding: const EdgeInsets.only(bottom: 8),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  for (int c = 0; c < _colsPerRow[r]; c++)
                    _buildKey(r, c, highlight),
                ],
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildKey(int row, int col, DiscoveryPosition? highlight) {
    final isNext = highlight != null && highlight.row == row && highlight.col == col;
    final isMapped = _progress != null && (
      _progress!.nextKey == null || // All mapped
      (row < _progress!.nextKey!.row) || // Previous row
      (row == _progress!.nextKey!.row && col < _progress!.nextKey!.col) // Previous col in same row
    );

    Color color = Colors.grey.shade300;
    if (isNext) color = Colors.blue; // Active target
    else if (isMapped) color = Colors.green; // Mapped

    return Container(
      width: 32,
      height: 32,
      margin: const EdgeInsets.symmetric(horizontal: 2),
      decoration: BoxDecoration(
        color: color,
        borderRadius: BorderRadius.circular(4),
        border: isNext ? Border.all(color: Colors.blue.shade800, width: 2) : null,
        boxShadow: isNext ? [BoxShadow(color: Colors.blue.withOpacity(0.4), blurRadius: 8, spreadRadius: 2)] : null,
      ),
      child: Center(
        child: Text(
          '', // Could show key label if we knew it, but we don't yet
          style: const TextStyle(fontSize: 10),
        ),
      ),
    );
  }

  @override
  void initState() {
    super.initState();
    // Register for summary event
    widget.services.bridge.registerEventCallback(
      EventType.discoverySummary,
      (Uint8List payload) {
        try {
          final jsonStr = utf8.decode(payload);
          // The payload IS the profile JSON (or summary containing profile)
          // Rust: publish_session_update(&SessionUpdate::Finished(summary));
          // Summary serializes to { ..., profile: ... }

          // Let's assume the payload is what we need to pass to saveDeviceProfile.
          // Or we extract the profile from it.
          // `DiscoverySummary` struct has `profile: DeviceProfile`.
          final jsonMap = json.decode(jsonStr) as Map<String, dynamic>;

          // Extract profile part if nested
          if (jsonMap.containsKey('profile')) {
             _finalProfileJson = json.encode(jsonMap['profile']);
          } else {
             _finalProfileJson = jsonStr;
          }
        } catch (e) {
          print('Error parsing summary: $e');
        }
      },
    );
  }
}

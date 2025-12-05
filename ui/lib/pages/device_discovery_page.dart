/// Wizard for creating a new device profile.
library;

import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../ffi/bridge.dart';
import '../models/discovery_progress.dart';
import '../models/key_codes_windows.dart';
import '../models/keyboard_layout.dart';
import '../services/facade/keyrx_facade.dart';
import '../services/facade/result.dart';
import '../services/service_registry.dart';
import '../widgets/visual_keyboard.dart';
import '../services/device_profile_service.dart';

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

  // Manual input state
  bool _showManualInput = false;
  final Map<String, VisualKeyOverride> _visualOverrides = {};
  final Map<String, String> _mappedLabels = {}; // Key ID -> Label (e.g., "A", "Esc")
  String? _flashKeyId; // Key ID to flash (for duplicates)
  Timer? _flashTimer;

  // Input trap
  final TextEditingController _textController = TextEditingController();
  late FocusNode _textFocusNode;

  @override
  void initState() {
    super.initState();

    // Initialize focus node with key handler attached directly
    _textFocusNode = FocusNode(
      onKey: (node, event) {
        if (event is RawKeyEvent) {
           _handleRawKeyEvent(event);
           return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
    );

    widget.services.bridge.registerEventCallback(
      EventType.discoveryDuplicate,
      (Uint8List payload) {
         try {
           final jsonStr = utf8.decode(payload);
           final jsonMap = json.decode(jsonStr) as Map<String, dynamic>;
           // Payload: { "scan_code": 123, "existing": {"row": 0, "col": 0}, "attempted": ... }
           if (jsonMap['existing'] != null) {
             final existing = jsonMap['existing'];
             final row = existing['row'];
             final col = existing['col'];
             final keyId = 'r${row}_c${col}';

             if (mounted) {
               setState(() {
                 _flashKeyId = keyId;
               });
               _flashTimer?.cancel();
               _flashTimer = Timer(const Duration(milliseconds: 500), () {
                 if (mounted) {
                   setState(() {
                     _flashKeyId = null;
                   });
                 }
               });

               ScaffoldMessenger.of(context).hideCurrentSnackBar();
               ScaffoldMessenger.of(context).showSnackBar(
                 SnackBar(
                   content: Text('Duplicate key! Already mapped at Row ${row + 1}, Col ${col + 1}'),
                   backgroundColor: Colors.orange,
                   duration: const Duration(seconds: 1),
                 ),
               );
             }
           }
         } catch (e) {
           print('Error parsing duplicate: $e');
         }
      },
    );

    // Register for summary event
    widget.services.bridge.registerEventCallback(
      EventType.discoverySummary,
      (Uint8List payload) {
        try {
          final jsonStr = utf8.decode(payload);
          final jsonMap = json.decode(jsonStr) as Map<String, dynamic>;

          // Extract profile part if nested or construct it from summary
          if (jsonMap.containsKey('profile')) {
             _finalProfileJson = json.encode(jsonMap['profile']);
          } else {
             // The summary JSON structure doesn't match DeviceProfile structure exactly.
             // We need to transform it to match what saveDeviceProfile expects.
             // Summary has: device_id: {vendor_id, product_id}, rows, cols_per_row, keymap, aliases...
             // Profile needs: vendor_id, product_id, schema_version, discovered_at, source, ...

             final deviceId = jsonMap['device_id'] as Map<String, dynamic>;
             final vendorId = deviceId['vendor_id'];
             final productId = deviceId['product_id'];

             final profileMap = {
               'schema_version': 1,
               'vendor_id': vendorId,
               'product_id': productId,
               'name': widget.device.name,
               'discovered_at': DateTime.now().toUtc().toIso8601String(),
               'rows': jsonMap['rows'],
               'cols_per_row': jsonMap['cols_per_row'],
               'keymap': jsonMap['keymap'],
               'aliases': jsonMap['aliases'],
               'source': 'Discovered',
             };

             _finalProfileJson = json.encode(profileMap);
          }
        } catch (e) {
          print('Error parsing summary: $e');
        }
      },
    );

    _flashTimer?.cancel();
    _focusNode.dispose();
    if (_isDiscovering) {
      widget.facade.cancelDiscovery();
    }
  }

  @override
  void dispose() {
    _textController.dispose();
    _textFocusNode.dispose();
    _flashTimer?.cancel();
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
        _textFocusNode.requestFocus();
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

    // Store label for the current target key position if known
    if (_progress?.nextKey != null) {
      final next = _progress!.nextKey!;
      final keyId = 'r${next.row}_c${next.col}';

      // Try to find a label for this scan code
      String label = '0x${scanCode.toRadixString(16).toUpperCase()}';
      // Simple reverse lookup in key_codes_windows.dart if we had it,
      // but we only have logical key maps usually.
      // Let's try to get the logical key label from the event itself
      if (event.logicalKey.keyLabel.isNotEmpty) {
        label = event.logicalKey.keyLabel.toUpperCase();
      }

      setState(() {
        _mappedLabels[keyId] = label;
      });
    }

    // Pass to FFI
    // Timestamp: microseconds
    final timestamp = DateTime.now().microsecondsSinceEpoch;
    widget.services.bridge.processDiscoveryEvent(scanCode, true, timestamp);
  }

  void _skipCurrentKey() {
    final success = widget.services.bridge.skipDiscoveryKey();
    if (success) {
       // The progress update will move to next key
       // We can mark the current (now skipped) key in our visual overrides if needed?
       // Yes, we should mark it as "skipped" for visual rendering later.
       if (_progress?.nextKey != null) {
         final keyId = 'r${_progress!.nextKey!.row}_c${_progress!.nextKey!.col}';
         _updateVisualOverride(keyId, isSkipped: true);
       }
    }
    // Keep focus on input trap
    _textFocusNode.requestFocus();
  }

  void _undoLastMapping() {
    // Determine which key we are undoing to clear its visual state
    if (_progress != null && _progress!.captured > 0) {
      final prevIndex = _progress!.captured - 1;
      final prevPos = _indexToPosition(prevIndex);
      if (prevPos != null) {
        final keyId = 'r${prevPos.row}_c${prevPos.col}';
        // Clear the skipped flag if it was skipped, so it appears active again
        _updateVisualOverride(keyId, isSkipped: false);
      }
    }

    widget.services.bridge.undoDiscoveryMapping();
    // Keep focus on input trap
    _textFocusNode.requestFocus();
  }

  DiscoveryPosition? _indexToPosition(int index) {
    int current = 0;
    for (int r = 0; r < _colsPerRow.length; r++) {
      int cols = _colsPerRow[r];
      if (index < current + cols) {
        return DiscoveryPosition(row: r, col: index - current);
      }
      current += cols;
    }
    return null;
  }

  void _updateVisualOverride(String keyId, {double? width, bool? isSkipped}) {
    setState(() {
      final current = _visualOverrides[keyId] ?? const VisualKeyOverride();
      _visualOverrides[keyId] = VisualKeyOverride(
        width: width ?? current.width,
        isSkipped: isSkipped ?? current.isSkipped,
      );
    });
  }

  void _processScanCode(int scanCode) {
    final timestamp = DateTime.now().microsecondsSinceEpoch;
    widget.services.bridge.processDiscoveryEvent(scanCode, true, timestamp);
  }

  void _handleManualKey(KeyDefinition key) {
    final scanCode = keyIdToWindowsScanCode[key.id];
    if (scanCode != null) {
      _processScanCode(scanCode);
    } else {
      // Fallback or error for unknown keys
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('No scan code mapping for ${key.label}')),
      );
    }
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
      body: Stepper(
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

                          print('Attempting to save profile: $_finalProfileJson');

                          // Save profile
                          bool success = false;
                          try {
                            final profileMap = json.decode(_finalProfileJson!) as Map<String, dynamic>;
                            final profile = DeviceProfile.fromJson(profileMap);
                            await widget.services.deviceProfileService.saveProfile(profile, setActive: true);
                            success = true;
                          } catch (e) {
                            print('Error saving profile: $e');
                          }

                          print('saveProfile result: $success');

                          // Save visual overrides
                          if (success) {
                            await widget.services.deviceProfileService.saveVisualOverrides(
                              widget.device.vendorId,
                              widget.device.productId,
                              _visualOverrides,
                            );
                          }

                          if (mounted) {
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
        // Input Trap: Visible TextField to capture input without IME
        Container(
          margin: const EdgeInsets.only(bottom: 16),
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          decoration: BoxDecoration(
            color: Colors.blue.withOpacity(0.1),
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: Colors.blue.withOpacity(0.3)),
          ),
          child: Column(
            children: [
              const Text(
                'Click inside the box below to ensure keys are captured:',
                style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold),
              ),
              const SizedBox(height: 8),
              TextField(
                controller: _textController,
                focusNode: _textFocusNode,
                autofocus: true,
                readOnly: true, // Disables IME composition
                textAlign: TextAlign.center,
                decoration: const InputDecoration(
                  hintText: 'Focus here & press keys on device',
                  border: OutlineInputBorder(),
                  filled: true,
                  fillColor: Colors.white,
                  isDense: true,
                ),
              ),
            ],
          ),
        ),
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
        const SizedBox(height: 32),
        if (next != null) ...[
          const SizedBox(height: 16),
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              FilledButton.icon(
                onPressed: _skipCurrentKey,
                icon: const Icon(Icons.skip_next),
                label: const Text('Skip (Blank)'),
                style: FilledButton.styleFrom(
                  backgroundColor: Colors.grey.shade700,
                ),
              ),
              const SizedBox(width: 16),
              FilledButton.icon(
                onPressed: (_progress?.captured ?? 0) > 0 ? _undoLastMapping : null,
                icon: const Icon(Icons.undo),
                label: const Text('Back'),
                style: FilledButton.styleFrom(
                  backgroundColor: Colors.orange.shade700,
                ),
              ),
              const SizedBox(width: 16),
              OutlinedButton.icon(
                onPressed: () {
                  setState(() {
                    _showManualInput = !_showManualInput;
                  });
                },
                icon: Icon(_showManualInput ? Icons.keyboard_hide : Icons.keyboard),
                label: Text(_showManualInput ? 'Hide Visual Keyboard' : 'Cannot press key?'),
              ),
            ],
          ),
          if (_showManualInput) ...[
            const SizedBox(height: 16),
            const Text(
              'Click a key below to map it to the current position:',
              style: TextStyle(fontStyle: FontStyle.italic),
            ),
            const SizedBox(height: 8),
            Container(
              height: 300,
              decoration: BoxDecoration(
                border: Border.all(color: Colors.grey.withOpacity(0.3)),
                borderRadius: BorderRadius.circular(8),
              ),
              child: VisualKeyboard(
                layout: KeyboardLayout.full(unitSize: 36),
                enabled: true,
                showMappingOverlay: false,
                enableDragDrop: false,
                showSecondaryLabels: false,
                onKeyTap: _handleManualKey,
              ),
            ),
          ],
        ],
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

    final keyId = 'r${row}_c${col}';
    final override = _visualOverrides[keyId];
    final width = override?.width ?? 1.0;
    final isSkipped = override?.isSkipped ?? false;
    final label = _mappedLabels[keyId] ?? '';
    final isFlashing = _flashKeyId == keyId;

    Color color = Colors.grey.shade300;
    if (isFlashing) color = Colors.orangeAccent; // Flashing duplicate
    else if (isNext) color = Colors.blue; // Active target
    else if (isMapped) color = isSkipped ? Colors.grey.shade400 : Colors.green; // Mapped

    // Base unit size (e.g., 32px) times width factor
    final pixelWidth = 32.0 * width;

    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: pixelWidth,
          height: 32,
          margin: const EdgeInsets.symmetric(horizontal: 2),
          decoration: BoxDecoration(
            color: color,
            borderRadius: BorderRadius.circular(4),
            border: isNext ? Border.all(color: Colors.blue.shade800, width: 2) : null,
            boxShadow: isNext ? [BoxShadow(color: Colors.blue.withOpacity(0.4), blurRadius: 8, spreadRadius: 2)] : null,
          ),
          child: Stack(
            children: [
               Center(
                child: Text(
                  isSkipped ? 'SKIP' : label,
                  style: TextStyle(
                    fontSize: isSkipped ? 8 : 10,
                    fontWeight: isSkipped ? FontWeight.bold : FontWeight.normal
                  ),
                ),
              ),
              if (isMapped || isNext)
                Positioned(
                   right: 0,
                   bottom: 0,
                   child: Row(
                     mainAxisSize: MainAxisSize.min,
                     children: [
                       _TinyButton(
                         icon: Icons.remove,
                         onTap: () => _updateVisualOverride(keyId, width: (width - 0.25).clamp(0.25, 10.0)),
                       ),
                       _TinyButton(
                         icon: Icons.add,
                         onTap: () => _updateVisualOverride(keyId, width: (width + 0.25).clamp(0.25, 10.0)),
                       ),
                     ],
                   ),
                ),
            ],
          ),
        ),
        if (width != 1.0)
          Text('${width}u', style: const TextStyle(fontSize: 8, color: Colors.grey)),
      ],
    );
  }

}

class _TinyButton extends StatelessWidget {
  const _TinyButton({required this.icon, required this.onTap});
  final IconData icon;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      child: Container(
        width: 12,
        height: 12,
        color: Colors.black12,
        child: Icon(icon, size: 8),
      ),
    );
  }
}

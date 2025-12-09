import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../ffi/bridge.dart';
import '../ffi/event_types.dart';
import '../services/facade/facade_state.dart';
import '../services/facade/keyrx_facade.dart';
import '../state/app_state.dart';

class MonitorPage extends StatefulWidget {
  const MonitorPage({super.key});

  @override
  State<MonitorPage> createState() => _MonitorPageState();
}

class _MonitorPageState extends State<MonitorPage> {
  final List<MonitorEvent> _events = [];
  final ScrollController _scrollController = ScrollController();
  bool _autoScroll = true;

  // Basic Virtual Key Map (fallback/local to avoid external dependency issues)
  final Map<int, String> _vkMap = {
    0x08: 'Backspace',
    0x09: 'Tab',
    0x0D: 'Enter',
    0x1B: 'Escape',
    0x20: 'Space',
    0x25: 'Left',
    0x26: 'Up',
    0x27: 'Right',
    0x28: 'Down',
    0x30: '0',
    0x31: '1',
    0x32: '2',
    0x33: '3',
    0x34: '4',
    0x35: '5',
    0x36: '6',
    0x37: '7',
    0x38: '8',
    0x39: '9',
    0x41: 'A',
    0x42: 'B',
    0x43: 'C',
    0x44: 'D',
    0x45: 'E',
    0x46: 'F',
    0x47: 'G',
    0x48: 'H',
    0x49: 'I',
    0x4A: 'J',
    0x4B: 'K',
    0x4C: 'L',
    0x4D: 'M',
    0x4E: 'N',
    0x4F: 'O',
    0x50: 'P',
    0x51: 'Q',
    0x52: 'R',
    0x53: 'S',
    0x54: 'T',
    0x55: 'U',
    0x56: 'V',
    0x57: 'W',
    0x58: 'X',
    0x59: 'Y',
    0x5A: 'Z',
    0xA0: 'LShift',
    0xA1: 'RShift',
    0xA2: 'LCtrl',
    0xA3: 'RCtrl',
    0xA4: 'LAlt',
    0xA5: 'RAlt',
  };

  @override
  void initState() {
    super.initState();
    // Defer FFI registration to after build to avoid blocking init
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _registerCallbacks();
    });
  }

  void _registerCallbacks() {
    final facade = context.read<KeyrxFacade>();
    final bridge = facade.services.bridge;

    // Check if callbacks are already registered to avoid duplicates
    if (!bridge.isEventCallbackRegistered(EventType.rawInput)) {
      bridge.registerEventCallback(EventType.rawInput, (payload) {
        if (!mounted) return;
        _handleEvent(EventType.rawInput, payload);
      });
    }

    if (!bridge.isEventCallbackRegistered(EventType.rawOutput)) {
      bridge.registerEventCallback(EventType.rawOutput, (payload) {
        if (!mounted) return;
        _handleEvent(EventType.rawOutput, payload);
      });
    }
  }

  void _handleEvent(EventType type, Uint8List payload) {
    try {
      final jsonStr = utf8.decode(payload);
      final data = json.decode(jsonStr);

      final event = MonitorEvent(
        type: type,
        timestamp: DateTime.now(),
        data: data,
        rawJson: jsonStr,
      );

      setState(() {
        _events.add(event);
        // Keep buffer size manageable
        if (_events.length > 1000) {
          _events.removeRange(0, 100);
        }
      });

      if (_autoScroll && _scrollController.hasClients) {
        // Scroll to bottom after frame
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (_scrollController.hasClients) {
            _scrollController.jumpTo(
              _scrollController.position.maxScrollExtent,
            );
          }
        });
      }
    } catch (e) {
      debugPrint('Error parsing monitor event: $e');
    }
  }

  Future<void> _startEngine() async {
    final appState = context.read<AppState>();
    final facade = context.read<KeyrxFacade>();

    String? scriptPath = appState.loadedScript;
    bool isDefaultScript = false;

    // If no script loaded, use/create a default monitor script
    if (scriptPath == null) {
      try {
        final profilesPath = facade.services.storagePathResolver
            .resolveProfilesPath();
        scriptPath = '$profilesPath${Platform.pathSeparator}monitor.rhai';
        isDefaultScript = true;

        // Ensure the script exists
        final saveResult = await facade.saveScript(
          scriptPath,
          '// Monitor Mode - Passthrough\n// This script allows raw input monitoring without remapping.\n',
        );

        if (saveResult.isErr) {
          throw Exception(
            saveResult.errOrNull?.userMessage ??
                'Failed to create default monitor script',
          );
        }
      } catch (e) {
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('Error preparing monitor: $e'),
              backgroundColor: Colors.red,
            ),
          );
        }
        return;
      }
    }

    final result = await facade.startEngine(scriptPath);
    result.when(
      ok: (_) {
        if (mounted && isDefaultScript) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(
              content: Text('Started in Monitor Mode (monitor.rhai)'),
              backgroundColor: Colors.blue,
              duration: Duration(seconds: 2),
            ),
          );
        }
      },
      err: (error) {
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('Failed to start engine: ${error.userMessage}'),
              backgroundColor: Colors.red,
            ),
          );
        }
      },
    );
  }

  Future<void> _stopEngine() async {
    final facade = context.read<KeyrxFacade>();
    await facade.stopEngine();
  }

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final facade = context.read<KeyrxFacade>();

    return StreamBuilder<FacadeState>(
      stream: facade.stateStream,
      initialData: facade.currentState,
      builder: (context, snapshot) {
        final facadeState = snapshot.data ?? facade.currentState;
        final isRunning = facadeState.engine == EngineStatus.running;

        return Scaffold(
          appBar: AppBar(
            title: Row(
              children: [
                const Icon(Icons.monitor_heart),
                const SizedBox(width: 8),
                const Text('Input Monitor'),
                const SizedBox(width: 16),
                _buildStatusBadge(isRunning),
              ],
            ),
            actions: [
              if (!isRunning)
                FilledButton.icon(
                  onPressed: _startEngine,
                  icon: const Icon(Icons.play_arrow),
                  label: const Text('Start Monitor'),
                  style: FilledButton.styleFrom(
                    backgroundColor: Colors.green,
                    foregroundColor: Colors.white,
                  ),
                )
              else
                FilledButton.icon(
                  onPressed: _stopEngine,
                  icon: const Icon(Icons.stop),
                  label: const Text('Stop Monitor'),
                  style: FilledButton.styleFrom(
                    backgroundColor: Colors.red,
                    foregroundColor: Colors.white,
                  ),
                ),
              const SizedBox(width: 8),
              IconButton(
                icon: Icon(
                  _autoScroll
                      ? Icons.vertical_align_bottom
                      : Icons.vertical_align_center,
                ),
                tooltip: _autoScroll
                    ? 'Disable Auto-scroll'
                    : 'Enable Auto-scroll',
                onPressed: () {
                  setState(() => _autoScroll = !_autoScroll);
                },
              ),
              IconButton(
                icon: const Icon(Icons.delete_sweep),
                tooltip: 'Clear History',
                onPressed: () {
                  setState(() => _events.clear());
                },
              ),
              const SizedBox(width: 8),
            ],
          ),
          body: _events.isEmpty
              ? _buildEmptyState(isRunning)
              : Column(
                  children: [
                    _buildHeader(),
                    Expanded(
                      child: ListView.builder(
                        controller: _scrollController,
                        padding: const EdgeInsets.only(bottom: 100),
                        itemCount: _events.length,
                        itemBuilder: (context, index) {
                          return _buildEventRow(_events[index]);
                        },
                      ),
                    ),
                  ],
                ),
        );
      },
    );
  }

  Widget _buildStatusBadge(bool isRunning) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: isRunning
            ? Colors.green.withValues(alpha: 0.2)
            : Colors.grey.withValues(alpha: 0.2),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: isRunning ? Colors.green : Colors.grey),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            Icons.fiber_manual_record,
            size: 10,
            color: isRunning ? Colors.green : Colors.grey,
          ),
          const SizedBox(width: 4),
          Text(
            isRunning ? 'RUNNING' : 'STOPPED',
            style: TextStyle(
              fontSize: 10,
              fontWeight: FontWeight.bold,
              color: isRunning ? Colors.green : Colors.grey,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildEmptyState(bool isRunning) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.monitor_heart_outlined, size: 64, color: Colors.grey[700]),
          const SizedBox(height: 16),
          Text(
            isRunning
                ? 'Monitoring Active\nWaiting for input...'
                : 'Monitor Stopped\nClick "Start Monitor" to begin',
            textAlign: TextAlign.center,
            style: TextStyle(fontSize: 16, color: Colors.grey[500]),
          ),
          if (isRunning) ...[
            const SizedBox(height: 24),
            const CircularProgressIndicator(),
          ],
        ],
      ),
    );
  }

  Widget _buildHeader() {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 16),
      color: Colors.black12,
      child: const Row(
        children: [
          Expanded(
            child: Text(
              'HARDWARE INPUT',
              style: TextStyle(fontWeight: FontWeight.bold, fontSize: 12),
            ),
          ),
          SizedBox(width: 16),
          Expanded(
            child: Text(
              'OS OUTPUT',
              style: TextStyle(fontWeight: FontWeight.bold, fontSize: 12),
            ),
          ),
          SizedBox(width: 80), // Time column
        ],
      ),
    );
  }

  Widget _buildEventRow(MonitorEvent event) {
    final isInput = event.type == EventType.rawInput;
    final timeStr =
        "${event.timestamp.hour.toString().padLeft(2, '0')}:${event.timestamp.minute.toString().padLeft(2, '0')}:${event.timestamp.second.toString().padLeft(2, '0')}.${event.timestamp.millisecond.toString().padLeft(3, '0')}";

    return Container(
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: Colors.grey[900]!)),
        color: isInput
            ? Colors.blue.withValues(alpha: 0.05)
            : Colors.orange.withValues(alpha: 0.05),
      ),
      padding: const EdgeInsets.symmetric(vertical: 4, horizontal: 16),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Input Column
          Expanded(
            child: isInput
                ? _buildEventContent(event, Colors.blue[300]!)
                : const SizedBox(),
          ),

          Container(width: 1, height: 24, color: Colors.white10),
          const SizedBox(width: 16),

          // Output Column
          Expanded(
            child: !isInput
                ? _buildEventContent(event, Colors.orange[300]!)
                : const SizedBox(),
          ),

          // Timestamp
          SizedBox(
            width: 80,
            child: Text(
              timeStr,
              style: TextStyle(
                fontSize: 10,
                color: Colors.grey[600],
                fontFamily: 'monospace',
              ),
              textAlign: TextAlign.right,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildEventContent(MonitorEvent event, Color color) {
    String summary = "Unknown";
    String details = "";

    // Parse common fields
    final data = event.data;
    if (data is Map) {
      if (data.containsKey('ScanCode')) {
        final sc = data['ScanCode'];
        final state = data['State'];
        summary = "ScanCode $sc";
        details = "$state";
      } else if (data.containsKey('VirtualKey')) {
        final vk = data['VirtualKey'];
        final state = data['State'];
        final name = _vkMap[vk] ?? "VK $vk";
        summary = name;
        details = "$state";
      } else if (data.containsKey('SendInput')) {
        final input = data['SendInput'];
        if (input is Map && input.containsKey('wScan')) {
          final scan = input['wScan'];
          final flags = input['dwFlags'];
          final isUp = (flags & 2) != 0; // KEYEVENTF_KEYUP = 0x0002
          summary = "Sent ScanCode $scan";
          details = isUp ? "Up" : "Down";
        }
      } else {
        // Fallback for generic map
        summary = "Event Data";
        details = event.rawJson;
      }
    } else {
      summary = event.rawJson;
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          summary,
          style: TextStyle(color: color, fontWeight: FontWeight.w500),
        ),
        if (details.isNotEmpty)
          Text(
            details,
            style: TextStyle(color: Colors.grey[500], fontSize: 10),
            overflow: TextOverflow.ellipsis,
            maxLines: 2,
          ),
      ],
    );
  }
}

class MonitorEvent {
  final EventType type;
  final DateTime timestamp;
  final dynamic data;
  final String rawJson;

  MonitorEvent({
    required this.type,
    required this.timestamp,
    required this.data,
    required this.rawJson,
  });
}


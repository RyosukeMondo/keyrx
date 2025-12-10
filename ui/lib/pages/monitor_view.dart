import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../ffi/bridge.dart';
import '../ffi/event_types.dart';
import '../services/facade/keyrx_facade.dart';

class MonitorView extends StatefulWidget {
  const MonitorView({super.key});

  @override
  State<MonitorView> createState() => _MonitorViewState();
}

class _MonitorViewState extends State<MonitorView>
    with AutomaticKeepAliveClientMixin {
  final List<MonitorEvent> _events = [];
  final ScrollController _scrollController = ScrollController();
  bool _autoScroll = true;

  // Basic map for visualization
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
  bool get wantKeepAlive => true;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _registerCallbacks();
    });
  }

  void _registerCallbacks() {
    final facade = context.read<KeyrxFacade>();
    final bridge = facade.services.bridge;

    bridge.registerEventCallback(EventType.rawInput, (payload) {
      if (!mounted) return;
      _handleEvent(EventType.rawInput, payload);
    });

    bridge.registerEventCallback(EventType.rawOutput, (payload) {
      if (!mounted) return;
      _handleEvent(EventType.rawOutput, payload);
    });
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
        if (_events.length > 500) {
          _events.removeRange(0, 50);
        }
      });

      if (_autoScroll && _scrollController.hasClients) {
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

  @override
  void dispose() {
    if (mounted) {
      final facade = context.read<KeyrxFacade>();
      final bridge = facade.services.bridge;
      bridge.unregisterEventCallback(EventType.rawInput);
      bridge.unregisterEventCallback(EventType.rawOutput);
    }
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    super.build(context);
    // Keep monitor view simple: just the table list
    return Column(
      children: [
        _buildToolbar(),
        _buildHeader(),
        Expanded(
          child: ListView.builder(
            controller: _scrollController,
            itemCount: _events.length,
            itemBuilder: (context, index) {
              return _buildEventRow(_events[index]);
            },
          ),
        ),
      ],
    );
  }

  Widget _buildToolbar() {
    return Padding(
      padding: const EdgeInsets.all(8.0),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.end,
        children: [
          TextButton.icon(
            icon: Icon(
              _autoScroll
                  ? Icons.vertical_align_bottom
                  : Icons.vertical_align_center,
            ),
            label: Text(_autoScroll ? 'Auto-scroll On' : 'Auto-scroll Off'),
            onPressed: () => setState(() => _autoScroll = !_autoScroll),
          ),
          const SizedBox(width: 8),
          TextButton.icon(
            icon: const Icon(Icons.delete_sweep),
            label: const Text('Clear'),
            onPressed: () => setState(() => _events.clear()),
          ),
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
              'INPUT',
              style: TextStyle(fontWeight: FontWeight.bold, fontSize: 11),
            ),
          ),
          SizedBox(width: 16),
          Expanded(
            child: Text(
              'OUTPUT',
              style: TextStyle(fontWeight: FontWeight.bold, fontSize: 11),
            ),
          ),
          SizedBox(
            width: 60,
            child: Text(
              'TIME',
              style: TextStyle(fontWeight: FontWeight.bold, fontSize: 11),
              textAlign: TextAlign.right,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildEventRow(MonitorEvent event) {
    final isInput = event.type == EventType.rawInput;
    final timeStr =
        "${event.timestamp.hour.toString().padLeft(2, '0')}:${event.timestamp.minute.toString().padLeft(2, '0')}:${event.timestamp.second.toString().padLeft(2, '0')}";

    return Container(
      decoration: BoxDecoration(
        border: Border(
          bottom: BorderSide(color: Colors.grey.withValues(alpha: 0.1)),
        ),
        color: isInput
            ? Colors.blue.withValues(alpha: 0.05)
            : Colors.orange.withValues(alpha: 0.05),
      ),
      padding: const EdgeInsets.symmetric(vertical: 2, horizontal: 16),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            child: isInput
                ? _buildEventContent(event, Colors.blue[300]!)
                : const SizedBox(),
          ),
          Container(
            width: 1,
            height: 20,
            color: Colors.grey.withValues(alpha: 0.2),
          ),
          const SizedBox(width: 16),
          Expanded(
            child: !isInput
                ? _buildEventContent(event, Colors.orange[300]!)
                : const SizedBox(),
          ),
          SizedBox(
            width: 60,
            child: Text(
              timeStr,
              style: const TextStyle(
                fontSize: 10,
                fontFamily: 'monospace',
                color: Colors.grey,
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

    final data = event.data;
    if (data is Map) {
      if (data.containsKey('ScanCode')) {
        final sc = data['ScanCode'];
        final state = data['State'];
        summary = "SC $sc ($state)";
      } else if (data.containsKey('VirtualKey')) {
        final vk = data['VirtualKey'];
        final state = data['State'];
        final name = _vkMap[vk] ?? "VK $vk";
        summary = "$name ($state)";
      } else if (data.containsKey('SendInput')) {
        summary = "Sent Input";
      }
    } else {
      summary = "Raw Data";
    }

    return Text(
      summary,
      style: TextStyle(color: color, fontSize: 11, fontFamily: 'monospace'),
      maxLines: 1,
      overflow: TextOverflow.ellipsis,
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

/// Rhai REPL console page.
///
/// Provides an interactive console for typing Rhai commands
/// directly into the running engine.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';

import '../services/engine_service.dart';
import '../services/service_registry.dart';

/// Interactive Rhai REPL console.
class ConsolePage extends StatefulWidget {
  const ConsolePage({super.key});

  @override
  State<ConsolePage> createState() => _ConsolePageState();
}

class _ConsolePageState extends State<ConsolePage> {
  final TextEditingController _inputController = TextEditingController();
  final ScrollController _scrollController = ScrollController();
  final List<ConsoleEntry> _history = [];
  final List<String> _commandHistory = [];
  int _historyIndex = -1;
  EngineService? _engine;
  bool _isBusy = false;

  @override
  void initState() {
    super.initState();
    final registry = Provider.of<ServiceRegistry>(context, listen: false);
    _engine = registry.engineService;
  }

  @override
  void dispose() {
    _inputController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Rhai Console'),
        actions: [
          IconButton(
            icon: const Icon(Icons.clear_all),
            onPressed: _clearHistory,
            tooltip: 'Clear Console',
          ),
        ],
      ),
      body: Column(
        children: [
          // Output area
          Expanded(
            child: Container(
              color: Colors.black87,
              child: ListView.builder(
                controller: _scrollController,
                padding: const EdgeInsets.all(8),
                itemCount: _history.length,
                itemBuilder: (context, index) {
                  final entry = _history[index];
                  return _buildEntry(entry);
                },
              ),
            ),
          ),
          // Input area
          Container(
            color: Colors.black,
            padding: const EdgeInsets.all(8),
            child: Row(
              children: [
                const Text(
                  '> ',
                  style: TextStyle(
                    color: Colors.green,
                    fontFamily: 'monospace',
                    fontSize: 16,
                  ),
                ),
                Expanded(
                  child: KeyboardListener(
                    focusNode: FocusNode(),
                    onKeyEvent: _handleKeyEvent,
                    child: TextField(
                      controller: _inputController,
                      style: const TextStyle(
                        color: Colors.white,
                        fontFamily: 'monospace',
                        fontSize: 16,
                      ),
                      decoration: const InputDecoration(
                        border: InputBorder.none,
                        hintText: 'Enter Rhai command...',
                        hintStyle: TextStyle(color: Colors.grey),
                      ),
                      onSubmitted: _executeCommand,
                      enabled: !_isBusy,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildEntry(ConsoleEntry entry) {
    final lower = entry.text.toLowerCase();
    final isError = entry.isError || lower.startsWith('error:');
    final isOk = lower.startsWith('ok:');
    final badgeColor = entry.isInput
        ? Colors.blueAccent
        : isError
        ? Colors.redAccent
        : Colors.green;
    final label = entry.isInput
        ? 'CMD'
        : isError
        ? 'ERROR'
        : isOk
        ? 'OK'
        : 'OUT';
    final displayText = entry.isInput ? entry.text : _stripPrefix(entry.text);

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Container(
        decoration: BoxDecoration(
          color: Colors.white.withOpacity(0.04),
          borderRadius: BorderRadius.circular(6),
          border: Border.all(color: badgeColor.withOpacity(0.35)),
        ),
        padding: const EdgeInsets.symmetric(vertical: 6, horizontal: 8),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _buildBadge(label, badgeColor),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                displayText,
                style: const TextStyle(
                  color: Colors.white,
                  fontFamily: 'monospace',
                  fontSize: 14,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  void _handleKeyEvent(KeyEvent event) {
    if (event is KeyDownEvent) {
      if (event.logicalKey == LogicalKeyboardKey.arrowUp) {
        _navigateHistory(-1);
      } else if (event.logicalKey == LogicalKeyboardKey.arrowDown) {
        _navigateHistory(1);
      }
    }
  }

  void _navigateHistory(int direction) {
    if (_commandHistory.isEmpty) return;

    setState(() {
      _historyIndex += direction;
      _historyIndex = _historyIndex.clamp(-1, _commandHistory.length - 1);

      if (_historyIndex >= 0) {
        _inputController.text = _commandHistory[_historyIndex];
        _inputController.selection = TextSelection.fromPosition(
          TextPosition(offset: _inputController.text.length),
        );
      } else {
        _inputController.clear();
      }
    });
  }

  Future<void> _executeCommand(String command) async {
    if (command.trim().isEmpty || _isBusy) return;

    setState(() {
      _history.add(ConsoleEntry(command, isInput: true));
      _commandHistory.insert(0, command);
      _historyIndex = -1;
      _isBusy = true;
    });

    final engine = _engine;
    ConsoleEvalResult result;

    if (engine == null) {
      result = const ConsoleEvalResult(
        success: false,
        output: 'Engine unavailable.',
      );
    } else {
      result = await engine.eval(command);
    }

    setState(() {
      _history.add(ConsoleEntry(result.output, isError: result.isError));
      _isBusy = false;
    });

    _inputController.clear();
    _scrollToBottom();
  }

  void _scrollToBottom() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _scrollController.animateTo(
        _scrollController.position.maxScrollExtent,
        duration: const Duration(milliseconds: 100),
        curve: Curves.easeOut,
      );
    });
  }

  void _clearHistory() {
    setState(() {
      _history.clear();
    });
  }

  Widget _buildBadge(String text, Color color) {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 2, horizontal: 6),
      decoration: BoxDecoration(
        color: color.withOpacity(0.15),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: color.withOpacity(0.5)),
      ),
      child: Text(
        text,
        style: TextStyle(
          color: color,
          fontWeight: FontWeight.bold,
          fontSize: 11,
          letterSpacing: 0.5,
        ),
      ),
    );
  }

  String _stripPrefix(String text) {
    final lower = text.toLowerCase();
    if (lower.startsWith('ok:') || lower.startsWith('error:')) {
      final idx = text.indexOf(':');
      return idx > -1 ? text.substring(idx + 1).trimLeft() : text;
    }
    return text;
  }
}

/// A single console history entry.
class ConsoleEntry {
  final String text;
  final bool isInput;
  final bool isError;

  ConsoleEntry(this.text, {this.isInput = false, this.isError = false});
}

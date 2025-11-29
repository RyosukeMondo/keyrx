/// Rhai REPL console page.
///
/// Provides an interactive console for typing Rhai commands
/// directly into the running engine.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

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
    final color = entry.isError ? Colors.red : Colors.white;
    final prefix = entry.isInput ? '> ' : '  ';

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Text(
        '$prefix${entry.text}',
        style: TextStyle(
          color: entry.isInput ? Colors.green : color,
          fontFamily: 'monospace',
          fontSize: 14,
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

  void _executeCommand(String command) {
    if (command.trim().isEmpty) return;

    setState(() {
      _history.add(ConsoleEntry(command, isInput: true));
      _commandHistory.insert(0, command);
      _historyIndex = -1;
    });

    // TODO: Execute command via FFI bridge
    final result = _mockExecute(command);

    setState(() {
      _history.add(ConsoleEntry(result.text, isError: result.isError));
    });

    _inputController.clear();
    _scrollToBottom();
  }

  ConsoleEntry _mockExecute(String command) {
    // Mock execution for now
    if (command.startsWith('print')) {
      return ConsoleEntry('()', isError: false);
    }
    return ConsoleEntry('Command executed', isError: false);
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
}

/// A single console history entry.
class ConsoleEntry {
  final String text;
  final bool isInput;
  final bool isError;

  ConsoleEntry(this.text, {this.isInput = false, this.isError = false});
}

/// Rhai REPL console page.
///
/// Provides an interactive console for typing Rhai commands
/// directly into the running engine.
library;

import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';

import '../services/console_parser.dart';
import '../services/facade/keyrx_facade.dart';
import '../models/log_entry.dart';

/// Interactive Rhai REPL console.
class ConsolePage extends StatefulWidget {
  const ConsolePage({super.key, this.parser = const ConsoleParser()});

  /// The parser for classifying console output.
  final ConsoleParser parser;

  @override
  State<ConsolePage> createState() => _ConsolePageState();
}

class _ConsolePageState extends State<ConsolePage> {
  final TextEditingController _inputController = TextEditingController();
  final ScrollController _scrollController = ScrollController();
  final List<ConsoleEntry> _history = [];
  final List<String> _commandHistory = [];
  int _historyIndex = -1;
  bool _isBusy = false;

  StreamSubscription? _logSubscription;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (_logSubscription == null) {
      final facade = Provider.of<KeyrxFacade>(context, listen: false);
      _logSubscription = facade.logStream.listen((data) {
        if (data is LogEntry) {
          if (!mounted) return;
          setState(() {
            _history.add(ConsoleEntry(data.message, logData: data));
          });
          _scrollToBottom();
        }
      });
    }
  }

  @override
  void dispose() {
    _logSubscription?.cancel();
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
    final parser = widget.parser;
    final entryType = parser.classify(entry.text, isInput: entry.isInput);
    final isError = entry.isError || entryType == ConsoleEntryType.error;
    final style = _getEntryStyle(entryType, isError, entry);
    final showInitButton = parser.needsInitButton(entry.text, isError: isError);
    final displayText = entry.isInput
        ? entry.text
        : parser.stripPrefix(entry.text);

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Container(
        decoration: BoxDecoration(
          color: Colors.black.withAlpha(128),
          borderRadius: BorderRadius.circular(6),
          border: Border.all(color: style.color.withAlpha(89)),
        ),
        padding: const EdgeInsets.symmetric(vertical: 6, horizontal: 8),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _buildEntryRow(displayText, style),
            if (showInitButton) _buildInitButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildEntryRow(String text, _EntryStyle style) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildBadge(style.label, style.color),
        const SizedBox(width: 6),
        Icon(style.icon, color: style.color, size: 16),
        const SizedBox(width: 6),
        Expanded(
          child: SelectableText(
            text,
            style: const TextStyle(
              color: Colors.white,
              fontFamily: 'monospace',
              fontSize: 14,
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildInitButton() {
    return Padding(
      padding: const EdgeInsets.only(top: 8),
      child: ElevatedButton.icon(
        onPressed: _initializeEngine,
        icon: const Icon(Icons.power_settings_new, size: 16),
        label: const Text('Initialize Engine'),
        style: ElevatedButton.styleFrom(
          backgroundColor: Colors.blueAccent,
          foregroundColor: Colors.white,
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          textStyle: const TextStyle(fontSize: 12),
        ),
      ),
    );
  }

  _EntryStyle _getEntryStyle(
    ConsoleEntryType entryType,
    bool isError,
    ConsoleEntry entry,
  ) {
    final color = switch (entryType) {
      ConsoleEntryType.command => Colors.blueAccent,
      ConsoleEntryType.error => Colors.redAccent,
      _ when isError => Colors.redAccent,
      _
          when ((entryType == ConsoleEntryType.output) ||
              entry.logData != null) =>
        Colors.grey,
      _ => Colors.green,
    };
    final label = switch (entryType) {
      ConsoleEntryType.command => 'CMD',
      ConsoleEntryType.error => 'ERROR',
      _ when isError => 'ERROR',
      ConsoleEntryType.ok => 'OK',
      ConsoleEntryType.output => 'OUT',
    };
    final icon = switch (entryType) {
      ConsoleEntryType.command => Icons.chevron_right,
      ConsoleEntryType.error => Icons.warning_amber_rounded,
      _ when isError => Icons.warning_amber_rounded,
      _ => Icons.check_circle_outline,
    };
    return _EntryStyle(color: color, label: label, icon: icon);
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

    final facade = Provider.of<KeyrxFacade>(context, listen: false);
    final engineService = facade.services.engineService;
    final result = await engineService.eval(command);

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

  Future<void> _initializeEngine() async {
    setState(() => _isBusy = true);
    final facade = Provider.of<KeyrxFacade>(context, listen: false);
    final engineService = facade.services.engineService;
    final success = await engineService.initialize();
    setState(() {
      _history.add(
        ConsoleEntry(
          success ? 'ok: Engine initialized.' : 'error: Initialization failed.',
          isError: !success,
        ),
      );
      _isBusy = false;
    });
    _scrollToBottom();
  }

  Widget _buildBadge(String text, Color color) {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 2, horizontal: 6),
      decoration: BoxDecoration(
        color: color.withAlpha(38),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: color.withAlpha(128)),
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
}

/// A single console history entry.
class ConsoleEntry {
  final String text;
  final bool isInput;
  final bool isError;

  ConsoleEntry(
    this.text, {
    this.isInput = false,
    this.isError = false,
    this.logData,
  });

  final LogEntry? logData;
}

/// Style configuration for a console entry.
class _EntryStyle {
  const _EntryStyle({
    required this.color,
    required this.label,
    required this.icon,
  });

  final Color color;
  final String label;
  final IconData icon;
}

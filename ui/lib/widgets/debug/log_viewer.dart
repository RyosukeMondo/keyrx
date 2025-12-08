/// LogViewer widget for displaying and filtering logs.
///
/// Provides a comprehensive log viewing interface with:
/// - Log level filtering
/// - Target/module filtering
/// - Search/text filtering
/// - Auto-scroll capability
/// - Color-coded log levels
library;

import 'dart:async';

import 'package:flutter/material.dart';

import '../../services/observability_service.dart';

/// Widget for viewing and filtering log entries.
class LogViewer extends StatefulWidget {
  const LogViewer({required this.observabilityService, super.key});

  final ObservabilityService observabilityService;

  @override
  State<LogViewer> createState() => _LogViewerState();
}

class _LogViewerState extends State<LogViewer> {
  final List<LogEntry> _allLogs = [];
  List<LogEntry> _filteredLogs = [];
  StreamSubscription<List<LogEntry>>? _logSubscription;

  // Filter state
  LogLevel _minLevel = LogLevel.trace;
  String _searchText = '';
  String? _targetFilter;
  bool _autoScroll = true;

  final ScrollController _scrollController = ScrollController();
  final TextEditingController _searchController = TextEditingController();

  @override
  void initState() {
    super.initState();
    _initializeLogging();
  }

  Future<void> _initializeLogging() async {
    await widget.observabilityService.initialize();
    await widget.observabilityService.setLogEnabled(true);

    // Subscribe to log stream
    _logSubscription = widget.observabilityService.logStream.listen((logs) {
      setState(() {
        _allLogs.addAll(logs);
        _applyFilters();
      });

      // Auto-scroll to bottom if enabled
      if (_autoScroll && _scrollController.hasClients) {
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (_scrollController.hasClients) {
            _scrollController.animateTo(
              _scrollController.position.maxScrollExtent,
              duration: const Duration(milliseconds: 200),
              curve: Curves.easeOut,
            );
          }
        });
      }
    });
  }

  void _applyFilters() {
    _filteredLogs = _allLogs.where((log) {
      // Level filter
      if (log.level.index < _minLevel.index) {
        return false;
      }

      // Target filter
      if (_targetFilter != null && log.target != _targetFilter) {
        return false;
      }

      // Search filter
      if (_searchText.isNotEmpty) {
        final searchLower = _searchText.toLowerCase();
        final matchesMessage = log.message.toLowerCase().contains(searchLower);
        final matchesTarget = log.target.toLowerCase().contains(searchLower);
        final matchesFields = log.fields.entries.any(
          (e) =>
              e.key.toLowerCase().contains(searchLower) ||
              e.value.toLowerCase().contains(searchLower),
        );

        if (!matchesMessage && !matchesTarget && !matchesFields) {
          return false;
        }
      }

      return true;
    }).toList();
  }

  Set<String> _getAvailableTargets() {
    return _allLogs.map((log) => log.target).toSet();
  }

  Color _getLevelColor(LogLevel level) {
    return switch (level) {
      LogLevel.trace => Colors.grey,
      LogLevel.debug => Colors.blue,
      LogLevel.info => Colors.green,
      LogLevel.warn => Colors.orange,
      LogLevel.error => Colors.red,
    };
  }

  IconData _getLevelIcon(LogLevel level) {
    return switch (level) {
      LogLevel.trace => Icons.bug_report,
      LogLevel.debug => Icons.code,
      LogLevel.info => Icons.info_outline,
      LogLevel.warn => Icons.warning_amber,
      LogLevel.error => Icons.error_outline,
    };
  }

  String _formatTimestamp(DateTime dt) {
    return '${dt.hour.toString().padLeft(2, '0')}:'
        '${dt.minute.toString().padLeft(2, '0')}:'
        '${dt.second.toString().padLeft(2, '0')}.'
        '${(dt.millisecond ~/ 100).toString()}';
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final availableTargets = _getAvailableTargets();

    return Column(
      children: [
        // Toolbar
        Card(
          margin: const EdgeInsets.all(8),
          child: Padding(
            padding: const EdgeInsets.all(8),
            child: Column(
              children: [
                Row(
                  children: [
                    // Level filter
                    Expanded(
                      child: DropdownButtonFormField<LogLevel>(
                        initialValue: _minLevel,
                        decoration: const InputDecoration(
                          labelText: 'Min Level',
                          isDense: true,
                          border: OutlineInputBorder(),
                        ),
                        items: LogLevel.values.map((level) {
                          return DropdownMenuItem(
                            value: level,
                            child: Row(
                              children: [
                                Icon(
                                  _getLevelIcon(level),
                                  size: 16,
                                  color: _getLevelColor(level),
                                ),
                                const SizedBox(width: 8),
                                Text(level.name.toUpperCase()),
                              ],
                            ),
                          );
                        }).toList(),
                        onChanged: (level) {
                          if (level != null) {
                            setState(() {
                              _minLevel = level;
                              _applyFilters();
                            });
                          }
                        },
                      ),
                    ),
                    const SizedBox(width: 8),
                    // Target filter
                    Expanded(
                      child: DropdownButtonFormField<String?>(
                        initialValue: _targetFilter,
                        decoration: const InputDecoration(
                          labelText: 'Target',
                          isDense: true,
                          border: OutlineInputBorder(),
                        ),
                        items: [
                          const DropdownMenuItem(
                            value: null,
                            child: Text('All'),
                          ),
                          ...availableTargets.map((target) {
                            return DropdownMenuItem(
                              value: target,
                              child: Text(
                                target,
                                overflow: TextOverflow.ellipsis,
                              ),
                            );
                          }),
                        ],
                        onChanged: (target) {
                          setState(() {
                            _targetFilter = target;
                            _applyFilters();
                          });
                        },
                      ),
                    ),
                    const SizedBox(width: 8),
                    // Auto-scroll toggle
                    Tooltip(
                      message: 'Auto-scroll',
                      child: IconButton(
                        icon: Icon(
                          _autoScroll
                              ? Icons.vertical_align_bottom
                              : Icons.vertical_align_center,
                        ),
                        onPressed: () {
                          setState(() {
                            _autoScroll = !_autoScroll;
                          });
                        },
                        color: _autoScroll
                            ? theme.colorScheme.primary
                            : theme.iconTheme.color,
                      ),
                    ),
                    // Clear logs button
                    Tooltip(
                      message: 'Clear logs',
                      child: IconButton(
                        icon: const Icon(Icons.delete_outline),
                        onPressed: () {
                          setState(() {
                            _allLogs.clear();
                            _applyFilters();
                          });
                          widget.observabilityService.clearLogs();
                        },
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 8),
                // Search field
                TextField(
                  controller: _searchController,
                  decoration: InputDecoration(
                    labelText: 'Search',
                    isDense: true,
                    border: const OutlineInputBorder(),
                    prefixIcon: const Icon(Icons.search),
                    suffixIcon: _searchText.isNotEmpty
                        ? IconButton(
                            icon: const Icon(Icons.clear),
                            onPressed: () {
                              _searchController.clear();
                              setState(() {
                                _searchText = '';
                                _applyFilters();
                              });
                            },
                          )
                        : null,
                  ),
                  onChanged: (value) {
                    setState(() {
                      _searchText = value;
                      _applyFilters();
                    });
                  },
                ),
              ],
            ),
          ),
        ),
        // Stats bar
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
          color: theme.colorScheme.surfaceContainerHighest,
          child: Row(
            children: [
              Text(
                'Showing ${_filteredLogs.length} of ${_allLogs.length} logs',
                style: theme.textTheme.bodySmall,
              ),
            ],
          ),
        ),
        // Log list
        Expanded(
          child: _filteredLogs.isEmpty
              ? Center(
                  child: Text(
                    _allLogs.isEmpty ? 'No logs yet' : 'No logs match filters',
                    style: theme.textTheme.bodyLarge?.copyWith(
                      color: theme.colorScheme.onSurface.withOpacity(0.5),
                    ),
                  ),
                )
              : ListView.builder(
                  controller: _scrollController,
                  itemCount: _filteredLogs.length,
                  itemBuilder: (context, index) {
                    final log = _filteredLogs[index];
                    return _LogEntryTile(
                      log: log,
                      levelColor: _getLevelColor(log.level),
                      levelIcon: _getLevelIcon(log.level),
                      formatTimestamp: _formatTimestamp,
                    );
                  },
                ),
        ),
      ],
    );
  }

  @override
  void dispose() {
    _logSubscription?.cancel();
    _scrollController.dispose();
    _searchController.dispose();
    super.dispose();
  }
}

/// Individual log entry tile.
class _LogEntryTile extends StatefulWidget {
  const _LogEntryTile({
    required this.log,
    required this.levelColor,
    required this.levelIcon,
    required this.formatTimestamp,
  });

  final LogEntry log;
  final Color levelColor;
  final IconData levelIcon;
  final String Function(DateTime) formatTimestamp;

  @override
  State<_LogEntryTile> createState() => _LogEntryTileState();
}

class _LogEntryTileState extends State<_LogEntryTile> {
  bool _expanded = false;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final hasFields = widget.log.fields.isNotEmpty;
    final hasSpan = widget.log.span != null;
    final canExpand = hasFields || hasSpan;

    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
      child: InkWell(
        onTap: canExpand
            ? () {
                setState(() {
                  _expanded = !_expanded;
                });
              }
            : null,
        child: Padding(
          padding: const EdgeInsets.all(8),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // Level indicator
                  Icon(widget.levelIcon, size: 16, color: widget.levelColor),
                  const SizedBox(width: 8),
                  // Timestamp
                  Text(
                    widget.formatTimestamp(widget.log.dateTime),
                    style: theme.textTheme.bodySmall?.copyWith(
                      fontFamily: 'monospace',
                      color: theme.colorScheme.onSurface.withOpacity(0.6),
                    ),
                  ),
                  const SizedBox(width: 8),
                  // Target
                  Container(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 6,
                      vertical: 2,
                    ),
                    decoration: BoxDecoration(
                      color: theme.colorScheme.secondaryContainer,
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(
                      widget.log.target,
                      style: theme.textTheme.bodySmall?.copyWith(
                        fontFamily: 'monospace',
                        color: theme.colorScheme.onSecondaryContainer,
                      ),
                    ),
                  ),
                  const SizedBox(width: 8),
                  // Message
                  Expanded(
                    child: Text(
                      widget.log.message,
                      style: theme.textTheme.bodyMedium?.copyWith(
                        fontFamily: 'monospace',
                      ),
                    ),
                  ),
                  // Expand indicator
                  if (canExpand)
                    Icon(
                      _expanded ? Icons.expand_less : Icons.expand_more,
                      size: 16,
                      color: theme.colorScheme.onSurface.withOpacity(0.6),
                    ),
                ],
              ),
              // Expanded details
              if (_expanded) ...[
                const SizedBox(height: 8),
                const Divider(height: 1),
                const SizedBox(height: 8),
                if (hasSpan)
                  Padding(
                    padding: const EdgeInsets.only(left: 24, bottom: 4),
                    child: Row(
                      children: [
                        Text(
                          'Span: ',
                          style: theme.textTheme.bodySmall?.copyWith(
                            fontWeight: FontWeight.bold,
                            color: theme.colorScheme.onSurface.withOpacity(0.6),
                          ),
                        ),
                        Text(
                          widget.log.span!,
                          style: theme.textTheme.bodySmall?.copyWith(
                            fontFamily: 'monospace',
                          ),
                        ),
                      ],
                    ),
                  ),
                if (hasFields)
                  Padding(
                    padding: const EdgeInsets.only(left: 24),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'Fields:',
                          style: theme.textTheme.bodySmall?.copyWith(
                            fontWeight: FontWeight.bold,
                            color: theme.colorScheme.onSurface.withOpacity(0.6),
                          ),
                        ),
                        const SizedBox(height: 4),
                        ...widget.log.fields.entries.map((entry) {
                          return Padding(
                            padding: const EdgeInsets.only(left: 8, bottom: 2),
                            child: Row(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                SizedBox(
                                  width: 120,
                                  child: Text(
                                    '${entry.key}:',
                                    style: theme.textTheme.bodySmall?.copyWith(
                                      fontFamily: 'monospace',
                                      color: theme.colorScheme.primary,
                                    ),
                                  ),
                                ),
                                Expanded(
                                  child: Text(
                                    entry.value,
                                    style: theme.textTheme.bodySmall?.copyWith(
                                      fontFamily: 'monospace',
                                    ),
                                  ),
                                ),
                              ],
                            ),
                          );
                        }),
                      ],
                    ),
                  ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}

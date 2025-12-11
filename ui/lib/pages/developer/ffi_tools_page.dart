/// FFI Developer Tools page for browsing and testing FFI functions.
///
/// Provides:
/// - Function browser with live search
/// - Interactive parameter editor with validation
/// - Call history and results log
/// - Real-time event monitoring
library;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'dart:convert';

import '../../services/ffi_introspection_service.dart';
import '../../ffi/introspection_models.dart';
import '../../ffi/bindings.dart';

/// FFI Tools page for developer introspection
class FfiToolsPage extends StatefulWidget {
  const FfiToolsPage({super.key});

  @override
  State<FfiToolsPage> createState() => _FfiToolsPageState();
}

class _FfiToolsPageState extends State<FfiToolsPage>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  final _searchController = TextEditingController();

  IntrospectionData? _metadata;
  bool _isLoading = false;
  String? _error;

  // Function browser state
  String? _selectedDomain;
  FunctionMetadata? _selectedFunction;
  final Map<String, TextEditingController> _paramControllers = {};

  // Call history
  final List<FfiCallRecord> _callHistory = [];

  // Event monitor
  final List<FfiEventRecord> _eventLog = [];

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 3, vsync: this);
    _loadMetadata();
  }

  @override
  void dispose() {
    _tabController.dispose();
    _searchController.dispose();
    for (final controller in _paramControllers.values) {
      controller.dispose();
    }
    super.dispose();
  }

  Future<void> _loadMetadata() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    try {
      final bindings = context.read<KeyrxBindings>();
      final service = FfiIntrospectionService(bindings);
      final data = await service.getMetadata();

      if (!mounted) return;

      setState(() {
        _metadata = data;
        _isLoading = false;
      });
    } catch (e) {
      if (!mounted) return;

      setState(() {
        _error = 'Failed to load FFI metadata: $e';
        _isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('FFI Developer Tools'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _loadMetadata,
            tooltip: 'Refresh metadata',
          ),
        ],
        bottom: TabBar(
          controller: _tabController,
          tabs: const [
            Tab(icon: Icon(Icons.functions), text: 'Functions'),
            Tab(icon: Icon(Icons.history), text: 'Call History'),
            Tab(icon: Icon(Icons.monitor), text: 'Events'),
          ],
        ),
      ),
      body: _isLoading
          ? const Center(child: CircularProgressIndicator())
          : _error != null
              ? Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.error_outline,
                          size: 64, color: Colors.red[300]),
                      const SizedBox(height: 16),
                      Text(_error!, style: const TextStyle(color: Colors.red)),
                      const SizedBox(height: 16),
                      ElevatedButton(
                        onPressed: _loadMetadata,
                        child: const Text('Retry'),
                      ),
                    ],
                  ),
                )
              : TabBarView(
                  controller: _tabController,
                  children: [
                    _buildFunctionBrowser(),
                    _buildCallHistory(),
                    _buildEventMonitor(),
                  ],
                ),
    );
  }

  Widget _buildFunctionBrowser() {
    if (_metadata == null) {
      return const Center(child: Text('No metadata available'));
    }

    return Row(
      children: [
        // Left sidebar: Domain & function list
        SizedBox(
          width: 300,
          child: Card(
            margin: const EdgeInsets.all(8),
            child: Column(
              children: [
                Padding(
                  padding: const EdgeInsets.all(8),
                  child: TextField(
                    controller: _searchController,
                    decoration: const InputDecoration(
                      hintText: 'Search functions...',
                      prefixIcon: Icon(Icons.search),
                      border: OutlineInputBorder(),
                    ),
                    onChanged: (_) => setState(() {}),
                  ),
                ),
                Expanded(
                  child: ListView(
                    children: _buildFunctionList(),
                  ),
                ),
              ],
            ),
          ),
        ),
        // Right panel: Function details and tester
        Expanded(
          child: _selectedFunction == null
              ? const Center(
                  child: Text('Select a function to test'),
                )
              : _buildFunctionTester(),
        ),
      ],
    );
  }

  List<Widget> _buildFunctionList() {
    final query = _searchController.text.toLowerCase();
    final widgets = <Widget>[];

    for (final domain in _metadata!.domains) {
      final matchingFunctions = domain.functions.where((f) =>
          query.isEmpty ||
          f.name.toLowerCase().contains(query) ||
          f.description.toLowerCase().contains(query));

      if (matchingFunctions.isEmpty) continue;

      widgets.add(
        ExpansionTile(
          title: Text(
            domain.name,
            style: const TextStyle(fontWeight: FontWeight.bold),
          ),
          subtitle: Text('${matchingFunctions.length} functions'),
          initiallyExpanded: _selectedDomain == domain.name,
          children: matchingFunctions.map((func) {
            final isSelected = _selectedFunction?.name == func.name &&
                _selectedDomain == domain.name;

            return ListTile(
              title: Text(func.name),
              subtitle: Text(
                func.description,
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
              ),
              selected: isSelected,
              onTap: () {
                setState(() {
                  _selectedDomain = domain.name;
                  _selectedFunction = func;
                  _initParamControllers(func);
                });
              },
              trailing: func.deprecated
                  ? const Chip(
                      label: Text('Deprecated',
                          style: TextStyle(fontSize: 10)),
                      backgroundColor: Colors.orange,
                    )
                  : null,
            );
          }).toList(),
        ),
      );
    }

    return widgets;
  }

  void _initParamControllers(FunctionMetadata func) {
    _paramControllers.clear();
    for (final param in func.parameters) {
      _paramControllers[param.name] = TextEditingController();
    }
  }

  Widget _buildFunctionTester() {
    final func = _selectedFunction!;

    return Card(
      margin: const EdgeInsets.all(8),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Function header
            Row(
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        func.name,
                        style: const TextStyle(
                          fontSize: 20,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        func.description,
                        style: TextStyle(
                          color: Colors.grey[600],
                          fontSize: 14,
                        ),
                      ),
                      const SizedBox(height: 8),
                      Text(
                        'Rust: ${func.rustName}',
                        style: const TextStyle(
                          fontFamily: 'monospace',
                          fontSize: 12,
                          color: Colors.blue,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
            const Divider(height: 32),

            // Parameters section
            if (func.parameters.isNotEmpty) ...[
              const Text(
                'Parameters',
                style: TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.bold,
                ),
              ),
              const SizedBox(height: 12),
              Expanded(
                child: ListView.builder(
                  itemCount: func.parameters.length,
                  itemBuilder: (context, index) {
                    final param = func.parameters[index];
                    return _buildParameterInput(param);
                  },
                ),
              ),
            ] else
              const Expanded(
                child: Center(
                  child: Text('No parameters required'),
                ),
              ),

            const Divider(height: 32),

            // Example section
            if (func.example != null) ...[
              const Text(
                'Example',
                style: TextStyle(
                  fontSize: 14,
                  fontWeight: FontWeight.bold,
                ),
              ),
              const SizedBox(height: 8),
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Colors.grey[100],
                  borderRadius: BorderRadius.circular(4),
                  border: Border.all(color: Colors.grey[300]!),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text('Input:',
                        style: TextStyle(fontWeight: FontWeight.bold)),
                    Text(
                      jsonEncode(func.example!.input),
                      style: const TextStyle(fontFamily: 'monospace'),
                    ),
                    const SizedBox(height: 8),
                    const Text('Output:',
                        style: TextStyle(fontWeight: FontWeight.bold)),
                    Text(
                      jsonEncode(func.example!.output),
                      style: const TextStyle(fontFamily: 'monospace'),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 16),
            ],

            // Invoke button
            SizedBox(
              width: double.infinity,
              child: ElevatedButton.icon(
                onPressed: () => _invokeFunction(func),
                icon: const Icon(Icons.play_arrow),
                label: const Text('Invoke Function'),
                style: ElevatedButton.styleFrom(
                  padding: const EdgeInsets.all(16),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildParameterInput(ParameterMetadata param) {
    final controller = _paramControllers[param.name]!;

    return Padding(
      padding: const EdgeInsets.only(bottom: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Text(
                param.name,
                style: const TextStyle(
                  fontWeight: FontWeight.bold,
                  fontSize: 14,
                ),
              ),
              const SizedBox(width: 8),
              Chip(
                label: Text(
                  param.typeName,
                  style: const TextStyle(fontSize: 11),
                ),
                backgroundColor: Colors.blue[100],
                padding: EdgeInsets.zero,
                visualDensity: VisualDensity.compact,
              ),
              if (param.required) ...[
                const SizedBox(width: 8),
                const Chip(
                  label: Text('Required',
                      style: TextStyle(fontSize: 11)),
                  backgroundColor: Colors.orange,
                  padding: EdgeInsets.zero,
                  visualDensity: VisualDensity.compact,
                ),
              ],
            ],
          ),
          const SizedBox(height: 4),
          Text(
            param.description,
            style: TextStyle(
              fontSize: 12,
              color: Colors.grey[600],
            ),
          ),
          const SizedBox(height: 8),
          TextField(
            controller: controller,
            decoration: InputDecoration(
              border: const OutlineInputBorder(),
              hintText: _getHintForType(param),
              helperText: _getConstraintsText(param),
              helperMaxLines: 2,
            ),
          ),
        ],
      ),
    );
  }

  String _getHintForType(ParameterMetadata param) {
    switch (param.typeName) {
      case 'string':
        return 'Enter text';
      case 'bool':
        return 'true or false';
      default:
        if (param.typeName.startsWith('uint') ||
            param.typeName.startsWith('int')) {
          return 'Enter number';
        }
        if (param.typeName.startsWith('float')) {
          return 'Enter decimal number';
        }
        return 'Enter value';
    }
  }

  String? _getConstraintsText(ParameterMetadata param) {
    if (param.constraints == null) return null;

    final parts = <String>[];
    final c = param.constraints!;

    if (c['min'] != null) parts.add('min: ${c['min']}');
    if (c['max'] != null) parts.add('max: ${c['max']}');
    if (c['min_length'] != null) parts.add('min length: ${c['min_length']}');
    if (c['max_length'] != null) parts.add('max length: ${c['max_length']}');
    if (c['pattern'] != null) parts.add('pattern: ${c['pattern']}');

    return parts.isEmpty ? null : parts.join(', ');
  }

  Future<void> _invokeFunction(FunctionMetadata func) async {
    // TODO: Implement actual FFI function invocation
    // For now, just add to call history
    final record = FfiCallRecord(
      timestamp: DateTime.now(),
      domain: _selectedDomain!,
      function: func.name,
      parameters: Map.fromEntries(
        func.parameters.map((p) => MapEntry(
              p.name,
              _paramControllers[p.name]!.text,
            )),
      ),
      result: 'Not implemented yet',
      success: false,
    );

    setState(() {
      _callHistory.insert(0, record);
    });

    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Function invocation not yet implemented'),
        backgroundColor: Colors.orange,
      ),
    );
  }

  Widget _buildCallHistory() {
    if (_callHistory.isEmpty) {
      return const Center(
        child: Text('No function calls yet'),
      );
    }

    return ListView.builder(
      itemCount: _callHistory.length,
      itemBuilder: (context, index) {
        final record = _callHistory[index];
        return Card(
          margin: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          child: ExpansionTile(
            leading: Icon(
              record.success ? Icons.check_circle : Icons.error,
              color: record.success ? Colors.green : Colors.red,
            ),
            title: Text('${record.domain}.${record.function}'),
            subtitle: Text(
              'Called at ${record.timestamp.toLocal().toString().substring(11, 19)}',
            ),
            children: [
              Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      'Parameters:',
                      style: TextStyle(fontWeight: FontWeight.bold),
                    ),
                    const SizedBox(height: 8),
                    Text(
                      jsonEncode(record.parameters),
                      style: const TextStyle(fontFamily: 'monospace'),
                    ),
                    const SizedBox(height: 16),
                    const Text(
                      'Result:',
                      style: TextStyle(fontWeight: FontWeight.bold),
                    ),
                    const SizedBox(height: 8),
                    Text(
                      record.result,
                      style: const TextStyle(fontFamily: 'monospace'),
                    ),
                  ],
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildEventMonitor() {
    if (_eventLog.isEmpty) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.monitor_heart_outlined, size: 64),
            SizedBox(height: 16),
            Text('No events captured yet'),
            SizedBox(height: 8),
            Text(
              'Events will appear here when emitted by FFI functions',
              style: TextStyle(color: Colors.grey),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      itemCount: _eventLog.length,
      itemBuilder: (context, index) {
        final event = _eventLog[index];
        return ListTile(
          leading: const Icon(Icons.notifications_active),
          title: Text(event.name),
          subtitle: Text(
            'Received at ${event.timestamp.toLocal().toString()}',
          ),
          trailing: IconButton(
            icon: const Icon(Icons.info_outline),
            onPressed: () {
              _showEventDetails(event);
            },
          ),
        );
      },
    );
  }

  void _showEventDetails(FfiEventRecord event) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(event.name),
        content: SingleChildScrollView(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              const Text(
                'Payload:',
                style: TextStyle(fontWeight: FontWeight.bold),
              ),
              const SizedBox(height: 8),
              Text(
                jsonEncode(event.payload),
                style: const TextStyle(fontFamily: 'monospace'),
              ),
              const SizedBox(height: 16),
              const Text(
                'Timestamp:',
                style: TextStyle(fontWeight: FontWeight.bold),
              ),
              Text(event.timestamp.toLocal().toString()),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }
}

/// FFI call record for history
class FfiCallRecord {
  final DateTime timestamp;
  final String domain;
  final String function;
  final Map<String, String> parameters;
  final String result;
  final bool success;

  FfiCallRecord({
    required this.timestamp,
    required this.domain,
    required this.function,
    required this.parameters,
    required this.result,
    required this.success,
  });
}

/// FFI event record for monitoring
class FfiEventRecord {
  final DateTime timestamp;
  final String name;
  final Map<String, dynamic> payload;

  FfiEventRecord({
    required this.timestamp,
    required this.name,
    required this.payload,
  });
}

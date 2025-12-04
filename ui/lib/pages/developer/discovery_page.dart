/// Discovery page for device profile discovery wizard.
///
/// Provides step-by-step wizard flow for discovering keyboard layouts,
/// with key press feedback, progress indicator, and save/cancel options.
library;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../ffi/bridge.dart';
import '../../services/facade/keyrx_facade.dart';

/// Discovery page for creating device profiles through guided wizard.
class DiscoveryPage extends StatefulWidget {
  const DiscoveryPage({super.key});

  @override
  State<DiscoveryPage> createState() => _DiscoveryPageState();
}

enum _DiscoveryStep { selectDevice, configureLayout, discovering, complete }

class _DiscoveryPageState extends State<DiscoveryPage> {
  _DiscoveryStep _currentStep = _DiscoveryStep.selectDevice;
  List<KeyboardDevice> _devices = [];
  KeyboardDevice? _selectedDevice;
  bool _isLoading = false;
  String? _error;

  // Layout configuration
  int _rows = 5;
  final List<int> _colsPerRow = [14, 14, 13, 12, 10]; // QWERTY layout default

  // Discovery state
  int _totalKeys = 0;
  int _discoveredKeys = 0;

  @override
  void initState() {
    super.initState();
    _loadDevices();
  }

  Future<void> _loadDevices() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    final facade = context.read<KeyrxFacade>();
    final result = await facade.listDevices();

    if (!mounted) return;

    await result.when(
      ok: (devices) async {
        setState(() {
          _isLoading = false;
          _devices = devices;
        });
      },
      err: (error) async {
        setState(() {
          _isLoading = false;
          _error = error.userMessage;
        });
      },
    );
  }

  Future<void> _startDiscovery() async {
    if (_selectedDevice == null) return;

    final facade = context.read<KeyrxFacade>();
    final result = await facade.startDiscovery(
      device: _selectedDevice!,
      rows: _rows,
      colsPerRow: _colsPerRow.take(_rows).toList(),
    );

    if (!mounted) return;

    await result.when(
      ok: (_) async {
        setState(() {
          _currentStep = _DiscoveryStep.discovering;
          _totalKeys = _colsPerRow.take(_rows).fold(0, (a, b) => a + b);
          _discoveredKeys = 0;
        });
      },
      err: (error) async {
        setState(() => _error = error.userMessage);
      },
    );
  }

  void _completeDiscovery() {
    setState(() => _currentStep = _DiscoveryStep.complete);
  }

  void _resetWizard() {
    setState(() {
      _currentStep = _DiscoveryStep.selectDevice;
      _selectedDevice = null;
      _discoveredKeys = 0;
      _error = null;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Device Discovery'),
        actions: [
          if (_currentStep != _DiscoveryStep.selectDevice)
            TextButton(
              onPressed: _resetWizard,
              child: const Text('Start Over'),
            ),
        ],
      ),
      body: _buildBody(),
    );
  }

  Widget _buildBody() {
    if (_error != null && _currentStep == _DiscoveryStep.selectDevice) {
      return _buildErrorState();
    }

    switch (_currentStep) {
      case _DiscoveryStep.selectDevice:
        return _buildDeviceSelection();
      case _DiscoveryStep.configureLayout:
        return _buildLayoutConfiguration();
      case _DiscoveryStep.discovering:
        return _buildDiscoveryProgress();
      case _DiscoveryStep.complete:
        return _buildComplete();
    }
  }

  Widget _buildErrorState() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.error_outline, size: 64, color: Colors.red[300]),
          const SizedBox(height: 16),
          Text('Error loading devices', style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 8),
          Text(_error!, style: TextStyle(color: Colors.grey[500])),
          const SizedBox(height: 24),
          FilledButton.icon(
            onPressed: _loadDevices,
            icon: const Icon(Icons.refresh),
            label: const Text('Retry'),
          ),
        ],
      ),
    );
  }

  Widget _buildDeviceSelection() {
    if (_isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _buildStepHeader(1, 'Select Device', 'Choose the keyboard to discover'),
          const SizedBox(height: 24),
          if (_devices.isEmpty)
            _buildEmptyDevices()
          else
            Expanded(
              child: ListView.builder(
                itemCount: _devices.length,
                itemBuilder: (context, index) => _buildDeviceTile(_devices[index]),
              ),
            ),
          const SizedBox(height: 16),
          FilledButton(
            onPressed: _selectedDevice == null
                ? null
                : () => setState(() => _currentStep = _DiscoveryStep.configureLayout),
            child: const Text('Next: Configure Layout'),
          ),
        ],
      ),
    );
  }

  Widget _buildEmptyDevices() {
    return Expanded(
      child: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.keyboard, size: 48, color: Colors.grey[400]),
            const SizedBox(height: 16),
            const Text('No devices found'),
            const SizedBox(height: 8),
            Text(
              'Make sure your keyboard is connected',
              style: TextStyle(color: Colors.grey[500]),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildDeviceTile(KeyboardDevice device) {
    final isSelected = _selectedDevice?.path == device.path;

    return Card(
      color: isSelected ? Theme.of(context).colorScheme.primaryContainer : null,
      child: ListTile(
        leading: Icon(
          Icons.keyboard,
          color: isSelected ? Theme.of(context).colorScheme.primary : null,
        ),
        title: Text(device.name),
        subtitle: Text(device.path, style: Theme.of(context).textTheme.bodySmall),
        trailing: isSelected ? const Icon(Icons.check_circle, color: Colors.green) : null,
        onTap: () => setState(() => _selectedDevice = device),
      ),
    );
  }

  Widget _buildLayoutConfiguration() {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _buildStepHeader(2, 'Configure Layout', 'Set keyboard dimensions'),
          const SizedBox(height: 24),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Number of Rows', style: Theme.of(context).textTheme.titleSmall),
                  const SizedBox(height: 8),
                  Slider(
                    value: _rows.toDouble(),
                    min: 3,
                    max: 8,
                    divisions: 5,
                    label: '$_rows rows',
                    onChanged: (value) {
                      setState(() {
                        _rows = value.toInt();
                        while (_colsPerRow.length < _rows) {
                          _colsPerRow.add(14);
                        }
                      });
                    },
                  ),
                  const SizedBox(height: 16),
                  Text('Columns per Row', style: Theme.of(context).textTheme.titleSmall),
                  const SizedBox(height: 8),
                  ...List.generate(_rows, (i) => _buildRowConfig(i)),
                ],
              ),
            ),
          ),
          const Spacer(),
          if (_error != null) _buildInlineError(),
          Row(
            children: [
              Expanded(
                child: OutlinedButton(
                  onPressed: () => setState(() => _currentStep = _DiscoveryStep.selectDevice),
                  child: const Text('Back'),
                ),
              ),
              const SizedBox(width: 16),
              Expanded(
                child: FilledButton(
                  onPressed: _startDiscovery,
                  child: const Text('Start Discovery'),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildRowConfig(int rowIndex) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          SizedBox(width: 60, child: Text('Row ${rowIndex + 1}')),
          Expanded(
            child: Slider(
              value: _colsPerRow[rowIndex].toDouble(),
              min: 5,
              max: 20,
              divisions: 15,
              label: '${_colsPerRow[rowIndex]}',
              onChanged: (value) => setState(() => _colsPerRow[rowIndex] = value.toInt()),
            ),
          ),
          SizedBox(width: 30, child: Text('${_colsPerRow[rowIndex]}')),
        ],
      ),
    );
  }

  Widget _buildDiscoveryProgress() {
    final progress = _totalKeys > 0 ? _discoveredKeys / _totalKeys : 0.0;

    return Padding(
      padding: const EdgeInsets.all(32),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.keyboard, size: 80, color: Theme.of(context).colorScheme.primary),
          const SizedBox(height: 32),
          Text('Press each key on your keyboard', style: Theme.of(context).textTheme.titleLarge),
          const SizedBox(height: 8),
          Text(
            'Discovery will detect the layout automatically',
            style: TextStyle(color: Colors.grey[600]),
          ),
          const SizedBox(height: 32),
          LinearProgressIndicator(value: progress, minHeight: 8),
          const SizedBox(height: 16),
          Text('$_discoveredKeys / $_totalKeys keys discovered'),
          const SizedBox(height: 48),
          OutlinedButton(
            onPressed: _completeDiscovery,
            child: const Text('Complete Discovery'),
          ),
        ],
      ),
    );
  }

  Widget _buildComplete() {
    return Padding(
      padding: const EdgeInsets.all(32),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.check_circle, size: 80, color: Colors.green),
          const SizedBox(height: 32),
          Text('Discovery Complete!', style: Theme.of(context).textTheme.titleLarge),
          const SizedBox(height: 8),
          Text(
            'Profile for ${_selectedDevice?.name ?? "device"} has been created',
            style: TextStyle(color: Colors.grey[600]),
          ),
          const SizedBox(height: 48),
          FilledButton(
            onPressed: _resetWizard,
            child: const Text('Discover Another Device'),
          ),
        ],
      ),
    );
  }

  Widget _buildStepHeader(int step, String title, String subtitle) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            CircleAvatar(
              radius: 16,
              backgroundColor: Theme.of(context).colorScheme.primary,
              child: Text('$step', style: const TextStyle(color: Colors.white)),
            ),
            const SizedBox(width: 12),
            Text(title, style: Theme.of(context).textTheme.titleLarge),
          ],
        ),
        const SizedBox(height: 8),
        Padding(
          padding: const EdgeInsets.only(left: 44),
          child: Text(subtitle, style: TextStyle(color: Colors.grey[600])),
        ),
      ],
    );
  }

  Widget _buildInlineError() {
    return Container(
      padding: const EdgeInsets.all(12),
      margin: const EdgeInsets.only(bottom: 16),
      decoration: BoxDecoration(
        color: Colors.red.shade50,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          const Icon(Icons.error, color: Colors.red, size: 20),
          const SizedBox(width: 8),
          Expanded(child: Text(_error!, style: const TextStyle(color: Colors.red))),
        ],
      ),
    );
  }
}

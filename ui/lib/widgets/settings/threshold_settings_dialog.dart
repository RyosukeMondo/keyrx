import 'package:flutter/material.dart';

import '../../services/metrics_service.dart';

/// Dialog for configuring metric thresholds.
class ThresholdSettingsDialog extends StatefulWidget {
  const ThresholdSettingsDialog({
    required this.initialThresholds,
    required this.onSave,
    super.key,
  });

  final MetricsThresholds initialThresholds;
  final void Function(MetricsThresholds) onSave;

  @override
  State<ThresholdSettingsDialog> createState() =>
      _ThresholdSettingsDialogState();
}

class _ThresholdSettingsDialogState extends State<ThresholdSettingsDialog> {
  late TextEditingController _latencyWarnController;
  late TextEditingController _latencyErrorController;
  late TextEditingController _memoryWarnController;
  late TextEditingController _memoryErrorController;

  @override
  void initState() {
    super.initState();
    _latencyWarnController = TextEditingController(
      text: widget.initialThresholds.latencyWarningUs.toString(),
    );
    _latencyErrorController = TextEditingController(
      text: widget.initialThresholds.latencyErrorUs.toString(),
    );
    _memoryWarnController = TextEditingController(
      text: (widget.initialThresholds.memoryWarningBytes ~/ (1024 * 1024))
          .toString(),
    );
    _memoryErrorController = TextEditingController(
      text: (widget.initialThresholds.memoryErrorBytes ~/ (1024 * 1024))
          .toString(),
    );
  }

  @override
  void dispose() {
    _latencyWarnController.dispose();
    _latencyErrorController.dispose();
    _memoryWarnController.dispose();
    _memoryErrorController.dispose();
    super.dispose();
  }

  void _save() {
    final latencyWarn = int.tryParse(_latencyWarnController.text);
    final latencyError = int.tryParse(_latencyErrorController.text);
    final memoryWarn = int.tryParse(_memoryWarnController.text);
    final memoryError = int.tryParse(_memoryErrorController.text);

    if (latencyWarn == null ||
        latencyError == null ||
        memoryWarn == null ||
        memoryError == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Please enter valid numbers for all thresholds'),
          backgroundColor: Colors.red,
        ),
      );
      return;
    }

    if (latencyWarn >= latencyError) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Latency warning must be less than error threshold'),
          backgroundColor: Colors.red,
        ),
      );
      return;
    }

    if (memoryWarn >= memoryError) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Memory warning must be less than error threshold'),
          backgroundColor: Colors.red,
        ),
      );
      return;
    }

    final thresholds = MetricsThresholds(
      latencyWarningUs: latencyWarn,
      latencyErrorUs: latencyError,
      memoryWarningBytes: memoryWarn * 1024 * 1024,
      memoryErrorBytes: memoryError * 1024 * 1024,
    );

    widget.onSave(thresholds);
    Navigator.of(context).pop();
  }

  @override
  Widget build(BuildContext context) => AlertDialog(
    title: const Text('Configure Thresholds'),
    content: SingleChildScrollView(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Latency Thresholds (microseconds)',
            style: TextStyle(fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _latencyWarnController,
            decoration: const InputDecoration(
              labelText: 'Warning',
              suffixText: 'μs',
              border: OutlineInputBorder(),
            ),
            keyboardType: TextInputType.number,
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _latencyErrorController,
            decoration: const InputDecoration(
              labelText: 'Error',
              suffixText: 'μs',
              border: OutlineInputBorder(),
            ),
            keyboardType: TextInputType.number,
          ),
          const SizedBox(height: 24),
          const Text(
            'Memory Thresholds (megabytes)',
            style: TextStyle(fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _memoryWarnController,
            decoration: const InputDecoration(
              labelText: 'Warning',
              suffixText: 'MB',
              border: OutlineInputBorder(),
            ),
            keyboardType: TextInputType.number,
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _memoryErrorController,
            decoration: const InputDecoration(
              labelText: 'Error',
              suffixText: 'MB',
              border: OutlineInputBorder(),
            ),
            keyboardType: TextInputType.number,
          ),
        ],
      ),
    ),
    actions: [
      TextButton(
        onPressed: () => Navigator.of(context).pop(),
        child: const Text('Cancel'),
      ),
      ElevatedButton(onPressed: _save, child: const Text('Save')),
    ],
  );
}

import 'package:flutter/material.dart';

/// Developer tool destination.
enum DeveloperTool {
  debugger('Debugger', Icons.bug_report_outlined, Icons.bug_report),
  console('Console', Icons.terminal_outlined, Icons.terminal),
  testRunner('Test Runner', Icons.science_outlined, Icons.science),
  simulator('Simulator', Icons.keyboard_outlined, Icons.keyboard),
  analyzer('Analyzer', Icons.analytics_outlined, Icons.analytics),
  profiler(
    'Profiler',
    Icons.local_fire_department_outlined,
    Icons.local_fire_department,
  ),
  benchmark('Benchmark', Icons.speed_outlined, Icons.speed),
  doctor('Doctor', Icons.health_and_safety_outlined, Icons.health_and_safety),
  discovery('Discovery', Icons.explore_outlined, Icons.explore);

  const DeveloperTool(this.label, this.icon, this.selectedIcon);

  final String label;
  final IconData icon;
  final IconData selectedIcon;
}

/// Developer tools navigation drawer widget.
class DeveloperDrawer extends StatelessWidget {
  final DeveloperTool? selectedTool;
  final ValueChanged<DeveloperTool> onToolSelected;
  final VoidCallback? onClose;

  const DeveloperDrawer({
    super.key,
    this.selectedTool,
    required this.onToolSelected,
    this.onClose,
  });

  @override
  Widget build(BuildContext context) {
    return NavigationDrawer(
      selectedIndex: selectedTool?.index,
      onDestinationSelected: (index) {
        onToolSelected(DeveloperTool.values[index]);
      },
      children: [
        _buildHeader(context),
        const Divider(indent: 16, endIndent: 16),
        ...DeveloperTool.values.map(_buildDestination),
      ],
    );
  }

  Widget _buildHeader(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 16, 8, 8),
      child: Row(
        children: [
          const Icon(Icons.developer_mode, size: 28),
          const SizedBox(width: 12),
          Text(
            'Developer Tools',
            style: Theme.of(context).textTheme.titleLarge,
          ),
          const Spacer(),
          if (onClose != null)
            IconButton(
              icon: const Icon(Icons.close),
              onPressed: onClose,
              tooltip: 'Close',
            ),
        ],
      ),
    );
  }

  Widget _buildDestination(DeveloperTool tool) {
    return NavigationDrawerDestination(
      icon: Icon(tool.icon),
      selectedIcon: Icon(tool.selectedIcon),
      label: Text(tool.label),
    );
  }
}

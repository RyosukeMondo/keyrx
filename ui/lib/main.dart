/// KeyRx UI - Visual interface for the KeyRx input remapping engine.
library;

import 'package:flutter/material.dart';
import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:provider/provider.dart';

import 'pages/debugger.dart';
import 'pages/console.dart';
import 'pages/devices.dart';
import 'pages/developer/ffi_tools_page.dart';
import 'pages/developer/test_runner_page.dart';
import 'pages/developer/profiler_page.dart';
import 'pages/layouts.dart';
import 'pages/migration_prompt_page.dart';
import 'pages/mapping_page.dart';
import 'pages/trade_off_page.dart';
import 'pages/run_controls_page.dart';
import 'pages/calibration_page.dart';
import 'pages/metrics_dashboard.dart';
import 'pages/wiring.dart';
import 'services/service_registry.dart';
import 'state/app_state.dart';
import 'state/providers.dart';
import 'widgets/developer_drawer.dart';
import 'widgets/migration_wrapper.dart';
import 'services/facade/keyrx_facade.dart';
import 'services/facade/facade_state.dart';

void main() {
  runApp(MultiProvider(providers: createProviders(), child: const KeyrxApp()));
}

/// Main application widget.
class KeyrxApp extends StatelessWidget {
  const KeyrxApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'KeyRx',
      debugShowCheckedModeBanner: false,
      theme: ThemeData.dark(useMaterial3: true).copyWith(
        colorScheme: ColorScheme.fromSeed(
          seedColor: Colors.blue,
          brightness: Brightness.dark,
        ),
      ),
      home: Builder(
        builder: (context) => MigrationWrapper(
          onCheckMigrationNeeded: () => _checkMigrationNeeded(context),
          onRunMigration: () => _runMigration(context),
          child: const HomePage(),
        ),
      ),
    );
  }

  /// Check if migration is needed by checking for old profiles directory.
  Future<bool> _checkMigrationNeeded(BuildContext context) async {
    final registry = context.read<ServiceRegistry>();
    final oldPath = _resolveOldProfilesPath();

    if (!await Directory(oldPath).exists()) {
      return false;
    }

    return registry.bridge.checkMigrationNeeded(oldPath);
  }

  /// Run the migration process.
  Future<MigrationReport> _runMigration(BuildContext context) async {
    final registry = context.read<ServiceRegistry>();
    final oldPath = _resolveOldProfilesPath();
    final newPath = registry.storagePathResolver.resolveProfilesPath();

    return registry.bridge.runMigration(oldPath, newPath);
  }

  String _resolveOldProfilesPath() {
    // Legacy path: ~/.config/keyrx/device_profiles
    final env = Platform.environment;
    final home = Platform.isWindows
        ? env['USERPROFILE'] ?? env['HOME']
        : env['HOME'] ?? env['USERPROFILE'];

    if (home == null || home.isEmpty) {
      // Should not happen, but safe fallback
      return '';
    }

    // This is hardcoded to the legacy location on Linux/Mac.
    // On Windows, the legacy location might differ, but assuming XDG-like structure for now
    // based on project history. Adjust if Windows legacy path was different.
    return p.join(home, '.config', 'keyrx', 'device_profiles');
  }
}

/// Home page with navigation.
class HomePage extends StatefulWidget {
  const HomePage({super.key});

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  int _selectedIndex = 0;
  DeveloperTool? _selectedDevTool;
  final GlobalKey<ScaffoldState> _scaffoldKey = GlobalKey<ScaffoldState>();

  List<Widget> _buildPages(ServiceRegistry registry) {
    return [
      const RunControlsPage(),
      DevicesPage(
        runtimeService: registry.runtimeService,
        hardwareService: registry.hardwareService,
        keymapService: registry.keymapService,
        deviceRegistryService: registry.deviceRegistryService,
      ),
      const LayoutsPage(),
      const WiringPage(),
      const MappingPage(),
    ];
  }

  final List<NavigationDestination> _destinations = const [
    NavigationDestination(
      icon: Icon(Icons.dashboard_outlined),
      selectedIcon: Icon(Icons.dashboard),
      label: 'Dashboard',
    ),
    NavigationDestination(
      icon: Icon(Icons.keyboard_outlined),
      selectedIcon: Icon(Icons.keyboard),
      label: 'Devices',
    ),
    NavigationDestination(
      icon: Icon(Icons.view_quilt_outlined),
      selectedIcon: Icon(Icons.view_quilt),
      label: 'Layouts',
    ),
    NavigationDestination(
      icon: Icon(Icons.cable_outlined),
      selectedIcon: Icon(Icons.cable),
      label: 'Wiring',
    ),
    NavigationDestination(
      icon: Icon(Icons.grid_view_outlined),
      selectedIcon: Icon(Icons.grid_view),
      label: 'Mapping',
    ),
  ];

  @override
  void initState() {
    super.initState();
    // Load developer mode on startup (engine will init on demand)
    WidgetsBinding.instance.addPostFrameCallback((_) {
      context.read<AppState>().loadDeveloperMode();
    });
  }

  @override
  Widget build(BuildContext context) {
    final registry = context.read<ServiceRegistry>();
    final appState = context.watch<AppState>();
    final pages = _buildPages(registry);

    return Scaffold(
      key: _scaffoldKey,
      endDrawer: DeveloperDrawer(
        selectedTool: _selectedDevTool,
        onToolSelected: _onDevToolSelected,
        onClose: () => _scaffoldKey.currentState?.closeEndDrawer(),
      ),
      body: Row(
        children: [
          NavigationRail(
            selectedIndex: _selectedDevTool == null ? _selectedIndex : null,
            onDestinationSelected: (index) {
              setState(() {
                _selectedIndex = index;
                _selectedDevTool = null;
              });
            },
            labelType: NavigationRailLabelType.all,
            leading: const Padding(
              padding: EdgeInsets.symmetric(vertical: 16),
              child: Icon(Icons.keyboard_alt, size: 32),
            ),
            trailing: Expanded(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  if (appState.isDeveloperMode)
                    IconButton(
                      icon: Icon(
                        _selectedDevTool != null
                            ? Icons.developer_mode
                            : Icons.developer_mode_outlined,
                      ),
                      tooltip: 'Developer Tools',
                      onPressed: () =>
                          _scaffoldKey.currentState?.openEndDrawer(),
                    ),
                  const SizedBox(height: 16),
                ],
              ),
            ),
            destinations: _destinations
                .map(
                  (d) => NavigationRailDestination(
                    icon: d.icon,
                    selectedIcon: d.selectedIcon,
                    label: Text(d.label),
                  ),
                )
                .toList(),
          ),
          const VerticalDivider(thickness: 1, width: 1),
          Expanded(child: _buildCurrentPage(registry, pages)),
        ],
      ),
      bottomNavigationBar: _buildStatusBar(),
    );
  }

  void _onDevToolSelected(DeveloperTool tool) {
    setState(() {
      _selectedDevTool = tool;
    });
    _scaffoldKey.currentState?.closeEndDrawer();
  }

  Widget _buildCurrentPage(ServiceRegistry registry, List<Widget> pages) {
    if (_selectedDevTool == null) {
      return IndexedStack(index: _selectedIndex, children: pages);
    }
    return _buildDevToolPage(registry);
  }

  Widget _buildDevToolPage(ServiceRegistry registry) {
    switch (_selectedDevTool!) {
      case DeveloperTool.debugger:
        return const DebuggerPage();
      case DeveloperTool.console:
        return const ConsolePage();
      case DeveloperTool.testRunner:
        return const TestRunnerPage();
      case DeveloperTool.profiler:
        return const ProfilerPage();
      case DeveloperTool.benchmark:
        return const MetricsDashboardPage();
      case DeveloperTool.doctor:
        return const CalibrationPage();
      case DeveloperTool.analyzer:
        return const TradeOffVisualizerPage();
      case DeveloperTool.ffiTools:
        return const FfiToolsPage();
      case DeveloperTool.simulator:
      case DeveloperTool.discovery:
        return _buildPlaceholderPage(_selectedDevTool!.label);
    }
  }

  Widget _buildPlaceholderPage(String title) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.construction, size: 64, color: Colors.grey[600]),
          const SizedBox(height: 16),
          Text(
            '$title - Coming Soon',
            style: Theme.of(context).textTheme.headlineSmall,
          ),
          const SizedBox(height: 8),
          Text(
            'This developer tool is under construction.',
            style: TextStyle(color: Colors.grey[500]),
          ),
        ],
      ),
    );
  }

  Widget _buildStatusBar() {
    return Consumer<AppState>(
      builder: (context, appState, _) {
        final facade = context.read<KeyrxFacade>();

        return Container(
          height: 24,
          color: Colors.black87,
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Row(
            children: [
              StreamBuilder<FacadeState>(
                stream: facade.stateStream,
                initialData: facade.currentState,
                builder: (context, snapshot) {
                  final state = snapshot.data ?? facade.currentState;
                  final statusText = switch (state.engine) {
                    EngineStatus.running => 'Engine Running',
                    EngineStatus.ready => 'Engine Ready',
                    EngineStatus.uninitialized => 'Engine Not Ready',
                    EngineStatus.initializing => 'Initializing...',
                    EngineStatus.loading => 'Loading Script...',
                    EngineStatus.stopping => 'Stopping...',
                    EngineStatus.paused => 'Engine Paused',
                    EngineStatus.error => 'Engine Error',
                  };

                  final color = switch (state.engine) {
                    EngineStatus.running => Colors.green,
                    EngineStatus.ready => Colors.blue,
                    EngineStatus.error => Colors.red,
                    _ => Colors.grey,
                  };

                  return Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(
                        state.engine == EngineStatus.error
                            ? Icons.error
                            : Icons.check_circle,
                        size: 14,
                        color: color,
                      ),
                      const SizedBox(width: 8),
                      Text(
                        statusText,
                        style: TextStyle(fontSize: 12, color: color),
                      ),
                    ],
                  );
                },
              ),
              const Spacer(),
              if (appState.loadedScript != null)
                Text(
                  'Script: ${appState.loadedScript}',
                  style: const TextStyle(fontSize: 12),
                ),
              const SizedBox(width: 16),
              InkWell(
                onTap: appState.toggleDeveloperMode,
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(
                      appState.isDeveloperMode
                          ? Icons.developer_mode
                          : Icons.developer_mode_outlined,
                      size: 14,
                      color: appState.isDeveloperMode
                          ? Colors.amber
                          : Colors.grey,
                    ),
                    const SizedBox(width: 4),
                    Text(
                      'Dev',
                      style: TextStyle(
                        fontSize: 12,
                        color: appState.isDeveloperMode
                            ? Colors.amber
                            : Colors.grey,
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(width: 16),
              Text(
                'v${appState.version}',
                style: const TextStyle(fontSize: 12, color: Colors.grey),
              ),
            ],
          ),
        );
      },
    );
  }
}

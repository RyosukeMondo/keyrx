/// KeyRx UI - Visual interface for the KeyRx input remapping engine.

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'pages/editor_page.dart';
import 'pages/debugger.dart';
import 'pages/console.dart';
import 'pages/devices_page.dart';
import 'pages/developer/test_runner_page.dart';
import 'pages/run_controls_page.dart';
import 'pages/trade_off_visualizer.dart';
import 'services/service_registry.dart';
import 'services/facade/keyrx_facade.dart';
import 'state/app_state.dart';
import 'state/providers.dart';
import 'widgets/developer_drawer.dart';

void main() {
  runApp(
    MultiProvider(
      providers: createProviders(),
      child: const KeyrxApp(),
    ),
  );
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
      home: const HomePage(),
    );
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

  List<Widget> _buildPages(ServiceRegistry registry, KeyrxFacade facade) {
    return [
      EditorPage(
        facade: facade,
        mappingRepository: registry.mappingRepository,
      ),
      DevicesPage(facade: facade),
      const RunControlsPage(),
      const DebuggerPage(),
      const ConsolePage(),
      const TradeOffVisualizerPage(),
    ];
  }

  final List<NavigationDestination> _destinations = const [
    NavigationDestination(
      icon: Icon(Icons.edit_outlined),
      selectedIcon: Icon(Icons.edit),
      label: 'Editor',
    ),
    NavigationDestination(
      icon: Icon(Icons.keyboard_outlined),
      selectedIcon: Icon(Icons.keyboard),
      label: 'Devices',
    ),
    NavigationDestination(
      icon: Icon(Icons.play_circle_outlined),
      selectedIcon: Icon(Icons.play_circle),
      label: 'Run',
    ),
    NavigationDestination(
      icon: Icon(Icons.bug_report_outlined),
      selectedIcon: Icon(Icons.bug_report),
      label: 'Debugger',
    ),
    NavigationDestination(
      icon: Icon(Icons.terminal_outlined),
      selectedIcon: Icon(Icons.terminal),
      label: 'Console',
    ),
    NavigationDestination(
      icon: Icon(Icons.tune_outlined),
      selectedIcon: Icon(Icons.tune),
      label: 'Timing',
    ),
  ];

  @override
  void initState() {
    super.initState();
    // Initialize engine and load developer mode on startup
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _initializeEngine();
      context.read<AppState>().loadDeveloperMode();
    });
  }

  Future<void> _initializeEngine() async {
    final appState = context.read<AppState>();
    final success = await appState.initialize();

    if (!success && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Failed to initialize engine: ${appState.error}'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final registry = context.read<ServiceRegistry>();
    final facade = context.read<KeyrxFacade>();
    final appState = context.watch<AppState>();
    final pages = _buildPages(registry, facade);

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
      return pages[_selectedIndex];
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
      case DeveloperTool.simulator:
      case DeveloperTool.analyzer:
      case DeveloperTool.benchmark:
      case DeveloperTool.doctor:
      case DeveloperTool.replay:
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
        return Container(
          height: 24,
          color: Colors.black87,
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Row(
            children: [
              Icon(
                appState.initialized ? Icons.check_circle : Icons.error,
                size: 14,
                color: appState.initialized ? Colors.green : Colors.red,
              ),
              const SizedBox(width: 8),
              Text(
                appState.initialized ? 'Engine Ready' : 'Engine Not Ready',
                style: const TextStyle(fontSize: 12),
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
                      color:
                          appState.isDeveloperMode ? Colors.amber : Colors.grey,
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

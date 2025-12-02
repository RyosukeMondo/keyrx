/// KeyRx UI - Visual interface for the KeyRx input remapping engine.

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'pages/editor_page.dart';
import 'pages/debugger.dart';
import 'pages/console.dart';
import 'pages/training_screen.dart';
import 'pages/trade_off_visualizer.dart';
import 'services/service_registry.dart';
import 'state/app_state.dart';

void main() {
  runApp(
    MultiProvider(
      providers: [
        Provider<ServiceRegistry>(
          create: (_) => ServiceRegistry.real(),
          dispose: (_, registry) => registry.dispose(),
        ),
        ChangeNotifierProvider(
          create: (context) {
            final registry = context.read<ServiceRegistry>();
            return AppState(
              engineService: registry.engineService,
              errorTranslator: registry.errorTranslator,
            );
          },
        ),
      ],
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

  final List<Widget> _pages = const [
    TrainingScreen(),
    EditorPage(),
    DebuggerPage(),
    ConsolePage(),
    TradeOffVisualizerPage(),
  ];

  final List<NavigationDestination> _destinations = const [
    NavigationDestination(
      icon: Icon(Icons.graphic_eq_outlined),
      selectedIcon: Icon(Icons.graphic_eq),
      label: 'Training',
    ),
    NavigationDestination(
      icon: Icon(Icons.keyboard),
      selectedIcon: Icon(Icons.keyboard),
      label: 'Editor',
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
    // Initialize engine on startup
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _initializeEngine();
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
    return Scaffold(
      body: Row(
        children: [
          NavigationRail(
            selectedIndex: _selectedIndex,
            onDestinationSelected: (index) {
              setState(() {
                _selectedIndex = index;
              });
            },
            labelType: NavigationRailLabelType.all,
            leading: const Padding(
              padding: EdgeInsets.symmetric(vertical: 16),
              child: Icon(Icons.keyboard_alt, size: 32),
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
          Expanded(child: _pages[_selectedIndex]),
        ],
      ),
      bottomNavigationBar: _buildStatusBar(),
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

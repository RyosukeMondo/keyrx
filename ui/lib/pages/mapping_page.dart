import 'package:flutter/material.dart';
import 'mapping/mapping_dashboard.dart';

class MappingPage extends StatelessWidget {
  const MappingPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Mapping')),
      body: const MappingDashboard(),
    );
  }
}

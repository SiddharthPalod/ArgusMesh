import 'package:flutter/material.dart';

/// Widget for displaying loading state.
/// Single Responsibility: Loading display only.
class LoadingDisplay extends StatelessWidget {
  const LoadingDisplay({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Argus Mesh')),
      body: const Center(child: CircularProgressIndicator()),
    );
  }
}

import 'package:flutter/material.dart';

/// Placeholder page – simulation is disabled in the FRB-only build.
class SimPage extends StatelessWidget {
  const SimPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Mesh simulation'),
      ),
      body: const Center(
        child: Padding(
          padding: EdgeInsets.all(24),
          child: Text(
            'Mesh simulation via manual FFI has been disabled.\n'
            'This build uses flutter_rust_bridge v2 only.',
            textAlign: TextAlign.center,
          ),
        ),
      ),
    );
  }
}

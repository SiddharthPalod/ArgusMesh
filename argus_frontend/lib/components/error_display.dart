import 'package:flutter/material.dart';

/// Widget for displaying error messages.
/// Single Responsibility: Error display only.
class ErrorDisplay extends StatelessWidget {
  final String errorMessage;

  const ErrorDisplay({
    super.key,
    required this.errorMessage,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Argus Mesh')),
      body: SingleChildScrollView(
        child: Padding(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const SizedBox(height: 48),
              const Icon(Icons.warning_amber, size: 48, color: Colors.orange),
              const SizedBox(height: 16),
              Text(
                errorMessage,
                textAlign: TextAlign.center,
                style: const TextStyle(fontSize: 12),
              ),
              const SizedBox(height: 16),
              const Text(
                'Build the backend with: cargo build (from backend/)',
                textAlign: TextAlign.center,
                style: TextStyle(fontSize: 11, fontStyle: FontStyle.italic),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

import 'package:flutter/material.dart';
import 'package:argus_frontend/bridge_generated.dart/rust_api/definitions/routing/envelope.dart' show Priority;

/// Widget for creating alerts with different priority levels.
/// Single Responsibility: Alert creation UI only.
class AlertCreationButtons extends StatelessWidget {
  final Function(Priority) onCreateAlert;

  const AlertCreationButtons({
    super.key,
    required this.onCreateAlert,
  });

  @override
  Widget build(BuildContext context) {
    return Wrap(
      spacing: 8,
      children: [
        FilledButton.tonal(
          onPressed: () => onCreateAlert(Priority.critical),
          child: const Text('Critical'),
        ),
        FilledButton.tonal(
          onPressed: () => onCreateAlert(Priority.high),
          child: const Text('High'),
        ),
        FilledButton.tonal(
          onPressed: () => onCreateAlert(Priority.normal),
          child: const Text('Normal'),
        ),
        FilledButton.tonal(
          onPressed: () => onCreateAlert(Priority.low),
          child: const Text('Low'),
        ),
      ],
    );
  }
}

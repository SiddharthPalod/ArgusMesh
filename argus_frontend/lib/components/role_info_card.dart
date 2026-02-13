import 'package:flutter/material.dart';

/// Widget displaying role-specific information.
/// Single Responsibility: Display role info only.
class RoleInfoCard extends StatelessWidget {
  final String role;

  const RoleInfoCard({
    super.key,
    required this.role,
  });

  @override
  Widget build(BuildContext context) {
    if (role == 'relay') {
      return Card(
        color: Theme.of(context).colorScheme.tertiaryContainer,
        child: const Padding(
          padding: EdgeInsets.all(16),
          child: Row(
            children: [
              Icon(Icons.sync_alt, size: 20),
              SizedBox(width: 8),
              Expanded(
                child: Text(
                  'Relay mode: this node only forwards alerts from other nodes.',
                  style: TextStyle(fontSize: 12),
                ),
              ),
            ],
          ),
        ),
      );
    }

    if (role == 'command') {
      return Card(
        color: Theme.of(context).colorScheme.tertiaryContainer,
        child: const Padding(
          padding: EdgeInsets.all(16),
          child: Row(
            children: [
              Icon(Icons.dashboard, size: 20),
              SizedBox(width: 8),
              Expanded(
                child: Text(
                  'Command view: aggregated alert feed from all field nodes.',
                  style: TextStyle(fontSize: 12),
                ),
              ),
            ],
          ),
        ),
      );
    }

    return const SizedBox.shrink();
  }
}

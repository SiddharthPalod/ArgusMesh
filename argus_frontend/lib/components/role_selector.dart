import 'package:flutter/material.dart';

/// Segmented button widget for selecting the node's role in the mesh.
class RoleSelector extends StatelessWidget {
  final String currentRole;
  final ValueChanged<String> onChanged;

  const RoleSelector({
    super.key,
    required this.currentRole,
    required this.onChanged,
  });

  static const roles = ['field', 'relay', 'command'];

  static const _labels = {
    'field': 'Field Node',
    'relay': 'Relay Node',
    'command': 'Command',
  };

  static const _icons = {
    'field': Icons.sensors,
    'relay': Icons.sync_alt,
    'command': Icons.dashboard,
  };

  @override
  Widget build(BuildContext context) {
    return Card(
      color: Theme.of(context).colorScheme.secondaryContainer,
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Node Role',
                style: Theme.of(context).textTheme.titleSmall),
            const SizedBox(height: 8),
            SegmentedButton<String>(
              segments: roles
                  .map((r) => ButtonSegment(
                        value: r,
                        label: Text(_labels[r]!),
                        icon: Icon(_icons[r]),
                      ))
                  .toList(),
              selected: {currentRole},
              onSelectionChanged: (set) => onChanged(set.first),
              showSelectedIcon: false,
            ),
          ],
        ),
      ),
    );
  }
}

import 'package:flutter/material.dart';
import 'package:argus_frontend/models/node_state_model.dart';

/// Widget displaying the current node state.
/// Single Responsibility: Display node state only.
class NodeStateCard extends StatelessWidget {
  final NodeStateModel? state;

  const NodeStateCard({
    super.key,
    this.state,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Node state',
              style: Theme.of(context).textTheme.titleSmall,
            ),
            const SizedBox(height: 8),
            Text(
              state?.displayStatus ?? '—',
              style: Theme.of(context).textTheme.bodyLarge,
            ),
          ],
        ),
      ),
    );
  }
}

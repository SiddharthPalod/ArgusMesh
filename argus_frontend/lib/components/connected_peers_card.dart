import 'package:flutter/material.dart';

/// Widget displaying connected BLE peers.
/// Single Responsibility: Display connected peers only.
class ConnectedPeersCard extends StatelessWidget {
  final Map<String, dynamic> connectedPeers;

  const ConnectedPeersCard({
    super.key,
    required this.connectedPeers,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      color: Theme.of(context).colorScheme.primaryContainer,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Connected Peers (${connectedPeers.length})',
              style: Theme.of(context).textTheme.titleSmall,
            ),
            const SizedBox(height: 8),
            if (connectedPeers.isEmpty)
              const Text(
                'Scanning for nearby ArgusMesh nodes...',
                style: TextStyle(fontSize: 12, fontStyle: FontStyle.italic),
              )
            else
              ...connectedPeers.keys.map((id) => Padding(
                    padding: const EdgeInsets.only(bottom: 4),
                    child: Row(
                      children: [
                        const Icon(Icons.bluetooth_connected, size: 16),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(id, style: const TextStyle(fontSize: 12)),
                        ),
                      ],
                    ),
                  )),
          ],
        ),
      ),
    );
  }
}

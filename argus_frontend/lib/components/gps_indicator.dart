import 'package:flutter/material.dart';
import 'package:geolocator/geolocator.dart';

/// Widget displaying GPS status and current position.
/// Single Responsibility: Display GPS information only.
class GpsIndicator extends StatelessWidget {
  final bool gpsAvailable;
  final Position? lastPosition;

  const GpsIndicator({
    super.key,
    required this.gpsAvailable,
    this.lastPosition,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
        child: Row(
          children: [
            Icon(
              gpsAvailable ? Icons.gps_fixed : Icons.gps_off,
              size: 18,
              color: gpsAvailable ? Colors.green : Colors.grey,
            ),
            const SizedBox(width: 8),
            Text(
              gpsAvailable
                  ? '📍 GPS: ${lastPosition?.latitude.toStringAsFixed(4)}, '
                      '${lastPosition?.longitude.toStringAsFixed(4)}'
                  : '📍 GPS: unavailable',
              style: const TextStyle(fontSize: 12),
            ),
          ],
        ),
      ),
    );
  }
}

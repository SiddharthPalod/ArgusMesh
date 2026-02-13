import 'package:flutter/material.dart';
import 'package:flutter_map/flutter_map.dart';
import 'package:latlong2/latlong.dart';
import 'package:geolocator/geolocator.dart';
import '../models/alert_model.dart';

/// Map page showing alert markers on an OpenStreetMap tile layer.
class MapView extends StatefulWidget {
  final List<dynamic> alerts;

  const MapView({super.key, required this.alerts});

  @override
  State<MapView> createState() => _MapViewState();
}

class _MapViewState extends State<MapView> {
  LatLng _center = const LatLng(28.6139, 77.2090); // Default: New Delhi
  bool _locationReady = false;

  @override
  void initState() {
    super.initState();
    _fetchLocation();
  }

  Future<void> _fetchLocation() async {
    try {
      final pos = await Geolocator.getCurrentPosition(
        desiredAccuracy: LocationAccuracy.medium,
      );
      if (mounted) {
        setState(() {
          _center = LatLng(pos.latitude, pos.longitude);
          _locationReady = true;
        });
      }
    } catch (_) {
      // Fall back to default center; GPS might not be available.
      if (mounted) setState(() => _locationReady = true);
    }
  }

  /// Try to extract lat/lng from the alert's payload JSON.
  LatLng? _parseAlertLocation(dynamic alert) {
    if (alert is! Map) return null;
    final alertModel = AlertModel.fromMap(alert as Map<String, dynamic>);
    
    // The payload might be a JSON string with lat/lng embedded
    try {
      final payload = alertModel.payload;
      final lat = payload['lat'];
      final lng = payload['lng'];
      if (lat is num && lng is num) {
        return LatLng(lat.toDouble(), lng.toDouble());
      }
    } catch (_) {
      // Payload isn't a JSON with coordinates — skip
    }
    return null;
  }

  Color _priorityColor(String? priority) {
    switch (priority?.toLowerCase()) {
      case 'critical':
        return Colors.red;
      case 'high':
        return Colors.orange;
      case 'normal':
        return Colors.blue;
      case 'low':
        return Colors.grey;
      default:
        return Colors.purple;
    }
  }

  @override
  Widget build(BuildContext context) {
    // Build markers from alerts that have coordinates
    final markers = <Marker>[];
    for (final alert in widget.alerts) {
      final loc = _parseAlertLocation(alert);
      if (loc == null) continue;

      final alertModel = alert is Map<String, dynamic>
          ? AlertModel.fromMap(alert)
          : AlertModel.fromMap(<String, dynamic>{});
      final priority = alertModel.displayPriority;
      final sender = alertModel.displaySender;
      final msgId = alertModel.displayMsgId;
      final color = _priorityColor(priority);

      markers.add(Marker(
        point: loc,
        width: 36,
        height: 36,
        child: GestureDetector(
          onTap: () {
            showDialog(
              context: context,
              builder: (_) => AlertDialog(
                title: Text('Alert: $priority'),
                content: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('Sender: $sender'),
                    Text('ID: $msgId'),
                    Text('Location: ${loc.latitude.toStringAsFixed(5)}, '
                        '${loc.longitude.toStringAsFixed(5)}'),
                  ],
                ),
                actions: [
                  TextButton(
                    onPressed: () => Navigator.pop(context),
                    child: const Text('Close'),
                  ),
                ],
              ),
            );
          },
          child: Icon(
            Icons.location_on,
            color: color,
            size: 36,
          ),
        ),
      ));
    }

    if (!_locationReady) {
      return const Center(child: CircularProgressIndicator());
    }

    return FlutterMap(
      options: MapOptions(
        initialCenter: _center,
        initialZoom: 13.0,
      ),
      children: [
        TileLayer(
          urlTemplate: 'https://tile.openstreetmap.org/{z}/{x}/{y}.png',
          userAgentPackageName: 'com.example.argus_frontend',
        ),
        MarkerLayer(markers: markers),
        // Show device location
        MarkerLayer(
          markers: [
            Marker(
              point: _center,
              width: 20,
              height: 20,
              child: Container(
                decoration: BoxDecoration(
                  color: Colors.blue.withOpacity(0.7),
                  shape: BoxShape.circle,
                  border: Border.all(color: Colors.white, width: 2),
                ),
              ),
            ),
          ],
        ),
      ],
    );
  }
}

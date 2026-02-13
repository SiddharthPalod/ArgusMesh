import 'package:geolocator/geolocator.dart';

/// Service responsible for GPS/location functionality.
/// Single Responsibility: GPS management only.
class GpsService {
  bool _gpsAvailable = false;
  Position? _lastPosition;

  bool get gpsAvailable => _gpsAvailable;
  Position? get lastPosition => _lastPosition;

  /// Initializes GPS and checks permissions.
  Future<void> initialize() async {
    try {
      final serviceEnabled = await Geolocator.isLocationServiceEnabled();
      if (!serviceEnabled) {
        _gpsAvailable = false;
        return;
      }

      var perm = await Geolocator.checkPermission();
      if (perm == LocationPermission.denied) {
        perm = await Geolocator.requestPermission();
      }
      if (perm == LocationPermission.denied ||
          perm == LocationPermission.deniedForever) {
        _gpsAvailable = false;
        return;
      }

      final pos = await Geolocator.getCurrentPosition(
        desiredAccuracy: LocationAccuracy.medium,
      );
      _gpsAvailable = true;
      _lastPosition = pos;
    } catch (_) {
      _gpsAvailable = false;
    }
  }

  /// Captures a fresh GPS position with high accuracy.
  /// Returns the position or the last cached position if capture fails.
  Future<Position?> capturePosition() async {
    try {
      final pos = await Geolocator.getCurrentPosition(
        desiredAccuracy: LocationAccuracy.high,
      ).timeout(const Duration(seconds: 5));
      _lastPosition = pos;
      _gpsAvailable = true;
      return pos;
    } catch (_) {
      return _lastPosition; // Use cached if fresh capture fails
    }
  }

  /// Updates the GPS state (for state management integration).
  void updateState(bool available, Position? position) {
    _gpsAvailable = available;
    _lastPosition = position;
  }
}

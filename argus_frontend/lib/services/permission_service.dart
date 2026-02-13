import 'package:permission_handler/permission_handler.dart';

/// Service responsible for handling runtime permissions.
/// Single Responsibility: Permission management only.
class PermissionService {
  /// Ensures all required permissions are granted.
  /// Returns null if successful, error message if permissions are denied.
  static Future<String?> ensurePermissions() async {
    // Only relevant on Android; other platforms simply skip.
    // permission_handler abstracts the platform check, so we can just request.
    final statuses = await [
      Permission.bluetoothScan,
      Permission.bluetoothConnect,
      Permission.locationWhenInUse,
    ].request();

    // On newer Android (12+), BLE lives under the "Nearby devices" group and is
    // guarded by BLUETOOTH_SCAN / BLUETOOTH_CONNECT.
    // On older Android, there is no "Nearby devices" group; BLE scans are
    // effectively gated by location permission instead.
    //
    // To support both generations cleanly, we accept either:
    // - the new BLE runtime permissions, OR
    // - location permission (which implies BLE scan access on older Android).
    final scanGranted =
        statuses[Permission.bluetoothScan]?.isGranted ?? false;
    final connectGranted =
        statuses[Permission.bluetoothConnect]?.isGranted ?? false;
    final locationGranted =
        statuses[Permission.locationWhenInUse]?.isGranted ?? false;

    final hasModernBlePermission = scanGranted && connectGranted;
    final hasLegacyBlePermission = locationGranted;

    if (!hasModernBlePermission && !hasLegacyBlePermission) {
      return 'Bluetooth / nearby-devices / location permissions are denied.\n'
          'On newer Android versions, enable "Nearby devices". On older '
          'versions, enable Location (and Bluetooth) for this app to allow BLE mesh.';
    }

    return null;
  }
}

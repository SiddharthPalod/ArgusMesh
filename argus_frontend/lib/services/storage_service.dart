import 'package:path_provider/path_provider.dart';
import 'package:argus_frontend/bridge_generated.dart/rust_api/definitions/core/api.dart' as api;

/// Service responsible for configuring storage paths.
/// Single Responsibility: Storage configuration only.
class StorageService {
  /// Configures the storage base directory for the Rust core.
  /// This allows alerts to persist where supported.
  static Future<void> configureStoragePath() async {
    try {
      final dir = await getApplicationSupportDirectory();
      await api.configureStorageBaseDir(baseDir: dir.path);
    } catch (_) {
      // If anything goes wrong here, we silently fall back to the default
      // in-memory / dev-time behavior.
    }
  }
}

import 'dart:convert';
import 'dart:typed_data';
import 'package:argus_frontend/bridge_generated.dart/rust_api/definitions/core/api.dart' as api;
import 'package:argus_frontend/bridge_generated.dart/rust_api/definitions/frb_generated.dart';
import 'dart:io' show Platform;
import 'package:flutter/foundation.dart' show kIsWeb;

/// Service responsible for mesh node operations and API calls.
/// Single Responsibility: Mesh API interactions only.
class MeshService {
  bool _initialized = false;

  /// Initializes the Rust library.
  /// On web platform, WASM build is required (backend.js/backend.wasm).
  /// On Android, native libraries are used.
  Future<void> initRustLib() async {
    print("MeshService: Initializing RustLib...");
    try {
      await RustLib.init();
      print("MeshService: RustLib initialized.");
    } catch (e) {
      if (kIsWeb) {
        print("MeshService: RustLib initialization failed on web - WASM files may be missing.");
        print("Error: $e");
        print("Note: To build WASM, run: cd backend && wasm-pack build --target web --out-dir ../argus_frontend/web/pkg");
        rethrow;
      } else {
        print("MeshService: RustLib initialization failed: $e");
        rethrow;
      }
    }
  }

  /// Initializes the mesh node with the given role tag.
  /// On Android, Flutter handles BLE. On other platforms, btleplug handles BLE.
  Future<void> startMeshNode(String role) async {
    if (!_initialized) {
      await api.initNode();
      _initialized = true;
    }
    await api.startMeshNode(tag: 'argus-$role');
  }

  /// Gets the current node state.
  Future<Map<String, dynamic>> getNodeState() async {
    final stateJson = await api.getNodeState();
    final decodedState = jsonDecode(stateJson);
    return decodedState is Map<String, dynamic>
        ? decodedState
        : Map<String, dynamic>.from(decodedState as Map);
  }

  /// Gets all known alerts.
  Future<List<dynamic>> getKnownAlerts() async {
    final alertsJson = await api.getKnownAlerts();
    final decodedAlerts = jsonDecode(alertsJson);
    return decodedAlerts is List ? decodedAlerts : [];
  }

  /// Creates a new alert.
  Future<String> createAlert(api.AlertInput input) async {
    return await api.createAlert(input: input);
  }

  /// Receives a BLE packet.
  /// On Android, this is called by Flutter when it receives BLE data.
  /// On other platforms, btleplug handles this internally.
  Future<void> receiveBlePacket(List<int> data) async {
    await api.receiveBlePacket(data: Uint8List.fromList(data));
  }
}

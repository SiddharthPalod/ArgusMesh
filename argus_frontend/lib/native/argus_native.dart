import 'dart:ffi';
import 'dart:io';

import 'argus_bindings.dart';

export 'argus_bindings.dart';

/// Load the Argus native library and return an [ArgusApi].
///
/// [path] overrides the default library path. If not set, uses:
/// - Windows: `backend/target/debug/backend.dll`
/// - macOS: `backend/target/debug/libbackend.dylib`
/// - Linux: `backend/target/debug/libbackend.so`
///
/// Build the Rust backend first from the repo root:
/// ```bash
/// cd backend && cargo build
/// ```
ArgusApi loadArgusNative({String? path}) {
  if (Platform.isAndroid) {
    return ArgusApi(DynamicLibrary.open('libbackend.so'));
  }

  final libPath = path ?? _defaultLibPath();
  if (libPath == null) {
    throw UnsupportedError('Unsupported platform: ${Platform.operatingSystem}');
  }
  return ArgusApi(DynamicLibrary.open(libPath));
}

String? _defaultLibPath() {
  if (Platform.isWindows) return 'backend/target/debug/backend.dll';
  if (Platform.isMacOS) return 'backend/target/debug/libbackend.dylib';
  if (Platform.isLinux) return 'backend/target/debug/libbackend.so';
  return null;
}


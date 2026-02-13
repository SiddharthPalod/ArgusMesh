import 'dart:ffi';
import 'dart:convert';
import 'package:ffi/ffi.dart';

typedef _InitNode = Pointer<Utf8> Function();
typedef _CreateAlert = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _GetState = Pointer<Utf8> Function();
typedef _GetAlerts = Pointer<Utf8> Function();
typedef _Free = Void Function(Pointer<Utf8>);
typedef _FreeDart = void Function(Pointer<Utf8>);
typedef _SimStartNative = Pointer<Utf8> Function(Int32);
typedef _SimStartDart = Pointer<Utf8> Function(int);
typedef _SimCreateAlertNative = Pointer<Utf8> Function(Int32, Pointer<Utf8>);
typedef _SimCreateAlertDart = Pointer<Utf8> Function(int, Pointer<Utf8>);
typedef _SimPropagate = Pointer<Utf8> Function();
typedef _SimGetAlertsNative = Pointer<Utf8> Function(Int32);
typedef _SimGetAlertsDart = Pointer<Utf8> Function(int);
typedef _SimNodeCountNative = Int32 Function();
typedef _SimNodeCountDart = int Function();
typedef _SimStopNative = Void Function();
typedef _SimStopDart = void Function();

class ArgusApi {
  ArgusApi(this.lib) {
    _resolve();
  }

  final DynamicLibrary lib;

  late Pointer<Utf8> Function() initNode;
  late Pointer<Utf8> Function(Pointer<Utf8>) createAlert;
  late Pointer<Utf8> Function() getState;
  late Pointer<Utf8> Function() getKnownAlerts;
  late void Function(Pointer<Utf8>) freeStr;

  late Pointer<Utf8> Function(int) _simStart;
  late Pointer<Utf8> Function(int, Pointer<Utf8>) _simCreateAlert;
  late Pointer<Utf8> Function() _simPropagate;
  late Pointer<Utf8> Function(int) _simGetAlerts;
  late int Function() _simNodeCount;
  late void Function() _simStop;

  void _resolve() {
    initNode = lib.lookupFunction<_InitNode, _InitNode>("argus_init_node");
    createAlert =
        lib.lookupFunction<_CreateAlert, _CreateAlert>("argus_create_alert");
    getState =
        lib.lookupFunction<_GetState, _GetState>("argus_get_node_state");
    getKnownAlerts =
        lib.lookupFunction<_GetAlerts, _GetAlerts>("argus_get_known_alerts");
    freeStr = lib.lookupFunction<_Free, _FreeDart>("argus_free_string");
    _simStart =
        lib.lookupFunction<_SimStartNative, _SimStartDart>("argus_sim_start");
    _simCreateAlert = lib.lookupFunction<_SimCreateAlertNative,
        _SimCreateAlertDart>("argus_sim_create_alert");
    _simPropagate =
        lib.lookupFunction<_SimPropagate, _SimPropagate>("argus_sim_propagate");
    _simGetAlerts = lib.lookupFunction<_SimGetAlertsNative,
        _SimGetAlertsDart>("argus_sim_get_alerts");
    _simNodeCount = lib.lookupFunction<_SimNodeCountNative,
        _SimNodeCountDart>("argus_sim_node_count");
    _simStop =
        lib.lookupFunction<_SimStopNative, _SimStopDart>("argus_sim_stop");
  }

  String _call0(Pointer<Utf8> Function() f) {
    final ptr = f();
    final s = ptr.toDartString();
    freeStr(ptr);
    return s;
  }

  String _call1(String json, Pointer<Utf8> Function(Pointer<Utf8>) f) {
    final c = json.toNativeUtf8();
    final ptr = f(c);
    malloc.free(c);
    final s = ptr.toDartString();
    freeStr(ptr);
    return s;
  }

  Map<String, dynamic> init() =>
      jsonDecode(_call0(initNode)) as Map<String, dynamic>;

  Map<String, dynamic> state() =>
      jsonDecode(_call0(getState)) as Map<String, dynamic>;

  List<dynamic> alerts() {
    final s = _call0(getKnownAlerts);
    final decoded = jsonDecode(s);
    if (decoded is Map && decoded.containsKey('error')) return [];
    if (decoded is List) return decoded;
    return [];
  }

  Map<String, dynamic> create(Map<String, dynamic> m) =>
      jsonDecode(_call1(jsonEncode(m), createAlert)) as Map<String, dynamic>;

  Map<String, dynamic> simStart(int n) {
    final ptr = _simStart(n);
    final s = ptr.toDartString();
    freeStr(ptr);
    return jsonDecode(s) as Map<String, dynamic>;
  }

  Map<String, dynamic> simCreateAlert(
          int nodeId, Map<String, dynamic> m) =>
      jsonDecode(
        _call1(
          jsonEncode(m),
          (p) => _simCreateAlert(nodeId, p),
        ),
      ) as Map<String, dynamic>;

  Map<String, dynamic> simPropagate() =>
      jsonDecode(_call0(_simPropagate)) as Map<String, dynamic>;

  List<dynamic> simGetAlerts(int nodeId) {
    final ptr = _simGetAlerts(nodeId);
    final s = ptr.toDartString();
    freeStr(ptr);
    final decoded = jsonDecode(s);
    if (decoded is Map && decoded.containsKey('error')) return [];
    if (decoded is List) return decoded;
    return [];
  }

  int simNodeCount() => _simNodeCount();

  void simStop() => _simStop();
}


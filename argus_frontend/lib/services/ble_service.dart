import 'dart:async';
import 'dart:convert';
import 'dart:io' show Platform;
import 'package:flutter/services.dart';
import 'package:flutter_blue_plus/flutter_blue_plus.dart';
import 'package:argus_frontend/bridge_generated.dart/rust_api/definitions/core/api.dart' as api;
import 'package:argus_frontend/bridge_generated.dart/core/api.dart' as core_api;

/// Service responsible for BLE operations (scanning, connecting, broadcasting).
/// Single Responsibility: BLE management only.
class BleService {
  // UUIDs matching the Kotlin GATT Server / Rust constants
  static final Guid _serviceGuid = Guid('12345678-90ab-cdef-1234-567890abcdef');
  static final Guid _charGuid = Guid('abcdef12-3456-7890-abcd-ef1234567890');

  static const platform = MethodChannel('argus_ble_channel');

  StreamSubscription? _scanSub;
  Timer? _outboundPollTimer;
  final Map<String, BluetoothDevice> _connectedPeers = {};
  final Set<String> _connectingPeers = {};
  final List<String> _peerLog = [];

  // Callbacks
  Function(String)? onPeerConnected;
  Function(String)? onPeerDisconnected;
  Function(String)? onPeerLog;

  Map<String, BluetoothDevice> get connectedPeers => Map.unmodifiable(_connectedPeers);
  List<String> get peerLog => List.unmodifiable(_peerLog);

  /// Initializes BLE (GATT Server + Advertising).
  /// On Android, Flutter handles BLE operations.
  /// On other platforms, btleplug handles BLE (this service is Android-specific).
  Future<void> initialize() async {
    // Only initialize Flutter BLE on Android - other platforms use btleplug in Rust
    if (!Platform.isAndroid) {
      print("BleService: Skipping Flutter BLE initialization on non-Android platform (using btleplug)");
      return;
    }
    // Setup method call handler to receive data from Kotlin
    platform.setMethodCallHandler((call) async {
      if (call.method == "onDataReceived") {
        List<int> bytes;
        if (call.arguments is Uint8List) {
          bytes = call.arguments as Uint8List;
        } else if (call.arguments is List) {
          bytes = (call.arguments as List).cast<int>();
        } else {
          print("Unknown data type received: ${call.arguments.runtimeType}");
          return;
        }

        print("Received BLE Data via Platform Channel: ${bytes.length} bytes");
        await api.receiveBlePacket(data: Uint8List.fromList(bytes));
      } else if (call.method == "onAdvertisingStarted") {
        final success = call.arguments as bool;
        print("Native BLE Advertising started: $success");
      }
    });

    // Start Native GATT Server (for Receiving)
    try {
      final bool success = await platform.invokeMethod('startGattServer');
      print("Native GATT Server started: $success");
    } catch (e) {
      print("Error starting Native GATT Server: $e");
    }

    // Start native BLE advertising (shares same BLE address as GATT server)
    try {
      final bool success = await platform.invokeMethod('startAdvertising');
      print("Native BLE Advertising requested: $success");
    } catch (e) {
      print("Error starting BLE advertising: $e");
    }

    // Start polling for outbound packets from Rust mesh core
    // On Android, Rust queues packets here because btleplug doesn't work
    _startOutboundPolling();
  }

  /// Polls Rust mesh core for outbound packets and sends them to connected peers.
  void _startOutboundPolling() {
    _outboundPollTimer?.cancel();
    _outboundPollTimer = Timer.periodic(const Duration(milliseconds: 200), (_) async {
      try {
        final packetOpt = await core_api.getNextOutboundPacket();
        if (packetOpt != null) {
          final packet = packetOpt as List<int>;
          if (packet.isNotEmpty) {
            print('BleService: Got outbound packet from Rust (${packet.length} bytes), broadcasting to ${_connectedPeers.length} peers');
            await _sendPacketToAllPeers(packet);
          }
        }
      } catch (e) {
        // Log but don't fail - might be transient
        print('BleService: Error polling outbound packets: $e');
      }
    });
  }

  /// Sends a raw packet (serialized Envelope) to all connected peers.
  Future<void> _sendPacketToAllPeers(List<int> packet) async {
    if (_connectedPeers.isEmpty) {
      return;
    }

    final bytes = Uint8List.fromList(packet);

    for (final entry in _connectedPeers.entries) {
      try {
        final services = await entry.value.discoverServices();
        for (final svc in services) {
          if (svc.uuid == _serviceGuid) {
            for (final chr in svc.characteristics) {
              if (chr.uuid == _charGuid && chr.properties.write) {
                await chr.write(bytes, withoutResponse: chr.properties.writeWithoutResponse);
                print('BleService: Sent Rust packet to peer ${entry.key}');
              }
            }
          }
        }
      } catch (e) {
        print('BleService: Failed to send Rust packet to peer ${entry.key}: $e');
      }
    }
  }

  /// Starts BLE scanning for ArgusMesh peers.
  void startScan() {
    // Only scan on Android - other platforms use btleplug in Rust
    if (!Platform.isAndroid) {
      print("BleService: Skipping Flutter BLE scan on non-Android platform (using btleplug)");
      return;
    }
    // Cancel any existing scan subscription first
    _scanSub?.cancel();

    print('BleService: Starting BLE scan for ArgusMesh peers...');

    // Listen for scan results
    _scanSub = FlutterBluePlus.scanResults.listen((results) {
      for (final r in results) {
        final id = r.device.remoteId.str;

        // Skip already-connected or currently-connecting peers
        if (_connectedPeers.containsKey(id) || _connectingPeers.contains(id)) {
          continue;
        }

        // Check if the device advertises our service
        final advertisesOurService = r.advertisementData.serviceUuids
            .any((u) => u == _serviceGuid);

        if (advertisesOurService) {
          print('BleService: Discovered ArgusMesh peer: $id '
              '(${r.device.platformName})');
          // Mark as connecting immediately to prevent duplicates
          _connectingPeers.add(id);
          // Stop scan, connect, then resume scan
          _stopScanAndConnect(r.device);
        }
      }
    });

    // Start scanning with a longer timeout
    FlutterBluePlus.startScan(
      withServices: [_serviceGuid],
      timeout: const Duration(seconds: 60),
    ).then((_) {
      print('BleService: Scan round finished. Waiting before restart...');
      Future.delayed(const Duration(seconds: 15), () {
        startScan();
      });
    });
  }

  /// Stops scanning, attempts connection, then restarts scanning.
  Future<void> _stopScanAndConnect(BluetoothDevice device) async {
    // Stop the active scan to free the BLE radio for connection
    print('BleService: Stopping scan to connect...');
    _scanSub?.cancel();
    await FlutterBluePlus.stopScan();

    // Short delay to let the radio settle
    await Future.delayed(const Duration(milliseconds: 500));

    await _connectToPeer(device);

    // Resume scanning after connection attempt (success or fail)
    await Future.delayed(const Duration(seconds: 2));
    startScan();
  }

  /// Connects to a peer device.
  Future<void> _connectToPeer(BluetoothDevice device) async {
    final id = device.remoteId.str;

    // Double-check guard
    if (_connectedPeers.containsKey(id)) {
      _connectingPeers.remove(id);
      return;
    }

    print('BleService: Connecting to peer $id...');
    try {
      await device.connect(timeout: const Duration(seconds: 15));
      print('BleService: Connected to peer $id!');

      // Discover services
      final services = await device.discoverServices();
      BluetoothCharacteristic? inboxChar;

      for (final svc in services) {
        if (svc.uuid == _serviceGuid) {
          for (final chr in svc.characteristics) {
            if (chr.uuid == _charGuid) {
              inboxChar = chr;
              break;
            }
          }
        }
        if (inboxChar != null) break;
      }

      if (inboxChar == null) {
        print('BleService: Peer $id has no ArgusMesh characteristic. Disconnecting.');
        await device.disconnect();
        _connectingPeers.remove(id);
        return;
      }

      print('BleService: Found INBOX characteristic on peer $id');

      // Subscribe to notifications (incoming data FROM the peer)
      if (inboxChar.properties.notify) {
        await inboxChar.setNotifyValue(true);
        inboxChar.onValueReceived.listen((value) {
          print('BleService: Received ${value.length} bytes from peer $id');
          api.receiveBlePacket(data: Uint8List.fromList(value));
        });
      }

      // Move from "connecting" to "connected"
      _connectingPeers.remove(id);
      _connectedPeers[id] = device;
      _peerLog.add('Connected: $id');
      onPeerLog?.call('Connected: $id');
      onPeerConnected?.call(id);
      print('BleService: Peer $id ready for mesh communication');

      // Listen for disconnection (skip initial state by filtering)
      device.connectionState.listen((state) {
        if (state == BluetoothConnectionState.disconnected &&
            _connectedPeers.containsKey(id)) {
          print('BleService: Peer $id disconnected');
          _connectedPeers.remove(id);
          _peerLog.add('Disconnected: $id');
          onPeerLog?.call('Disconnected: $id');
          onPeerDisconnected?.call(id);
        }
      });
    } catch (e) {
      print('BleService: Failed to connect to peer $id: $e');
      _connectingPeers.remove(id);
    }
  }

  /// Broadcasts an alert to all connected peers.
  Future<void> broadcastAlert(api.AlertInput input) async {
    if (_connectedPeers.isEmpty) {
      print('BleService: No connected peers to broadcast to.');
      return;
    }

    // Serialize as JSON for simplicity (matches Rust's deserialization)
    final bytes = utf8.encode(jsonEncode({
      'sender': input.sender,
      'priority': input.priority.toString(),
      'payload': input.payload,
    }));

    for (final entry in _connectedPeers.entries) {
      try {
        final services = await entry.value.discoverServices();
        for (final svc in services) {
          if (svc.uuid == _serviceGuid) {
            for (final chr in svc.characteristics) {
              if (chr.uuid == _charGuid && chr.properties.write) {
                await chr.write(bytes, withoutResponse: chr.properties.writeWithoutResponse);
                print('BleService: Sent alert to peer ${entry.key}');
              }
            }
          }
        }
      } catch (e) {
        print('BleService: Failed to send to peer ${entry.key}: $e');
      }
    }
  }

  /// Disposes resources and disconnects all peers.
  void dispose() {
    _scanSub?.cancel();
    _outboundPollTimer?.cancel();
    FlutterBluePlus.stopScan();
    for (final device in _connectedPeers.values) {
      device.disconnect();
    }
    _connectedPeers.clear();
    _connectingPeers.clear();
  }
}

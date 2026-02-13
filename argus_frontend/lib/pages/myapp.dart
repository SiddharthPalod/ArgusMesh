import 'dart:async';
import 'dart:convert';
import 'dart:io' show Platform;

import 'package:flutter/material.dart';
import 'package:argus_frontend/bridge_generated.dart/rust_api/definitions/core/api.dart' as api;
import 'package:argus_frontend/bridge_generated.dart/rust_api/definitions/routing/envelope.dart' show Priority;

import '../components/role_selector.dart';
import '../components/gps_indicator.dart';
import '../components/node_state_card.dart';
import '../components/connected_peers_card.dart';
import '../components/alert_creation_buttons.dart';
import '../components/alerts_list.dart';
import '../components/error_display.dart';
import '../components/loading_display.dart';
import '../components/role_info_card.dart';
import '../services/storage_service.dart';
import '../services/permission_service.dart';
import '../services/gps_service.dart';
import '../services/mesh_service.dart';
import '../services/ble_service.dart';
import '../models/node_state_model.dart';
import 'map_view.dart';

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Argus Mesh',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepOrange),
        useMaterial3: true,
      ),
      home: const ArgusHomePage(),
    );
  }
}

class ArgusHomePage extends StatefulWidget {
  const ArgusHomePage({super.key});

  @override
  State<ArgusHomePage> createState() => _ArgusHomePageState();
}

class _ArgusHomePageState extends State<ArgusHomePage> {
  String? _loadError;
  NodeStateModel? _state;
  List<dynamic> _alerts = [];
  bool _loading = true;

  // Role selection
  String _role = 'field';

  // Services
  final GpsService _gpsService = GpsService();
  final MeshService _meshService = MeshService();
  final BleService _bleService = BleService();

  // Bottom navigation
  int _navIndex = 0;

  @override
  void initState() {
    super.initState();
    _init();
  }

  Future<void> _init() async {
    try {
      // Initialize Rust library
      await _meshService.initRustLib();

      // Configure storage path (ensure DB path is set before init)
      await StorageService.configureStoragePath();

      // Request permissions (Android-specific)
      if (Platform.isAndroid) {
        final permissionError = await PermissionService.ensurePermissions();
        if (permissionError != null) {
          setState(() {
            _loadError = permissionError;
            _loading = false;
          });
          return;
        }

        // Initialize Flutter BLE (Android only - other platforms use btleplug)
        await _bleService.initialize();
      }

      // Start mesh node (works on all platforms - uses Flutter BLE on Android, btleplug elsewhere)
      await _meshService.startMeshNode(_role);

      // Start BLE scanning (Android uses Flutter, other platforms use btleplug in Rust)
      if (Platform.isAndroid) {
        _bleService.startScan();
      }

      // Initialize GPS
      await _gpsService.initialize();

      // Refresh data
      await _refresh();
    } catch (e, st) {
      if (mounted) {
        setState(() {
          _loadError = 'API error: $e\n$st';
          _loading = false;
        });
      }
    }
  }

  Future<void> _refresh() async {
    try {
      final stateMap = await _meshService.getNodeState();
      final alerts = await _meshService.getKnownAlerts();

      setState(() {
        _state = NodeStateModel.fromMap(stateMap);
        _alerts = alerts;
        _loading = false;
      });
    } catch (e) {
      setState(() {
        _loadError = 'API error: $e';
        _loading = false;
      });
    }
  }

  void _onRoleChanged(String role) {
    setState(() => _role = role);
    _meshService.startMeshNode(role);
  }

  Future<void> _createAlert(Priority priority) async {
    try {
      // Capture GPS for the alert
      final pos = await _gpsService.capturePosition();

      final payloadMap = <String, dynamic>{
        'time': DateTime.now().toIso8601String(),
        'role': _role,
      };
      if (pos != null) {
        payloadMap['lat'] = pos.latitude;
        payloadMap['lng'] = pos.longitude;
      }

      final input = api.AlertInput(
        sender: _state?.status?.toString() ?? 'field',
        priority: priority,
        payload: jsonEncode(payloadMap),
      );
      final msgId = await _meshService.createAlert(input);
      print('Alert created: $msgId');

      await _refresh();
    } catch (e) {
      setState(() => _loadError = 'Create failed: $e');
    }
  }

  @override
  void dispose() {
    _bleService.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (_loadError != null) {
      return ErrorDisplay(errorMessage: _loadError!);
    }

    if (_loading) {
      return const LoadingDisplay();
    }

    return Scaffold(
      appBar: AppBar(
        title: const Text('Argus Mesh'),
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: () => _refresh(),
            tooltip: 'Refresh',
          ),
        ],
      ),
      body: _navIndex == 0 ? _buildAlertsTab(context) : MapView(alerts: _alerts),
      bottomNavigationBar: NavigationBar(
        selectedIndex: _navIndex,
        onDestinationSelected: (i) => setState(() => _navIndex = i),
        destinations: const [
          NavigationDestination(
            icon: Icon(Icons.notifications_active_outlined),
            selectedIcon: Icon(Icons.notifications_active),
            label: 'Alerts',
          ),
          NavigationDestination(
            icon: Icon(Icons.map_outlined),
            selectedIcon: Icon(Icons.map),
            label: 'Map',
          ),
        ],
      ),
    );
  }

  Widget _buildAlertsTab(BuildContext context) {
    return RefreshIndicator(
      onRefresh: () async => _refresh(),
      child: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          // Role selector
          RoleSelector(
            currentRole: _role,
            onChanged: _onRoleChanged,
          ),
          const SizedBox(height: 12),

          // GPS indicator
          GpsIndicator(
            gpsAvailable: _gpsService.gpsAvailable,
            lastPosition: _gpsService.lastPosition,
          ),
          const SizedBox(height: 12),

          // Node state
          NodeStateCard(state: _state),
          const SizedBox(height: 16),

          // Connected peers
          ConnectedPeersCard(
            connectedPeers: _bleService.connectedPeers,
          ),
          const SizedBox(height: 16),

          // Create alert buttons — hidden for relay role
          if (_role != 'relay') ...[
            Text(
              'Create alert',
              style: Theme.of(context).textTheme.titleSmall,
            ),
            const SizedBox(height: 8),
            AlertCreationButtons(onCreateAlert: _createAlert),
            const SizedBox(height: 24),
          ],

          // Role mode info
          RoleInfoCard(role: _role),
          const SizedBox(height: 8),

          Text(
            'Known alerts (${_alerts.length})',
            style: Theme.of(context).textTheme.titleSmall,
          ),
          const SizedBox(height: 8),
          AlertsList(alerts: _alerts),
        ],
      ),
    );
  }
}

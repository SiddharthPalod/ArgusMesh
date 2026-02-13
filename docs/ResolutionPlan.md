# Resolution Plan for ArgusMesh Alert Propagation Issues

## Executive Summary

This document outlines a comprehensive plan to resolve the alert propagation issues observed during testing. The core problems stem from the mesh routing layer not being properly initialized and integrated with the BLE transport layer.

**Key Principle**: The app must be **WiFi-independent** and rely solely on BLE for peer-to-peer communication.

---

## Issues Identified

### 1. **Missing Mesh Runtime Infrastructure**
- `start_mesh_node()` doesn't create a Router or Node instance
- No processing loop (`Node::run()`) is running to handle incoming/outgoing packets
- No packet injector is initialized for Flutter → Rust communication

### 2. **Alert Transmission Not Integrated**
- Alerts are created and stored but never enqueued in the router's priority queue
- Flutter manually broadcasts alerts via BLE, bypassing Rust routing layer
- This means deduplication, encryption, signing, and TTL handling aren't applied

### 3. **Persistence Issues**
- Alerts reset to 0 on app restart
- In-memory cache may not be loading from DB properly
- Runtime tag persistence works but alerts aren't being restored

### 4. **BLE Transport Integration Gap**
- Packet injector channel not initialized
- No connection between Flutter BLE layer and Rust router
- Router expects packets via `Transport::recv()` but nothing is feeding it

### 5. **Symmetric Key Management**
- Each node generates its own symmetric key
- Nodes can't decrypt each other's messages
- Need shared key or key exchange mechanism

---

## Resolution Strategy

### Phase 1: Initialize Mesh Runtime Infrastructure

#### 1.1 Create Global Router Instance
**File**: `backend/src/core/api.rs`

- Create a global Router instance with:
  - BLE transport adapter
  - In-memory store (or persistent store)
  - Shared symmetric key (hardcoded for now, or loaded from config)
  - Node keys (generated per device)

- Store router in a `Lazy<Mutex<Option<Arc<Mutex<Router<...>>>>>>` for FFI access

#### 1.2 Initialize Packet Injector
**File**: `backend/src/core/api.rs` in `start_mesh_node()`

- Call `create_ble_transport()` to set up:
  - BLE scanner
  - BLE adapter
  - Packet injector channel
- Store adapter reference for use by router

#### 1.3 Start Node Processing Loop
**File**: `backend/src/core/api.rs`

- Create a `Node` instance wrapping the router
- Spawn `Node::run()` as a background task using `tokio::spawn`
- This loop will:
  - Process incoming packets from BLE via `transport.recv()`
  - Send outgoing packets via `transport.send()`
  - Handle router's priority queue

**Implementation Notes**:
```rust
// In start_mesh_node():
let (scanner, adapter) = create_ble_transport();
let store = MemStore::new(); // or persistent store
let keys = NodeKeys::generate();
let sym_key = get_shared_symmetric_key(); // See Phase 2
let router = Arc::new(Mutex::new(Router::new(adapter, store, keys, sym_key)));
let node = Node::new(router.clone(), Arc::new(adapter));
tokio::spawn(async move { node.run().await; });
```

---

### Phase 2: Fix Symmetric Key Sharing

#### 2.1 Shared Symmetric Key
**File**: `backend/src/core/api.rs` or new `backend/src/core/keys.rs`

**Option A**: Hardcoded shared key (simple, for testing)
```rust
fn get_shared_symmetric_key() -> SymKey {
    // Use a fixed key for all nodes (development only)
    [0x42; 32] // Replace with proper key management
}
```

**Option B**: Load from config/environment
- Store key in app's secure storage
- Generate on first run, persist, reuse
- All devices use same key

**Option C**: Key exchange protocol (future enhancement)
- Implement BLE-based key exchange
- More secure but complex

**Recommendation**: Start with Option A for testing, move to Option B for production.

---

### Phase 3: Integrate Alert Creation with Router

#### 3.1 Modify `create_alert()` to Enqueue
**File**: `backend/src/core/api.rs`

Current behavior:
- Creates envelope
- Stores in alert index
- Returns message ID

New behavior:
- Creates envelope
- Stores in alert index
- **Enqueues in router's priority queue** using `router.enqueue_local()`
- Router will handle encryption, signing, and transmission

**Implementation**:
```rust
pub fn create_alert(input: AlertInput) -> Result<String, String> {
    let env = Envelope::new(input.sender, input.priority, input.payload.into_bytes());
    let msg_id = env.msg_id.to_string();
    
    // Store alert
    add_alert(env.clone());
    
    // Enqueue for transmission
    if let Some(router) = get_global_router() {
        if let Ok(mut r) = router.lock() {
            let _ = r.enqueue_local(env);
        }
    }
    
    Ok(msg_id)
}
```

#### 3.2 Remove Flutter Manual Broadcasting
**File**: `argus_frontend/lib/pages/myapp.dart`

- Remove `_broadcastToPeers()` call from `_createAlert()`
- Rust router will handle all transmission automatically
- Flutter only needs to call `api.createAlert()`

---

### Phase 4: Fix Persistence and Restore

#### 4.1 Load Alerts on Startup
**File**: `backend/src/core/api.rs` in `init_node()`

- After opening DB, load all persisted alerts
- Populate in-memory cache
- Ensure `list_alerts()` returns persisted alerts

**Implementation**:
```rust
pub fn init_node() -> Result<(), String> {
    // ... existing tag restoration ...
    
    // Load persisted alerts into memory
    let alerts = list_alerts(); // This will load from DB if memory is empty
    log::info!("Loaded {} persisted alerts", alerts.len());
    
    Ok(())
}
```

#### 4.2 Ensure DB Path is Set Early
**File**: `argus_frontend/lib/pages/myapp.dart`

- Call `configure_storage_base_dir()` **before** `init_node()`
- Ensure DB is opened before any alert operations

**Current order**:
1. `configure_storage_base_dir()` (async, not awaited)
2. `init_node()`
3. `start_mesh_node()`

**Fix**: Await storage configuration before init.

---

### Phase 5: Connect BLE Layer to Router

#### 5.1 Ensure Packet Injector is Set
**File**: `backend/src/core/api.rs`

- Verify `create_ble_transport()` is called before any BLE packets arrive
- The injector channel must be initialized before Flutter starts receiving BLE data

#### 5.2 Router Receives from Injector
**File**: `backend/src/core/node.rs` or `backend/src/core/api.rs`

- The `Node::run()` loop calls `transport.recv()`
- `BleAdapter::recv()` reads from the notification channel
- This channel is fed by `inject_packet()` which Flutter calls

**Flow**:
```
Flutter BLE → receiveBlePacket() → inject_packet() → 
BLE adapter notification channel → transport.recv() → 
Router::receive() → add_alert() + enqueue for forwarding
```

---

### Phase 6: Fix Node State Persistence

#### 6.1 Persist Runtime Tag Properly
**File**: `backend/src/core/api.rs`

- `init_node()` already restores tag from DB
- Ensure `start_mesh_node()` persists the tag
- Verify tag is restored on restart

#### 6.2 Fix Node State Reporting
**File**: `backend/src/core/api.rs` in `get_node_state()`

- Current: Returns runtime tag as status
- Issue: Tag might be "argus-field" instead of "argus-node"
- Fix: Ensure tag is set correctly based on role

**Note**: The "argus_node" vs "argus_field" distinction seems to be role-based. Ensure role is persisted and restored.

---

## Implementation Checklist

### Backend Changes

- [ ] **Create global router instance**
  - [ ] Add `Lazy<Mutex<Option<Arc<Mutex<Router<...>>>>>>` in `api.rs`
  - [ ] Initialize in `start_mesh_node()`
  - [ ] Store BLE adapter reference

- [ ] **Initialize BLE transport**
  - [ ] Call `create_ble_transport()` in `start_mesh_node()`
  - [ ] Verify packet injector is set
  - [ ] Store adapter for router

- [ ] **Start Node processing loop**
  - [ ] Create `Node` instance
  - [ ] Spawn `Node::run()` as background task
  - [ ] Ensure it runs continuously

- [ ] **Implement shared symmetric key**
  - [ ] Create `get_shared_symmetric_key()` function
  - [ ] Use same key for all nodes (hardcoded for now)
  - [ ] Pass to router constructor

- [ ] **Integrate alert creation with router**
  - [ ] Modify `create_alert()` to call `router.enqueue_local()`
  - [ ] Ensure alerts are encrypted and signed before transmission
  - [ ] Remove dependency on Flutter for broadcasting

- [ ] **Fix persistence**
  - [ ] Ensure DB path is set before `init_node()`
  - [ ] Load alerts from DB on startup
  - [ ] Populate in-memory cache
  - [ ] Verify alerts persist across restarts

- [ ] **Add logging**
  - [ ] Log when router receives packets
  - [ ] Log when alerts are enqueued
  - [ ] Log when alerts are transmitted
  - [ ] Log decryption/signature verification failures

### Frontend Changes

- [ ] **Remove manual BLE broadcasting**
  - [ ] Remove `_broadcastToPeers()` call from `_createAlert()`
  - [ ] Remove `_broadcastToPeers()` function (or keep for debugging)
  - [ ] Trust Rust router to handle transmission

- [ ] **Fix initialization order**
  - [ ] Await `configure_storage_base_dir()` before `init_node()`
  - [ ] Ensure proper error handling

- [ ] **Add debug logging**
  - [ ] Log when alerts are created
  - [ ] Log when BLE packets are received
  - [ ] Log router state if accessible

---

## Testing Plan

### Test Case 1: Alert Creation and Storage
1. Create alert on Device A
2. Verify alert appears in `get_known_alerts()` on Device A
3. Restart app on Device A
4. Verify alert still appears (persistence test)

### Test Case 2: Alert Propagation (BLE)
1. Start app on Device A and Device B
2. Ensure both devices show "Connected Peers: 1"
3. Create alert on Device A
4. Wait 5-10 seconds
5. Verify alert appears on Device B via `get_known_alerts()`
6. Verify alert appears on Device A (should still be there)

### Test Case 3: Bidirectional Propagation
1. Create alert on Device A → verify on Device B
2. Create alert on Device B → verify on Device A
3. Verify both alerts appear on both devices

### Test Case 4: Persistence Across Restart
1. Create alerts on both devices
2. Restart both apps
3. Verify all alerts are still present
4. Verify node state (tag) is restored correctly

### Test Case 5: Multiple Hops (if 3+ devices available)
1. Create alert on Device A
2. Verify it propagates to Device B
3. Verify it propagates to Device C (via Device B)
4. Verify deduplication prevents duplicates

---

## Risk Mitigation

### Risk 1: Router Not Processing Packets
**Mitigation**: Add extensive logging to verify router is receiving and processing packets.

### Risk 2: Symmetric Key Mismatch
**Mitigation**: Use hardcoded key initially, verify all nodes use same key, add key validation logging.

### Risk 3: BLE Packet Format Mismatch
**Mitigation**: Ensure Flutter sends raw serialized Envelope bytes, not JSON. Verify serialization format matches.

### Risk 4: Performance Issues
**Mitigation**: Monitor CPU usage, ensure background tasks don't block UI, use appropriate queue sizes.

---

## Future Enhancements

1. **Proper Key Management**: Implement secure key exchange protocol
2. **WiFi Direct Support**: Add WiFi Direct transport adapter (as per ProjectPlan.md)
3. **Metrics and Monitoring**: Add alert propagation metrics, latency tracking
4. **Error Recovery**: Handle BLE disconnections gracefully, retry failed transmissions
5. **Battery Optimization**: Reduce BLE scanning frequency when no peers detected

---

## Notes

- **WiFi Independence**: All communication must use BLE only. No WiFi dependencies.
- **Store-Carry-Forward**: Alerts persist locally and forward opportunistically when peers are available.
- **Security**: All alerts are encrypted and signed. Invalid signatures are rejected.
- **Deduplication**: UUID-based deduplication prevents alert storms.

---

## Timeline Estimate

- **Phase 1-2**: 2-3 hours (Router initialization + key management)
- **Phase 3**: 1 hour (Alert integration)
- **Phase 4**: 1-2 hours (Persistence fixes)
- **Phase 5**: 1 hour (BLE integration verification)
- **Phase 6**: 30 minutes (State persistence)
- **Testing**: 2-3 hours

**Total**: ~8-10 hours of development time

---

## Success Criteria

✅ Alerts propagate between devices via BLE  
✅ Alerts persist across app restarts  
✅ Node state (tag) persists correctly  
✅ No manual Flutter broadcasting required  
✅ All alerts are encrypted and signed  
✅ Deduplication prevents duplicate alerts  
✅ System works without WiFi/internet  

---

*Last Updated: 2026-02-13*

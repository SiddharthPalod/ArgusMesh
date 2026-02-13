Perfect — now we switch from **architecture** to **execution**.
Below is an **exhaustive, phase-by-phase implementation checklist** you can literally track in a project board (Notion / GitHub Projects).

This is **build-order correct**, dependency-aware, and assumes **Rust core first, Flutter later**.

---

# 🛡️ Border Alert Mesh

## Exhaustive Phase-Based Implementation Plan

**Rust Core → Transport → Flutter → Security → Hardening**

---

# 🔵 PHASE 0 — Project Foundations (Do NOT skip)

### 0.1 Repo & Workspace Setup

* [ ] Create mono-repo

  ```
  /border-alert-mesh
    /rust-core
    /flutter-app
    /docs
  ```
* [ ] Initialize Rust workspace
* [ ] Initialize Flutter app
* [ ] Setup CI (basic build checks)

---

### 0.2 Documentation First

* [ ] Write **Problem Statement**
* [ ] Write **Threat Model (high-level)**
* [ ] Define **Non-goals**
* [ ] Architecture diagram (static)

> This prevents scope creep later.

---

# 🔵 PHASE 1 — Rust Core: Data & State (FOUNDATION)

### 1.1 Core Types

* [ ] `NodeId` generation (ephemeral)
* [ ] `AlertType` enum
* [ ] `Severity` enum
* [ ] `Alert` struct
* [ ] Serialization (bincode / protobuf)

---

### 1.2 Alert Lifecycle

* [ ] Create alert
* [ ] Validate fields
* [ ] Timestamp normalization
* [ ] TTL initialization

---

### 1.3 Local Persistence

* [ ] Storage engine selection (sled- Rust based DB (SQL+Embedded Oriented))
* [ ] Persist alerts
* [ ] Persist forwarding metadata
* [ ] Crash recovery tests

---

### 1.4 Unit Tests

* [ ] Alert creation tests
* [ ] Serialization/deserialization tests
* [ ] TTL boundary tests

---

# 🔵 PHASE 2 — Routing & Propagation Engine (CORE LOGIC)

### 2.1 Message Envelope

* [ ] Wrap alerts in transport-neutral envelope
* [ ] Include metadata:

  * hop count
  * sender id
  * priority

---

### 2.2 Priority Queues

* [ ] Implement multi-level queues
* [ ] Scheduling strategy
* [ ] Starvation prevention

---

### 2.3 Deduplication

* [ ] UUID-based dedup cache
* [ ] Eviction strategy
* [ ] Memory bounds

---

### 2.4 Store–Carry–Forward

* [ ] Persist unsent alerts
* [ ] Retry logic
* [ ] Reconnection handling

---

### 2.5 Conflict Resolution

* [ ] Same alert ID → latest wins
* [ ] Expired alerts dropped
* [ ] Invalid updates rejected

---

### 2.6 Stress Testing

* [ ] Simulate 100+ alerts
* [ ] Node churn simulation
* [ ] TTL exhaustion tests

---

# 🔵 PHASE 3 — Transport Layer (BLE FIRST)

### 3.1 Transport Abstraction

* [x] Define `Transport` trait
* [x] Send / receive interface
* [x] Error model

---

### 3.2 BLE Discovery

* [ ] Advertise node presence
* [x] Scan for peers
* [x] Connection management

---

### 3.3 BLE Messaging

* [x] Fragmentation logic
* [x] Reassembly
* [ ] Acknowledgement (optional)

---

### 3.4 Integration with Routing Engine

* [x] Async Router Architecture (Tokio)
* [x] Implement Main Event Loop
* [x] Pull from priority queues
* [x] Push received messages
* [ ] Enforce TTL & dedup

---

### 3.5 Transport Testing

* [x] Latency measurement
* [x] Packet loss simulation
* [ ] Battery impact logging

---

# 🔵 PHASE 4 — Security Layer (FIRST-CLASS)

### 4.1 Key Management

* [x] Generate node keypair
* [x] Key rotation strategy
* [x] Secure storage

---

### 4.2 Payload Encryption

* [x] Encrypt alert payloads
* [x] Decrypt on receive
* [x] Reject invalid ciphertext

---

### 4.3 Message Signing

* [x] Sign outgoing alerts
* [x] Verify signatures
* [x] Drop unauthenticated messages

---

### 4.4 Security Tests

* [x] Tampered packet rejection
* [x] Replay attack simulation
* [x] Invalid sender tests

---

# 🔵 PHASE 5 — Rust Core API & FFI

### 5.1 Core Public API

* [ ] `init_node()`
* [ ] `create_alert()`
* [ ] `get_known_alerts()`
* [ ] `get_node_state()`

---

### 5.2 FFI Boundary

* [ ] Define C-compatible API
* [ ] Memory ownership rules
* [ ] Error propagation model

---

### 5.3 Dart Bindings

* [ ] Dart FFI bindings
* [ ] Type mapping
* [ ] Safety wrappers

---

### 5.4 FFI Testing

* [ ] Stress calls from Dart
* [ ] Memory leak checks
* [ ] Crash recovery

---

# 🔵 PHASE 6 — Flutter Application (CONTROLLED SCOPE)

### 6.1 App Skeleton

* [ ] Role selection (field / command)
* [ ] State management
* [ ] FFI initialization

---

### 6.2 Alert Creation UI

* [ ] One-tap alert buttons
* [ ] GPS capture
* [ ] Manual override

---

### 6.3 Situational Awareness

* [ ] Alert list
* [ ] Map view
* [ ] Confidence indicators

---

### 6.4 Node Health UI

* [ ] Last seen time
* [ ] Known alert count
* [ ] Transport status

---

# 🔵 PHASE 7 — Internet Sync (OPTIONAL, REALISTIC)

### 7.1 Sync Protocol

* [ ] Encrypted upload format
* [ ] Summary-based sync
* [ ] Conflict handling

---

### 7.2 Backend (Minimal)

* [ ] Firebase setup
* [ ] Ingest endpoint
* [ ] Read-only dashboard

---

### 7.3 Sync Failure Handling

* [ ] Partial uploads
* [ ] Retry logic
* [ ] Offline fallback

---

# 🔵 PHASE 8 — Failure, Chaos & Field Testing

### 8.1 Failure Modes

* [ ] Long offline periods
* [ ] Sudden node loss
* [ ] Clock drift

---

### 8.2 Load Testing

* [ ] Alert storms
* [ ] Battery drain
* [ ] Memory pressure

---

### 8.3 Real Movement Testing

* [ ] Physical device movement
* [ ] Intermittent connections
* [ ] Recovery time measurement

---

# 🔵 PHASE 9 — Hardening & Polish

### 9.1 Performance Optimization

* [ ] Reduce wakeups
* [ ] BLE scan tuning
* [ ] Queue optimization

---

### 9.2 Documentation

* [ ] Architecture deep dive
* [ ] Security model
* [ ] Failure analysis

---

### 9.3 Final Demo Scenarios

* [ ] 3-node propagation
* [ ] 10-node stress demo
* [ ] Blackout recovery demo

---

# 🏁 FINAL DELIVERABLES

* ✅ Working offline mesh
* ✅ Secure alert propagation
* ✅ Cross-platform UI
* ✅ Research-grade documentation
* ✅ Defensible system design

---


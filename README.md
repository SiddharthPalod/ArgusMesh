# Argus Mesh
### Secure Offline Alert Propagation System
**Rust Mesh Core + Flutter UI (FFI)**

> **Ensure critical border alerts propagate reliably even when communication infrastructure is unavailable, jammed, or compromised.**

> "In modern border defense, the greatest threat isn’t just the intruder—it’s the Dead Zone. When electronic warfare jams the signals and satellite links go down, our frontline units go blind.
Enter Border Alert Mesh. We’ve built an 'unkillable' communication layer that lives entirely on edge devices. By combining the system-level safety of Rust with the rapid visualization of Flutter, we created a peer-to-peer mesh that propagates critical threats like a virus—phone to phone, unit to unit—without needing a single cell tower.
With Border Alert Mesh, when one soldier sees a threat, the entire line knows instantly. We don’t just survive the blackout; we own it."

This is a **fallback communication layer** for defence and civil-defence scenarios.

---

## 2️⃣ Non-Negotiable Design Principles

These define correctness:

1. **Offline-first** — internet is optional
2. **Eventual delivery > real-time delivery**
3. **Security by default** — encrypted, signed payloads
4. **Deterministic behavior** — predictable under failure
5. **Separation of concerns** — UI ≠ networking ≠ crypto

---

## 3️⃣ High-Level Architecture (Final)

```
┌────────────────────────────────┐
│        Flutter UI Layer         │
│  - Alert creation               │
│  - Map & situational view       │
│  - Role-based screens           │
└───────────────┬────────────────┘
                │ Dart FFI
┌───────────────▼────────────────┐
│        Rust Mesh Core           │
│  - Routing engine               │
│  - Deduplication                │
│  - Priority queues              │
│  - Crypto (sign + encrypt)      │
│  - Persistence                  │
└───────────────┬────────────────┘
                │
┌───────────────▼────────────────┐
│   Transport Adapters (Rust)     │
│  - BLE                          │
│  - Wi-Fi Direct (stretch)       │
└────────────────────────────────┘
```

Flutter **never** touches:

* routing logic
* cryptography
* peer state

That’s what makes this defensible.

---

## 4️⃣ System Roles (Logical, Not Hardcoded)

All devices run the same binary.

| Role         | Difference              |
| ------------ | ----------------------- |
| Field Node   | Can create alerts       |
| Relay Node   | Auto-forward only       |
| Command Node | Aggregates + visualizes |

Roles are **UI permissions**, not backend forks.

---

## 5️⃣ Rust Mesh Core (THE HEART)

### 📦 Crate Structure

```
border_mesh_core/
├── lib.rs
├── routing/
│   ├── queue.rs
│   ├── ttl.rs
│   ├── dedup.rs
├── crypto/
│   ├── keys.rs
│   ├── encrypt.rs
│   ├── sign.rs
├── transport/
│   ├── ble.rs
│   ├── wifi.rs
├── storage/
│   ├── local_store.rs
├── ffi/
│   ├── api.rs
```

Each module is testable **without Flutter**.

---

## 6️⃣ Data Model (Final)

### 🚨 Alert

```text
alert_id: UUID
type: enum { intrusion, drone, movement }
latitude: f64
longitude: f64
severity: u8 (1–5)
created_at: u64
last_updated_at: u64
origin_node_id: NodeId
ttl: u8
signature: bytes
```

### 🧠 Node State

```text
node_id
last_seen_timestamp
known_alert_count
```

---

## 7️⃣ Mesh Routing Logic (Core Algorithm)

### 🔁 Store–Carry–Forward

* Alerts persist locally
* Forward opportunistically
* Survive long disconnections

### 📌 Priority Queues

```
CRITICAL (alerts)
MEDIUM   (status)
LOW      (logs / sync)
```

Critical alerts **always transmit first**.

---

### 🔄 Deduplication

* UUID-based
* Bloom filter or LRU cache
* Prevents alert storms

---

### ⏱️ TTL & Hop Control

* Decrement TTL on every hop
* Drop expired alerts
* Prevent infinite loops

---

## 8️⃣ Security Model (Practical, Not Overkill)

### 🔐 Identity

* Ephemeral node IDs
* Rotated on restart / interval
* No personal identity

### 🔏 Encryption

* Payload encryption (libsodium / ring)
* Transport-agnostic

### ✍️ Integrity

* All alerts signed
* Invalid signatures dropped immediately


---

## 1️⃣3️⃣ What Makes This “Actually Usable”

✅ Works with zero internet
✅ Handles node loss gracefully
✅ Doesn’t flood the network
✅ Doesn’t trust the UI
✅ Recovers after hours offline
---

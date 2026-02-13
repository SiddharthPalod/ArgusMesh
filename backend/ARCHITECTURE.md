# Backend Architecture - Design Patterns & Modularity

This document describes the modular architecture of the Argus Mesh backend, highlighting the design patterns used to ensure consistency and maintainability.

## Design Patterns Implemented

### 1. **Service Layer Pattern** (`src/services/`)

The Service Layer pattern provides high-level abstractions for business operations, coordinating between multiple repositories and domain objects.

**Services:**
- `AlertService` - Manages alert creation, listing, and retrieval
- `MeshService` - Handles mesh node lifecycle (initialization, starting, status)
- `StorageService` - Manages storage configuration and paths

**Benefits:**
- Clean separation between API layer and domain logic
- Easy to test and mock
- Single responsibility per service

### 2. **Repository Pattern** (`src/repository/`)

The Repository pattern abstracts data access operations, making it easy to swap storage implementations.

**Repositories:**
- `AlertRepository` - Provides CRUD operations for alerts
- Generic `Repository<T, ID>` trait for consistent data access

**Benefits:**
- Easy to swap storage implementations (in-memory, database, etc.)
- Testable with mock repositories
- Consistent data access patterns

### 3. **Factory Pattern** (`src/factory/`)

Factories encapsulate object creation logic, centralizing complex initialization.

**Factories:**
- `TransportFactory` - Creates transport instances (BLE, TestSim, etc.)
- `StoreFactory` - Creates store instances (Memory, Sled, etc.)
- `RouterFactory` - Creates router instances with proper key initialization

**Benefits:**
- Centralized creation logic
- Easy to add new transport/store types
- Default configurations provided

### 4. **Builder Pattern** (`src/builder/`)

Builders provide a fluent interface for constructing complex objects with many optional parameters.

**Builders:**
- `NodeBuilder` - Constructs Node instances with various options
- `RouterBuilder` - Constructs Router instances with configuration options

**Benefits:**
- Improved readability
- Flexible object construction
- Future extensibility

### 5. **Unified Error Handling** (`src/error.rs`)

A centralized error handling system using `thiserror` for consistent error types across modules.

**Error Types:**
- `MeshError` - Main error enum with variants for different error categories
- `MeshResult<T>` - Result type alias for operations that can fail
- Automatic conversions from common error types (sled, bincode, serde_json, etc.)

**Benefits:**
- Consistent error handling
- Easy error propagation
- Clear error messages

## Module Structure

```
backend/src/
├── core/           # Core domain logic
│   ├── api.rs      # FFI API layer (uses services)
│   ├── alert.rs
│   ├── node.rs
│   └── ...
├── services/       # Service layer (business logic)
│   ├── alert_service.rs
│   ├── mesh_service.rs
│   └── storage_service.rs
├── repository/     # Repository pattern (data access)
│   ├── alert_repository.rs
│   └── traits.rs
├── factory/        # Factory pattern (object creation)
│   ├── transport_factory.rs
│   ├── store_factory.rs
│   └── router_factory.rs
├── builder/        # Builder pattern (complex construction)
│   ├── node_builder.rs
│   └── router_builder.rs
├── routing/        # Routing logic
├── transport/      # Transport layer
├── storage/        # Storage implementations
├── crypto/         # Cryptographic operations
└── error.rs        # Unified error handling
```

## Usage Examples

### Using Services

```rust
use crate::services::{AlertService, MeshService};

// Create an alert
let msg_id = AlertService::create_alert(
    "field-node-1".to_string(),
    Priority::Critical,
    payload_bytes,
)?;

// Start mesh node
MeshService::start_mesh_node("argus-node".to_string())?;
```

### Using Factories

```rust
use crate::factory::{TransportFactory, StoreFactory, RouterFactory};
use crate::factory::TransportType;

// Create transport
let transport = TransportFactory::create_default().await?;

// Create store
let store = StoreFactory::create_default()?;

// Create router
let router = RouterFactory::create(transport, store);
```

### Using Builders

```rust
use crate::builder::NodeBuilder;

let node = NodeBuilder::new()
    .with_router(router_arc)
    .with_transport(transport_arc)
    .build()?;
```

## Consistency Principles

1. **Single Responsibility** - Each module/struct has one clear purpose
2. **Dependency Injection** - Dependencies are injected, not created internally
3. **Error Handling** - All errors flow through `MeshError` type
4. **Trait-Based Design** - Interfaces defined via traits for flexibility
5. **Service Coordination** - Services coordinate between repositories and domain objects

## Future Enhancements

- **Strategy Pattern** - For different routing strategies
- **Observer Pattern** - For event notifications
- **Command Pattern** - For undo/redo operations
- **Dependency Injection Container** - For managing dependencies

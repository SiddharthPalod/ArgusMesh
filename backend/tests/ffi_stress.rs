//! Phase 5.4 — FFI stress & memory-safety integration tests.
//!
//! These tests exercise the public Rust API and C FFI boundary under load,
//! verifying correct JSON output, no panics, and no double-frees.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// ── Helpers to call extern "C" functions safely in tests ──────────────────

unsafe extern "C" {
    fn argus_init_node() -> *mut c_char;
    fn argus_create_alert(input_json: *const c_char) -> *mut c_char;
    fn argus_get_known_alerts() -> *mut c_char;
    fn argus_get_node_state() -> *mut c_char;
    fn argus_free_string(ptr: *mut c_char);

    fn argus_sim_start(n: i32) -> *mut c_char;
    fn argus_sim_create_alert(node_id: i32, input_json: *const c_char) -> *mut c_char;
    fn argus_sim_propagate() -> *mut c_char;
    fn argus_sim_get_alerts(node_id: i32) -> *mut c_char;
    fn argus_sim_stop();
}

/// Read a C string, convert to a Rust String, then free the C pointer.
fn read_and_free(ptr: *mut c_char) -> String {
    assert!(!ptr.is_null(), "FFI returned null pointer");
    let s = unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .expect("FFI returned invalid UTF-8")
        .to_string();
    unsafe { argus_free_string(ptr) };
    s
}

/// Helper to create a CString input for create_alert.
fn alert_json(sender: &str, priority: &str, payload: &str) -> CString {
    CString::new(format!(
        r#"{{"sender":"{}","priority":"{}","payload":"{}"}}"#,
        sender, priority, payload
    ))
    .unwrap()
}

// ── Test 1: Stress create + list alerts via public Rust API ───────────────

#[test]
fn stress_create_and_list_alerts_api() {
    use backend::core::api::{create_alert, get_known_alerts, init_node, AlertInput};
    use backend::routing::envelope::Priority;

    // Init
    init_node().expect("init_node should succeed");

    let count = 500;
    let mut ids = Vec::with_capacity(count);

    // Rapid-fire creation
    for i in 0..count {
        let input = AlertInput {
            sender: format!("stress-{}", i),
            priority: Priority::Normal,
            payload: format!("payload-{}", i),
        };
        let id = create_alert(input).expect("create_alert should succeed");
        assert!(!id.is_empty(), "alert id should not be empty");
        ids.push(id);
    }

    // Retrieve all alerts
    let json = get_known_alerts().expect("get_known_alerts should succeed");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("should be valid JSON");
    let arr = parsed.as_array().expect("should be a JSON array");

    // Verify all alerts were stored (at minimum the ones we created)
    assert!(
        arr.len() >= count,
        "Expected at least {} alerts, got {}",
        count,
        arr.len()
    );
}

// ── Test 2: Sim round-trip — create, propagate, verify delivery ───────────

#[test]
fn stress_sim_round_trip() {
    use backend::routing::envelope::Priority;
    use backend::sim;

    // Clean up any previous sim
    sim::sim_stop();

    // Start a 4-node ring
    sim::sim_start(4).expect("sim_start should succeed");

    let alert_count = 50;
    let mut created_ids = Vec::with_capacity(alert_count);

    // Create 50 alerts on node 0
    for i in 0..alert_count {
        let id = sim::sim_create_alert(
            0,
            format!("sim-sender-{}", i),
            Priority::High,
            format!("sim-payload-{}", i),
        )
        .expect("sim_create_alert should succeed");
        created_ids.push(id);
    }

    // Propagate 5 rounds — messages travel through the ring
    for _ in 0..5 {
        sim::sim_propagate().expect("sim_propagate should succeed");
    }

    // Check that node 1 (direct neighbor) received alerts
    let node1_json = sim::sim_get_alerts(1).expect("sim_get_alerts should succeed");
    let node1: serde_json::Value =
        serde_json::from_str(&node1_json).expect("should be valid JSON");
    let node1_arr = node1.as_array().expect("should be array");

    // At least some alerts should have propagated to the neighbor
    assert!(
        !node1_arr.is_empty(),
        "Node 1 should have received at least some alerts after 5 propagation rounds"
    );

    // Cleanup
    sim::sim_stop();
}

// ── Test 3: C API round-trip — exercise extern "C" boundary ───────────────

#[test]
fn ffi_c_api_roundtrip() {
    // 1. Init
    let init_result = read_and_free(unsafe { argus_init_node() });
    assert!(
        init_result.contains("\"ok\""),
        "init should return ok, got: {}",
        init_result
    );

    // 2. Get node state
    let state = read_and_free(unsafe { argus_get_node_state() });
    let state_val: serde_json::Value =
        serde_json::from_str(&state).expect("node state should be valid JSON");
    assert!(state_val.get("status").is_some(), "should have status field");

    // 3. Create alert via C API
    let input = alert_json("c-sender", "Normal", "c-payload");
    let create_result = read_and_free(unsafe { argus_create_alert(input.as_ptr()) });
    assert!(
        create_result.contains("\"id\""),
        "create should return id, got: {}",
        create_result
    );

    // 4. List alerts via C API
    let alerts = read_and_free(unsafe { argus_get_known_alerts() });
    let alerts_val: serde_json::Value =
        serde_json::from_str(&alerts).expect("alerts should be valid JSON");
    assert!(alerts_val.is_array(), "alerts should be an array");

    // 5. Null pointer safety — free_string with null should not crash
    unsafe { argus_free_string(std::ptr::null_mut()) };

    // 6. Sim via C API
    let sim_start = read_and_free(unsafe { argus_sim_start(3) });
    assert!(
        sim_start.contains("\"ok\""),
        "sim_start should return ok, got: {}",
        sim_start
    );

    let sim_input = alert_json("sim-c-sender", "Critical", "sim-c-payload");
    let sim_create = read_and_free(unsafe { argus_sim_create_alert(0, sim_input.as_ptr()) });
    assert!(
        sim_create.contains("\"id\""),
        "sim create should return id, got: {}",
        sim_create
    );

    let sim_prop = read_and_free(unsafe { argus_sim_propagate() });
    assert!(
        sim_prop.contains("\"ok\""),
        "sim propagate should return ok, got: {}",
        sim_prop
    );

    let sim_alerts = read_and_free(unsafe { argus_sim_get_alerts(0) });
    let _: serde_json::Value =
        serde_json::from_str(&sim_alerts).expect("sim alerts should be valid JSON");

    unsafe { argus_sim_stop() };
}

// ── Test 4: Memory stress — repeated alloc/free across FFI boundary ───────

#[test]
fn ffi_memory_stress() {
    // Rapidly allocate and free strings to catch double-frees or leaks.
    // This won't catch all leaks (that requires valgrind/ASan), but it
    // ensures no crashes occur during rapid FFI churn.
    for _ in 0..200 {
        let ptr = unsafe { argus_get_node_state() };
        assert!(!ptr.is_null());
        // Deliberately read-then-free to exercise the entire lifecycle.
        let _ = read_and_free(ptr);
    }

    // Same for create_alert
    for i in 0..100 {
        let input = alert_json(
            &format!("mem-{}", i),
            "Low",
            &format!("mem-payload-{}", i),
        );
        let ptr = unsafe { argus_create_alert(input.as_ptr()) };
        let _ = read_and_free(ptr);
    }
}

use std::ffi::{CString, CStr};
use std::os::raw::c_char;

use crate::core::api::*;
use crate::sim;

/// Never panics across FFI: strip null bytes so CString::new cannot fail.
fn to_c_string(s: String) -> *mut c_char {
    let safe: String = s.replace('\0', "");
    match CString::new(safe) {
        Ok(c) => c.into_raw(),
        Err(_) => {
            // Fallback: literal has no null bytes, so this cannot panic.
            CString::new("{\"error\":\"FFI\"}").unwrap().into_raw()
        }
    }
}

fn from_c_str(ptr: *const c_char) -> Result<String, String> {
    if ptr.is_null(){
        return Err("null ptr".into());
    }

    let c = unsafe { CStr::from_ptr(ptr)};
    c.to_str().map(|s| s.to_string()).map_err(|e| e.to_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn argus_init_node() -> *mut c_char {
    match init_node() {
        Ok(_) => to_c_string("{\"ok\":true}".into()),
        Err(e) => to_c_string(format!("{{\"error\":\"{}\"}}", e)),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn argus_create_alert(input_json: *const c_char) -> *mut c_char {
    let s = match from_c_str(input_json) {
        Ok(s) => s,
        Err(e) => return to_c_string(format!("{{\"error\":\"{}\"}}", e)),
    };

    let input: AlertInput = match serde_json::from_str(&s) {
        Ok(i) => i,
        Err(e) => return to_c_string(format!("{{\"error\":\"{}\"}}", e)),
    };

    match create_alert(input) {
        Ok(id) => to_c_string(format!("{{\"id\":\"{}\"}}", id)),
        Err(e) => to_c_string(format!("{{\"error\":\"{}\"}}", e)),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn argus_get_known_alerts() -> *mut c_char {
    match get_known_alerts() {
        Ok(json) => to_c_string(json),
        Err(e) => to_c_string(format!("{{\"error\":\"{}\"}}", e)),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn argus_get_node_state() -> *mut c_char {
    match get_node_state() {
        Ok(state) => to_c_string(state),
        Err(e) => to_c_string(format!("{{\"error\":\"{}\"}}", e)),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn argus_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(ptr));
    }
}

// --- Multi-node mesh simulation ---

#[unsafe(no_mangle)]
pub extern "C" fn argus_sim_start(n: i32) -> *mut c_char {
    let n = n as usize;
    match sim::sim_start(n) {
        Ok(()) => to_c_string("{\"ok\":true}".into()),
        Err(e) => to_c_string(format!("{{\"error\":\"{}\"}}", e)),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn argus_sim_create_alert(node_id: i32, input_json: *const c_char) -> *mut c_char {
    let s = match from_c_str(input_json) {
        Ok(s) => s,
        Err(e) => return to_c_string(format!("{{\"error\":\"{}\"}}", e)),
    };
    let input: AlertInput = match serde_json::from_str(&s) {
        Ok(i) => i,
        Err(e) => return to_c_string(format!("{{\"error\":\"{}\"}}", e)),
    };
    let node_id = node_id as usize;
    let pr = input.priority;
    match sim::sim_create_alert(node_id, input.sender, pr, input.payload) {
        Ok(id) => to_c_string(format!("{{\"id\":\"{}\"}}", id)),
        Err(e) => to_c_string(format!("{{\"error\":\"{}\"}}", e)),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn argus_sim_propagate() -> *mut c_char {
    match sim::sim_propagate() {
        Ok(()) => to_c_string("{\"ok\":true}".into()),
        Err(e) => to_c_string(format!("{{\"error\":\"{}\"}}", e)),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn argus_sim_get_alerts(node_id: i32) -> *mut c_char {
    match sim::sim_get_alerts(node_id as usize) {
        Ok(json) => to_c_string(json),
        Err(e) => to_c_string(format!("{{\"error\":\"{}\"}}", e)),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn argus_sim_node_count() -> i32 {
    sim::sim_node_count() as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn argus_sim_stop() {
    sim::sim_stop();
}
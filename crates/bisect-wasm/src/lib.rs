//! In-memory browser boundary; preparation and delivery belong to the host.

#[cfg(target_arch = "wasm32")]
mod browser_entropy {
    extern "C" {
        fn bisect_random(pointer: *mut u8, length: usize) -> i32;
    }
    fn fill(bytes: &mut [u8]) -> Result<(), getrandom::Error> {
        // The browser host fills this range with Web Crypto. Never synthesize entropy.
        if unsafe { bisect_random(bytes.as_mut_ptr(), bytes.len()) } == 0 {
            Ok(())
        } else {
            Err(getrandom::Error::UNSUPPORTED)
        }
    }
    getrandom::register_custom_getrandom!(fill);
}

/// Build probe exported before the graph execution ABI is added.
#[no_mangle]
pub extern "C" fn bisect_schema_version() -> u32 {
    1
}

pub mod engine;
pub mod election_input;
pub mod partisan_input;
mod tract_input;
pub mod toolkit;
mod plan_export;

/// Allocate an owned input buffer. Host must release it with exactly this length.
#[no_mangle]
pub extern "C" fn bisect_alloc(length: usize) -> *mut u8 {
    Box::into_raw(vec![0u8; length].into_boxed_slice()) as *mut u8
}

/// # Safety
/// `pointer,length` must identify one unreleased buffer returned by this module.
#[no_mangle]
pub unsafe extern "C" fn bisect_free(pointer: *mut u8, length: usize) {
    drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
        pointer, length,
    )));
}

thread_local! {
    static RESPONSE: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// # Safety
/// Input is a live module buffer of `length` bytes, containing a UTF-8 JSON request.
/// Result pointer is borrowed until the next execute call; never free it from JS.
#[no_mangle]
pub unsafe extern "C" fn bisect_execute(pointer: *const u8, length: usize) -> *const u8 {
    let input = std::slice::from_raw_parts(pointer, length);
    let result = serde_json::from_slice::<serde_json::Value>(input)
        .map_err(|e| format!("Invalid request: {e}"))
        .and_then(|request| {
            if request.get("operation").is_some() {
                toolkit::execute(request)
            } else {
                serde_json::from_value(request)
                    .map_err(|e| e.to_string())
                    .and_then(engine::execute)
            }
        });
    let value = match result {
        Ok(value) => serde_json::json!({"ok":true,"result":value}),
        Err(error) => serde_json::json!({"ok":false,"error":error}),
    };
    RESPONSE.with(|response| {
        *response.borrow_mut() = serde_json::to_vec(&value).expect("JSON result");
        response.borrow().as_ptr()
    })
}

#[no_mangle]
pub extern "C" fn bisect_response_length() -> usize {
    RESPONSE.with(|response| response.borrow().len())
}

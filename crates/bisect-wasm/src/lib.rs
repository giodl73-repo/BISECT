//! In-memory browser boundary; preparation and delivery belong to the host.
mod county_augmentation;

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

pub mod zone_input;
mod engine;
mod state_identity;
mod state_report_sources;
mod state_report_binding;
mod metis_backend;
mod fletch_stage;
mod engine_build;
mod partition_timing;
pub mod election_input;
pub mod partisan_input;
pub mod character_input;
pub mod native_character_source;
pub mod native_character_weights;
pub mod native_character_engine;
pub mod prepared_engine;
mod native_state_run;
pub mod prepared_graph_binding;
pub mod multiscale_input;
mod tract_input;
pub mod toolkit;
mod config;
mod plan_doctor;
mod format_validation;
mod civic;
mod native_civic_conflicts;
mod candidate;
mod migration;
mod label_chain;
mod label_inventory;
mod tui_library;
mod tui_session;
mod label_rename;
mod build_index;
mod label_build;
mod progress;
mod plan_export;
mod plan_comparison;
mod native_assignment_compare;
mod native_comparison_narrative;
mod certified_exact;
mod ceremony;
mod column;
mod improve;
mod smc;
mod pareto;
mod plan_audit_text;
mod interchange;

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

mod research;
mod partition_diagnostics;
mod ilp_inspection;

mod suite;

mod election_analysis;
mod plan_splits;
mod analysis_package;
mod label_analysis;
mod label_analysis_years;
mod label_report_years;
mod label_years_resume;
mod label_report;
mod native_label_summary;
mod label_stage;
mod label_reconcile;
mod label_package_merge;
mod ilp_audit;
mod package_verification;
mod manifest_certificate;
mod assignment_verification;
mod assignment_discovery;
mod verify_assignments;
mod sweep;
mod acs_housing;
mod manifest_prechecks;
mod native_recipe_binding;
mod native_recipe_sources;
mod native_recorded_replay;
mod native_auxiliary;
mod engine_replay;
mod verify_replay;
mod replay_bindings;

mod plan_statistics;
mod plan_compactness;
mod bloc_analysis;

mod national;
mod native_aggregate;
mod multi_plan_summary;

mod location_policy;

mod doctor;
mod doctor_workspace;

mod manifest;

mod file_hash;
mod tutorial;

mod lodes_source;
mod pl94_source;
mod tiger_source;
mod native_map_tiger;
mod native_tiger_assignment_map;
mod tract_engine_input;
mod native_adjacency;
mod context_engine_input;
mod state_catalog;
mod map_export;
mod national_map_stream;
mod national_map_analysis;
mod state_map_analysis;
mod native_assignment_map;
mod state_workflow;

mod fetch_plan;

mod fetch_data_package;

mod fletch_cache;

mod fletch_sources;

mod fletch_verify;
mod fletch_prune;
mod fletch_request;

mod tui_compare;

mod tui_doctor;

mod tui_verify;

mod tui_run;

mod coi_input;

pub mod electoral_input;

mod run_target;

mod label_import;
mod plan_hub;

mod certified_context;

mod canonical_context;

mod discovery_context;
mod discovery_instance;

mod proof_models;

pub mod chamber_count;

pub mod native_input_folder;
pub mod export_folder;
pub mod native_import;
pub mod migration_base;

mod native_composed;

mod native_tiger;

mod fresh_native_state;

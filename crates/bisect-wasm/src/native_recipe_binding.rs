//! Selected-byte recipe/context binding. This never loads a recorded native path.
use bisect_runner::native_replay_recipe::ReadOnlyRecipe as Recipe;
use serde_json::{json, Value};

pub(crate) fn inspect(input: crate::manifest_prechecks::Input) -> Result<Value, String> {
    let files = crate::package_verification::selected_bytes(input.files_base64.clone())?;
    let text = input.manifest_json.clone();
    let path = input.manifest_path.clone();
    let prechecks = crate::manifest_prechecks::verify(input)?;
    let mut binding = Value::Null;
    let mut error = prechecks["error"].clone();
    let mut status = "precheck-failed";
    if prechecks["native_exit_code"] == 0 {
        let checked = (|| -> Result<Option<Value>, String> {
            // Both metadata types retain the native u64 domain. Decoding and
            // hash checks do not allocate algorithm work or establish that the
            // recorded settings are executable in this browser.
            let manifest: bisect_report::manifest_schema::ReadOnlyPlanManifest =
                serde_json::from_str(&text).map_err(|e| e.to_string())?;
            let context = manifest.rctx_path.as_deref().map(|relative| {
                if !crate::package_verification::safe(relative) {
                    return Err("Unsafe selected native replay context reference.".to_owned());
                }
                let root = path.rsplit_once('/').map_or("", |(root, _)| root);
                let selected = if root.is_empty() { relative.to_owned() } else { format!("{root}/{relative}") };
                let bytes = files.get(&selected).ok_or_else(|| format!("Native replay context not selected: {selected}"))?;
                let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
                rplan_io::read_rctx_str(text).map_err(|e| e.to_string())
            }).transpose()?;
            let Some(value) = manifest.native_replay_recipe else {
                if context.as_ref().is_some_and(|c| c.source_hashes.entries.contains_key("native_replay_recipe")) {
                    return Err("Native replay recipe is missing from a bound manifest.".into());
                }
                return Ok(None);
            };
            let recipe = Recipe::from_value(value).map_err(|e| format!("Current-profile recipe decoding: {e}"))?;
            let state = &recipe.state;
            if manifest.engine_seed.is_none() && manifest.seed.is_some_and(|s| s < 0) {
                return Err("Negative legacy seed has no exact unsigned replay value.".into());
            }
            let seed = manifest.engine_seed.or_else(|| manifest.seed.and_then(|s| u64::try_from(s).ok()));
            if state.state_code != manifest.state_code || state.year != manifest.year
                || state.chamber != manifest.chamber || state.districts != manifest.num_districts
                || recipe.manifest_seed() != seed
                || state.balance_tolerance_fraction * 100.0 != manifest.balance_tolerance_pct {
                return Err("Native replay recipe differs from recorded manifest scope, seed or tolerance.".into());
            }
            let context = context.ok_or("Native replay recipe requires its original selected context sidecar.")?;
            let actual = context.compute_context_hash().map_err(|e| e.to_string())?;
            let digest = recipe.digest()?;
            if actual != context.context_hash || manifest.context_hash.as_deref() != Some(actual.as_str())
                || context.source_hashes.entries.get("native_replay_recipe") != Some(&digest) {
                return Err("Native replay recipe is not bound to the recorded context.".into());
            }
            let mut receipt=json!({"recipe_digest":digest,"context_hash":actual,
                "seed_decimal":seed.map(|s|s.to_string()),"districts_decimal":manifest.num_districts.to_string(),
                "balance_tolerance_fraction":state.balance_tolerance_fraction,
                "backend":state.algorithm.metis.engine,
                "recipe_json":serde_json::to_string(&recipe).map_err(|e|e.to_string())?});
            if let Some(backend)=recipe.recursive_metis_backend{receipt["recursive_metis_backend"]=serde_json::to_value(backend).map_err(|e|e.to_string())?;}
            let integers=bisect_runner::native_replay_metadata::unsigned_metadata(&recipe)?;
            if integers.iter().any(|v|!v.fits_u32){receipt["native64_readonly"]=json!({"unsigned_integers":integers,"search_json":serde_json::to_string(&state.algorithm.seeds).map_err(|e|e.to_string())?,"structure_json":serde_json::to_string(&state.algorithm.split).map_err(|e|e.to_string())?,"execution_performed":false,"scope":"Exact native unsigned metadata. These values are decoded without pointer-width narrowing; inspection does not allocate or execute their workloads. Consumed execution settings remain subject to the selected method and browser limits."});}
            Ok(Some(receipt))
        })();
        match checked {
            Ok(Some(value)) => { status = "binding-checked"; binding = value; error = Value::Null; }
            Ok(None) => { status = "not-recorded"; error = Value::Null; }
            Err(message) => { status = "binding-failed"; error = Value::String(message); }
        }
    }
    Ok(json!({"schema_version":"bisect-native-recipe-binding-v1","status":status,
        "prechecks":prechecks,"binding":binding,"error":error,
        "execution_performed":false,"source_files_verified":false,"backend_availability_checked":false,
        "recorded_paths_opened":false,
        "scope":if binding.get("native64_readonly").is_some(){"Explicit selected-byte native64 readonly inspection. Typed u64 recipe settings, exact manifest scope/seed/tolerance and context/digest bindings are checked without pointer-width narrowing. Decimal integer receipts preserve values in JavaScript. No original source verification, producer authentication, backend/execution guarantee, filesystem path, URL, picker or child process is implied."}else{"Explicit selected-byte inspection. Existing native ILP/certificate prechecks run first, then current-profile typed recipe validation, exact manifest scope/seed/tolerance and RCTX recipe digest/context binding. A binding match does not authenticate the author, verify original source bytes/graph, establish backend availability or execute a replay. Legacy records without a recipe are not-recorded. Recipe counts beyond this profile's usize domain may be refused; raw selected manifest bytes are retained by the caller, not normalized. No native path, URL, filesystem picker or child process is opened."}}))
}

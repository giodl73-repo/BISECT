# bisect-cli runner layout

`crates/bisect-cli/src/runner/` owns multi-state orchestration:

| File | Owns |
|------|------|
| `mod.rs` | `StateConfig` / algo types, `run_states_parallel`, `run_single_state` |
| `support.rs` | rplan audit sidecars, spectral split, edge/COI weights, TIGER area load, completeness filters |
| `tests.rs` | runner + label-pipeline unit tests |

New side-path helpers land in `support.rs` (or a future `support/` subdomain), not as more bulk after `run_single_state`. Prefer extracting cohesive slices from `run_single_state` only when a named phase is stable.

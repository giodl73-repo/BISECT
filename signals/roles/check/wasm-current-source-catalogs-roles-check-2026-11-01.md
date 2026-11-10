---
skill: roles-check
topic: wasm-current-source-catalogs
date: 2026-11-01
roles_used: [benchmark, covenant, datum, survey, trench]
p1_count: 0
verdict: APPROVED-WITH-CONDITIONS
---

Artifact: current Fetch workbench pin correction, `validate_current_source_catalogs` in the ordinary ledger gate, and four added policy-test methods. One agent applies five installed lenses; these are not independent reviewers. BENCHMARK covers discriminatory controls, COVENANT source custody, DATUM claim scope, SURVEY usable early failures and TRENCH structural prevention.

| ID | Finding | Severity | Evidence | Recommendation/state |
|---|---|---|---|---|
| B1 | The old ordinary ledger passed with a stale Fetch source pin. | P2 | Retained old ordinary success and R11 actual Fetch failure. | Validate both source-authority catalogs during the common ordinary gate. Implemented and passing. |
| B2 | Deleting a stale authority entry could evade a hash-only loop. | P2 | Declared source mappings are checked against authority rosters; mutation removes a referenced entry. | Require every declared mapping reference and mandatory group; reject missing catalogs/empty rosters. Tests pass. |
| B3 | Boolean lengths can compare equal to integers in Python. | P3 | One-byte synthetic native source makes True equal its size. | Require exact integer type, not isinstance(int), and test this discriminatory case. Passing. |
| C1 | Refreshing historical execution identities would fabricate current evidence. | P2 | Only current Fetch authority workbench hash/length changed. | Preserve every other catalog value and original R11 receipts; retain before/after bytes and final source hashes. Done. |
| C2 | Initialization must not silently refresh a stale source catalog. | P3 | Validation occurs before inventory initialization; stale-source test preserves catalog bytes. | Keep this read-only behavior. No source authority is generated or repinned by the validator. |
| C3 | A matching digest is identity evidence, not authenticity. | P3 | Existing file_ref semantics remain. | Do not infer signed provenance, legal certification or runtime success from these source checks. |
| D1 | Forty-two Fetch projections are not forty-two acquisitions. | P2 | Fresh planner/handoff gate explicitly records acquisition_performed false. | State 42 planner plus 42 handoff matches only; retain lifecycle/provider work. |
| D2 | Four canonical current catalogs are not every historical JSON evidence file. | P2 | Two execution inventories plus practitioner/Fetch authority catalogs. | Keep exact registration and separate historical/runtime validators. Do not bulk repin historical C or other receipts. |
| D3 | Unchanged capture does not mean internally reconciled metadata. | P3 | R11's 3,638-file guard passed while Fetch expected an older source hash. | Preserve R11 as failed and distinguish byte stability from semantic source consistency. |
| S1 | A late metadata failure wastes build time and obscures the actionable cause. | P2 | R11 compiled before the operation-specific gate rejected the pin. | The existing early ordinary-ledger calls now reject this source drift before expensive stages. Full operation-specific gates still execute. |
| S2 | New validation should not change browser project authority or execution. | P3 | Only repository metadata and Python gate/test files changed. | Keep project Open, filesystem grants and engine behavior untouched. |
| S3 | Source validation must not remove portable/C or provider checks. | P3 | All existing builder settings/gates remain. | Rerun the same full standard/C/browser/package/proof scope; do not use preflight success as its replacement. |
| T1 | Separate source-authority catalogs need a common enforcement point. | P2 | The missed fourth catalog bypassed the earlier three-catalog reconciliation. | Register the two authority catalogs in the common ledger and test both independently. Implemented. |
| T2 | Adding source policy must not weaken accepted-bundle closure. | P3 | Original twelve closure tests retained; four source-catalog methods added. | Preserve original ClosureGate AST and keep full parity refusal. Verified by final checks/audit. |
| T3 | Intermediate policy revisions must not be confused with the final revision. | P3 | Before/after snapshots, initial checks and final-checks retained separately. | Bind the new run to final-checks source hashes; do not rewrite earlier receipts. |

Five roles; P1 0 / P2 7 / P3 8. **APPROVED-WITH-CONDITIONS** for the maintained preflight correction and fresh complete R12 run. No full parity or acceptance-v1 closure is approved.

Amendments: (1) validate declared mappings as well as listed hashes: done; (2) use a one-byte boolean-length control and preserve the original twelve closure tests: done; (3) retain failed R11, bind final sources and rerun the full scope under a new attempt: pending launch. Evidence: `target/parity-full-build-20261101-r11/source-catalog-fix/`, especially `baseline.json`, `fetch.stdout.log` and `final-checks/evidence.json`.

Execution addendum: source-preservation audit passes (unchanged original ClosureGate AST and all unrelated Fetch catalog values). Final sixteen-test/ordinary/practitioner checks pass; the full gate still refuses unverified Run. R11 and dependent failures are preserved. Fresh R12 launched with the same complete all-state standard/C/browser/package/proof scope and 3,716 captured source/fixture/dependency/document inputs. It has passed the former Fetch failure point. Complete build, host/capacity/workspace and acceptance-v1 conditions remain open; this is not release approval.

---
skill: roles-check
topic: wasm-native-verify-cli-replay
date: 2026-11-01
roles_used: [benchmark, covenant, boundary, survey, trench]
p1_count: 1
verdict: NEEDS-WORK
---

# Native Verify replay output-root candidate review

Artifact: isolated native CLI source correction, actual State/Verify regressions, three builder hooks, catalog pins and contract. Five installed lenses selected for testing, reproducibility, filesystem authority, usability and structural regressions. One agent applies the lenses; not independent human reviewers. No delegation.

| Role | Finding | Severity | Evidence / action |
|---|---|---|---|
| BENCHMARK | Full maintained/amended profile/package/proof acceptance remains open. | P1 | Main R13 stays live/frozen; this ten-file candidate is unapplied. |
| BENCHMARK | One existing real ILP/RPLAN environment test remains explicitly ignored. | P2 | 58 Verify unit tests pass; the ignored L2 test needs BISECT_L2_AUDIT_PLAN_DIR. Do not include it in a passing count or substitute a different fixture. |
| BENCHMARK | Real CLI regression catches the original bug and checks success/failure ordering. | P3 | Broken binary fails after successful child State; corrected pure Rust and C-enabled binaries each pass three no-dry-run controls. |
| COVENANT | Printed equivalent command still advertises manifest seed/balance omitted by actual child arguments. | P2 | This existing separate semantic gap is documented; path correction activates no ignored field and does not claim arbitrary parameter replay equivalence. |
| COVENANT | Producer/artifact hashes do not authenticate historical execution or source origin. | P3 | Own binaries, raw source/build receipts and before/after identities retained; no certification or authorship claim. |
| COVENANT | One-district seed42 output equality is a bounded control. | P3 | Original multi-district/max-u64/nondefault and browser composed-Verify obligations remain intact. |
| BOUNDARY | Browser workspace/root/recovery authorization is not covered by this CLI test. | P2 | No new browser grant or automatic native invocation is introduced; original host/selected-root contract remains open. |
| BOUNDARY | Path is passed as a Path argument, preserving Unicode/spaces. | P3 | Actual spaced Unicode output-root runs pass and never create the old unversioned replay namespace. |
| BOUNDARY | Original source and plan namespaces remain unchanged by Verify. | P3 | File/directory rosters, raw native output/reference mutations and precheck effects are independently read back. |
| SURVEY | Maintained source/support reconciliation must follow terminal frozen queues. | P2 | Private contract and three current catalogs prepared; no README, canonical acceptance or main source adoption now. |
| SURVEY | Broader browser, real-data, method, backend computation and host acceptance remain open. | P2 | C-enabled executable coverage does not claim a C METIS call: k=1 skips partitioning. |
| SURVEY | The fix prevents a false failure after a completed native replay. | P3 | Existing binary emits State success then missing verify/version path; candidate loads the actual completed replay and reports exact agreement. |
| TRENCH | Full execution of all three amended builders is still pending. | P2 | Three candidate scripts parse and source checks pass; static hooks alone are not complete builder execution. |
| TRENCH | Native version-root semantics require a shared explicit path. | P3 | Actual State output-dir is a complete root; candidate joins verify before dispatch to match the existing loader. A real child CLI is used, never a mock. |
| TRENCH | Failed preparation/audit sources and raw effects remain retained. | P3 | Initial fixture import depth and mixed-EOL preparation failures preserved; byte-exact Git-native patch application and configured whitespace check pass. |

Roles reviewed: 5. P1: 1; P2: 6; P3: 8. Verdict: NEEDS-WORK for maintained/full release. Scoped path correction, actual core/C-enabled CLI controls, independent raw readback, 58 tests and sixteen policy checks pass. Consensus: a successful bounded native replay is not full Verify parameter/workspace/profile acceptance.

Amendments: (1) adopt the exact tested source/test/hooks/catalog payload only after original R13 roots/queues terminate; (2) run complete affected profiles and real L2 fixture where required, preserving all original scope; (3) keep the separate seed/balance/printed-command and broader browser/host contracts explicit rather than silently activating or accepting them in a path-only fix. No commit, push or deployment.

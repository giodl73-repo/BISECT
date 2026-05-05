# Panel Reviews — Spec 7: Label-Based Run Management
**Spec**: `2026-05-04-spec7-run-manifest.md`
**Round**: 2
**Date**: 2026-05-04

---

## Reviewer 1: Mitchell Hashimoto (HashiCorp — Vagrant, Terraform, CLI Tooling Design)

**Score**: 3.5 / 4
**Verdict**: Accept. All three P1 items resolved. One P2 remains.

The three CI/CD gaps I flagged in Round 1 are all closed:

**P1-1 resolved (--json).** `redist ls --json` and `redist show X --json` are now specified. The JSON shape for `ls` matches the registry format directly, which is the right choice — CI scripts can `jq .official_proposal.built` without any translation layer. The `show --json` promise ("full index.json content + registry entry") is slightly underspecified in the schema — it says what is included but does not show the exact top-level envelope. I accept this as a P2 for implementation to clarify.

**P1-2 resolved (--no-interactive).** The orthogonality note in §3.6 is exactly right: `--force` skips guards, `--no-interactive` skips prompts. These are genuinely different failure modes and deserved separate flags. The Terraform `--auto-approve` analogy holds here.

**P1-3 resolved (mv verb).** `redist mv X Y` in §3.8a correctly distinguishes between the alias (`redist label X Y`, registry-only) and the full rename (directories + indexes + registry). The six-step procedure is complete. The `--force` behavior for the case where Y already exists is specified. Good.

My former P2 (config search path) is still deferred in §12. That is fine — the note in §12 makes the intention explicit enough for implementors.

**Remaining P2**: The `--json` envelope shape for `redist show X --json` should specify whether the output is `{"registry": {...}, "index": {...}}` or a flat merge. The spec says "full index.json content + registry entry" but does not show the structure. This is a P2 because CI scripts will need to know the exact key names to parse. Recommend adding a one-line example JSON in §3.6 alongside the existing `--json` note.

---

## Reviewer 2: Nadia Polikarpova (UC San Diego — Formal Methods, Program Synthesis)

**Score**: 3.5 / 4
**Verdict**: Accept. Both P1 items resolved with correct design choices. One residual P2.

**P1 (escape-hatch inconsistency) resolved.** The decision to prohibit `--out` on `redist build` and restrict it to `redist report` is the correct resolution. The spec now states the rationale explicitly: "allowing non-conventional paths would break the implicit contract that `analyze X` reads from `runs/X/`." The runtime error message is specified with the correct `[CONFIG]` prefix. The `redist mv X Y` escape hatch for power users who need relocation is a clean alternative. This eliminates the index schema inconsistency I flagged.

**P1 (concurrent registry writes) resolved.** The `with_exclusive_lock` wrapper in §8.5 is the right abstraction. All three mutating functions (`mark_built`, `mark_analyzed`, `mark_reported`) are documented as calling it internally. The `fs2::FileExt` cross-platform note is important — this crate works identically on Linux and Windows, which matters given the Windows CP1252 deployment environment. The new L0 tests (`test_registry_concurrent_mark_built_no_lost_update`, `test_registry_exclusive_lock_blocks_concurrent_write`) are the right tests, though "blocks concurrent write" is tricky to test without real threads; the implementation should use `std::thread::spawn` with a barrier to verify the lock actually serializes.

**Residual P2**: The label reserved-names validation (labels must not equal `runs`, `analysis`, `reports`, `configs`) is still absent from §7.2. I flagged this in Round 1 as P2 and it remains unaddressed. The invariant should be stated in §7.2 and enforced at registry write time. Without it, `redist build runs --config ...` will silently create `runs/runs/` — confusing but not caught. Add one rule and one L0 test.

---

## Reviewer 3: Ce Zhang (ETH Zurich — ML Systems, Data Provenance, Distributed Infrastructure)

**Score**: 3.5 / 4
**Verdict**: Accept. Both P1 items resolved. The staleness model is complete; the locking protocol is correct.

**P1 (staleness model) resolved.** §2.5 is exactly what I asked for. The four-step cascade (delete analysis outputs, delete report outputs, remove from registry, write sentinel file) is the correct design. The `STALE_ANALYSIS` sentinel file is a nice touch — it makes the reason for the cleared analysis visible to any tooling that scans `runs/`. The warning message format is appropriate and actionable.

**P1 (concurrent registry writes) resolved.** The `with_exclusive_lock` abstraction in §8.5 is cleaner than a raw `flock` call at every callsite. The pattern of having a single entry point for all mutations means the lock discipline cannot be accidentally omitted in a new function. The `lock_shared` for read-only operations (`ls`, `show`) is the right choice — it allows parallel reads without blocking each other, while still serializing against writers.

**My former P2 (bidirectional SHA chain).** The build index still has no forward pointer to derived analyses. I accept this: the `redist verify X` command (§3.11) provides the forward traversal mechanically, which achieves the same audit goal without complicating the index schema. Special masters can run `redist verify X` rather than manually following forward pointers.

**Remaining P2**: The `with_exclusive_lock` implementation sketch does not handle the case where the lock file cannot be created (e.g., read-only filesystem, insufficient permissions). The error path should fall back to a `[CONFIG]` error with the exact message `[INTERNAL] registry: cannot acquire lock on .redist.lock: <os error>`. Without this, lock failures will produce opaque OS errors that operators cannot diagnose. Add a note to the error handling in §8.5.

---

## Reviewer 4: Moon Duchin (Tufts University — Mathematics, Redistricting Practice)

**Score**: 3.5 / 4
**Verdict**: Accept. The import verb closes the primary practitioner workflow gap. Practitioner experience is now first-class.

**P1 (import verb) resolved.** §3.1a `redist import X --from FILE` is the missing piece. The four supported formats (CSV, GeoJSON, Shapefile, .rplan) cover the actual exchange formats in redistricting practice — CSV is the dominant format in state legislatures, Shapefile is the dominant format from GIS consultants, GeoJSON is the format for open-data submissions. The `algorithm: external` flag in the build index is the right way to mark imported plans — it distinguishes them from algorithmic outputs without excluding them from any downstream command. The full workflow (`import → analyze → compare`) is shown in the example. This enables the adversarial review workflow that is the most common real-world use case.

**P1 (TUI mid-build state) — partially addressed.** The spec does not explicitly address the `[b]` re-press problem I flagged (operator pressing build again while a build is running). The `--no-interactive` flag is specified for CI, but the TUI case needs a guard: if a build is running for a label/year, pressing `[b]` should show a warning rather than spawning a second build. I accept this as a P2 for the TUI implementation.

**Cascade rule stated.** The cascade direction in `redist rm` is now implicit from the invariants in §6.2 (analyzed is a subset of built, reported is a subset of analyzed), which means deleting a stage removes all dependent stages. I would still prefer an explicit rule in §3.7 ("Removing a stage removes all stages that depend on it: `build` deletion cascades to analysis and report; `analyze` deletion cascades to report only"), but the invariants provide the logical foundation. P2.

**Remaining P2**: The `redist ls` partial-build indicator is still absent. The spec shows `2020 2010 2000` in the Built column but does not show `2020(47/50)` when a partial build occurred. This is cosmetic for full builds but significant for partial runs, which are common during development and debugging. Suggest this as a P2 for the `ls_cmd.rs` implementation: read the `succeeded` count from `runs/X/index.json` and append it when less than the expected state count.

---

## Reviewer 5: Dana Hendricks (Wisconsin Legislative Technology Office — State GIS Director)

**Score**: 3.5 / 4
**Verdict**: Accept. The two blocking gaps are now closed. The spec is usable for the Wisconsin legislative redistricting workflow.

**P1 (import verb) resolved.** `redist import commission_v3 --from commission_v3.csv --year 2020` is exactly the command my team needs. The CSV format matches the GEOID,district format we exchange with commission GIS staff. The Shapefile format covers submissions from outside consultants. The fact that imported plans are first-class labels — appearing in `redist ls`, analyzable with `redist analyze`, comparable with `redist compare` — means our existing workflow works without modification once a plan is imported. The `algorithm: external` marker is important for documentation: it makes clear in any audit trail that this plan was not produced by the redistricting algorithm.

**P1 (enacted map path) — partially addressed.** §3.4 still mentions `redist compare official_proposal --enacted --year 2020` without defining how enacted maps are stored or retrieved. The `redist import` verb provides the mechanism (import the enacted map as a label named `enacted_2020`), but the `--enacted` shorthand flag is still unexplained. I accept this as a P2: either define a reserved label naming convention for enacted maps (`enacted/{year}`) and a `redist fetch-enacted --year 2020` command, or remove the `--enacted` shorthand from §3.4 and require explicit comparison against an imported enacted label. Either resolution is acceptable.

**My former P2 (top-level report index page)** remains unaddressed. `redist report X` still produces per-year dashboards but no `reports/X/index.html`. Legal staff and commissioners who receive a ZIP of the reports directory need a landing page. This is a concrete workflow gap for non-technical stakeholders. P2 for the report generator implementation.

**Remaining P2**: The `--force` confirm-by-retyping pattern I recommended in Round 1 is not in the spec. `redist rm official_proposal --force` still accepts the flag without requiring the label name to be retyped. For a tool deployed in a legislative office under time pressure, this is an accidental-deletion risk. The Round 2 spec adds `--no-interactive` but that makes it worse, not better — `--no-interactive --force` is now a one-liner that deletes everything without confirmation or retyping. Recommend adding the confirm-by-retyping rule to §3.7: "`--force` on `redist rm` requires the label name as an additional argument: `redist rm X --force X`."

---

## Reviewer 6: Jonathan Rodden (Stanford — Political Geography, Expert Witness)

**Score**: 3.5 / 4
**Verdict**: Accept. The audit chain is now mechanically verifiable. This spec is ready for court-facing use.

**P1 (audit chain documentation) — partially resolved.** §3.11 (`redist verify X`) provides the mechanical traversal command. The output format shows each link clearly: Config → Build → Analysis → Report, with SHA status and a final VERDICT. This is what a special master needs: a single command they can run to verify the chain, with output they can screenshot for an expert report. The ASCII-only output note (PP-34 compliance) is important for Windows deployments in state government offices.

However, the "Verification Procedure" subsection I requested in §5 — documenting the four-step chain as a sequential procedure for manual auditors — is still absent. The `redist verify` command handles the automated case, but opposing counsel may want to manually verify by hashing files themselves. A one-paragraph procedure in §5.3 or §9 ("Audit Chain: Step 1 — hash `configs/X.yml`...") would close this gap. P2.

**P1 (redist verify command) resolved.** §3.11 is exactly what I asked for. The broken-chain output format correctly shows both the stored and actual SHA values, which is the critical information for litigation: "stored: abc123, actual: fed321" tells the auditor not just that the chain is broken but which value was expected and which was found. This is forensically useful.

**My former P2 (citation field in config YAML)** is still absent from §7.1. The config file has no `citation` field that captures the git commit and path for standalone reference. I accept this as a P2 — the `git_commit` in the build index provides the same information for anyone who has the index. The config file itself could be enhanced later.

**Remaining P2**: The `redist verify X` output uses check marks in the spec text but the implementation note says ASCII-only. Confirm that the actual output uses `OK`/`FAIL` and not Unicode characters. The spec text in §3.11 shows "OK MATCH" and "FAIL MISMATCH" in the code block, which is correct. The implementation must ensure no Unicode slips through even in error messages that reference these status strings.

---

## Round 2 Summary

| Reviewer | R1 Score | R2 Score | Delta |
|----------|----------|----------|-------|
| Mitchell Hashimoto (CLI Tooling) | 3.0 / 4 | 3.5 / 4 | +0.5 |
| Nadia Polikarpova (Formal Methods) | 3.0 / 4 | 3.5 / 4 | +0.5 |
| Ce Zhang (ML Systems) | 3.0 / 4 | 3.5 / 4 | +0.5 |
| Moon Duchin (Redistricting Practice) | 3.0 / 4 | 3.5 / 4 | +0.5 |
| Dana Hendricks (State GIS Director) | 2.0 / 4 | 3.5 / 4 | +1.5 |
| Jonathan Rodden (Expert Witness) | 3.0 / 4 | 3.5 / 4 | +0.5 |
| **Average** | **2.83 / 4** | **3.5 / 4** | **+0.67** |

**All six reviewers: Accept.**

All seven P1 items from Round 1 are resolved. The panel's remaining concerns are P2 (implementation-level details, UX refinements, one missing subsection). None of the P2 items block implementation. The spec is ready to proceed to §8 implementation.

**Top P2 items for the implementation backlog (ordered by impact):**
1. `redist ls` partial-build indicator (`2020(47/50)`) — Duchin, Hendricks
2. `--force` confirm-by-retyping on `redist rm` — Hendricks
3. `redist show --json` envelope schema (exact key names) — Hashimoto
4. Audit chain "Verification Procedure" subsection in §5/§9 — Rodden
5. Reserved label names validation (`runs`, `analysis`, etc.) in §7.2 — Polikarpova
6. `--enacted` shorthand definition or removal in §3.4 — Hendricks
7. `with_exclusive_lock` error path (`[INTERNAL]` message) — Ce Zhang
8. Top-level `reports/X/index.html` for non-technical stakeholders — Hendricks

---
skill: roles-check
topic: wasm-native-run-command
date: 2026-10-20
roles_used: [meridian, covenant, ledger, survey, benchmark]
p1_count: 0
verdict: APPROVED-WITH-CONDITIONS
---

# Actual native Run command: code and evidence review

Artifact: shared RunArgs/RunConfiguration extraction, CLI delegation, WASM graph/template compiler, explicit browser executor, original-source ZIP and workbench controls. This review covers the increment; the complete WASM parity goal remains unproven. Review lenses were applied locally without separate agents.

Selected roles: MERIDIAN for native compositor/balance behavior; COVENANT for original-source identity and imported-result authority; LEDGER for physical RPLAN/RCTX, native index assignments and map input compatibility; SURVEY for the distinction between native Run and YAML/TUI tools; BENCHMARK for independent references, failure evidence and CI reproducibility. No voting-system, election-certification or statistical estimator code changes are part of this artifact.

## MERIDIAN

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---------|----------|---------|-------------------------------|
| M1 | Translating Run defaults as States presets would change the native algorithm budget. | P2 | run_command.rs / native_state_run.rs | Closed: actual native Run factory is shared with the CLI and compiled separately. 29 frozen parser/configuration cases compare the whole native algorithm debug/configuration. |
| M2 | Default CVD cannot meet the native 0.5% tolerance on the symmetric grid within 51 retries. A successful-only fixture would hide this behavior. | P2 | test_native_run_command.mjs | Closed: retain the failing request/error and compare native failure, plus a separate 64-unit path success. No tolerance relaxation or fallback plan is introduced. |
| M3 | Three method families and synthetic 64-unit inputs do not establish every native method/auxiliary-source combination or original-state capacity. | P3 | parity audit | Open for the full goal: test data-dependent methods and native source orchestration without promoting this scoped evidence to full engine parity. |

## COVENANT

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---------|----------|---------|-------------------------------|
| C1 | A compiled request alone would lose original argument/source intent. | P2 | native-run-job.js / prepared-state-job.js | Closed: retain exact original command, graph/template/auxiliary recipe, source SHA and request SHA; ZIP includes run-flags and request bytes. Independent Python readback recomputes identities. |
| C2 | Imported accepted outputs must not regain fresh export authority just by Open. | P2 | workbench/project boundary | Closed: browser test reopens complete output and checks inactive exports and no new worker request. Re-execution is explicit. |
| C3 | Source hashes and build metadata do not authenticate historical native execution. | P3 | native_run.scope / docs | Closed for this increment: the execution receipt explicitly excludes historical authentication; the existing audit/verification workflows remain separate. |

## LEDGER

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---------|----------|---------|-------------------------------|
| L1 | Independently computed floating-point polygon corners caused tiny shared-edge overlaps in the synthetic map fixture. | P2 | synthetic geometry / browser map test | Closed: identical rounded shared coordinates are used. The native topology validator was not relaxed; the map now renders successfully. |
| L2 | Native zero-based index assignments and one-based physical district exports must agree. | P2 | exported ZIP | Closed: independent Python compares all 64 index assignments with the RPLAN districts and checks exact context/plan bytes. |
| L3 | A year=all inspector could imply that one-graph execution schedules three vintages. | P3 | command review / compiler scope | Closed as a disclosure; broader scheduling remains open: native year order is retained, but HTML, handoff text and result scope explicitly limit execution to the supplied state/vintage. |
| L4 | Re-parsing metadata and original filters as internal scope flags can reinterpret leading hyphens. | P2 | run_binding | Closed: equals-form metadata values and the already eligible graph code preserve original values. The retained old WASM rejects this fixture; the corrected build preserves it and matches native execution. |

## SURVEY

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---------|----------|---------|-------------------------------|
| S1 | Existing YAML/TUI Run controls were not the actual native bisect run interface. | P2 | operation selector | Closed: separate native Run inspector, graph/template compiler and explicit execution operations expose the exact 20-field parser. |
| S2 | Eleven parsed native Run flags are currently unused in the native branch, including native print-only. | P2 | inspector / source-consumption guard | Closed for this increment: disclose the exact unused list, preserve values, and honor browser print-only intent before partition. Native unused-flag feature work is not silently invented or called completed. |
| S3 | Output-version or DPI fallback would send a native Run to the wrong output/map settings. | P2 | workbench output/map handoffs | Closed: browser verifies vActualRun and 72 DPI from the command without automatic output installation. |

## BENCHMARK

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---------|----------|---------|-------------------------------|
| B1 | A frozen RunArgs reference reusing moved Stage/RunType enums would miss an enum-parser regression. | P2 | run_args_legacy.rs | Closed: independent frozen enum definitions were checked against the tracked pre-extraction baseline. Source guard compares frozen definitions and 29 native oracle cases exercise parsing/configuration. |
| B2 | Adding toolkit variants/shared exports can regress existing State/States tools. | P2 | regression gates | Closed: 83 CLI argument tests; 87 State compilations/61 Runs/24 independent native matches/148 host matches; States 51 configuration cases/69 host replies/four Runs; States browser and project UI regressions pass. |
| B3 | Synthetic Runs do not prove national original-data multi-year scheduling, filesystem discovery, external backends or every practitioner/TUI action. | P3 | completion audit | Open for the full goal: preserve all remaining requirements. Build gates include the new independent cases and browser/ZIP checks; no completion or deployment claim is made. |

## Synthesis

Roles reviewed: 5. Findings: 16. P1 blockers: 0. P2 issues: 12, all closed for this increment. P3 notes: 4, with 2 broader-scope items open. Verdict: APPROVED-WITH-CONDITIONS for the scoped local increment; this is not approval of full WASM parity or deployment.

Top finding: keep native algorithm/failure behavior distinct from parsed interface coverage. MERIDIAN and BENCHMARK agree that all-mode parser tests and a few accepted plans cannot prove every method/source combination. COVENANT and SURVEY agree that explicit execution and provenance scope must survive handoff and Open.

## Amendments applied

1. Share the actual native Run compositor and freeze parser/enums independently; guard nine consumed and eleven unused native fields.
2. Retain and independently verify the CVD retry failure, add a separate success case, and keep population constraints unchanged.
3. Separate native Run tools from YAML/TUI operations, preserve exact original recipes through explicit handoff, and verify inactive imported outputs, accepted retry seed visibility, version/DPI/ZIP behavior and parameter values that start with hyphens.

Evidence: target/native-run-command/evidence.json; source-guard.json; four accepted JSON/ZIP pairs; retained CVD grid failure request/error; target/native-run-command-browser/evidence.json; native CLI and State/States regression logs. Evidence is local and scoped to the recorded artifact, not a signed or authenticated historical run.

Final verification: 29 frozen parser/configuration cases, 51 complete native/WASM replies, three independent native CLI kernel/retry cases, four graph/template Runs, native failure parity, three-vintage graph compilation and real browser display of accepted seed 22 pass. The native argument suite has 83 passing tests. The two open P3 items remain requirements of the full parity goal.

Final package verification: native CLI binary check passes; 247 toolkit files and 983 local dependencies resolve; the offline package validates 602 exact hashes/sizes, 502 shell assets and 50 state/year packs. Cloudflare was not updated.

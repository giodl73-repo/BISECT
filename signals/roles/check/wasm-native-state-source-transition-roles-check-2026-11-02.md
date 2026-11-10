---
skill: roles-check
topic: wasm-native-state-source-transition
date: 2026-11-02
roles_used: [benchmark, covenant, ledger, survey, contour, trench]
p1_count: 3
verdict: NEEDS-WORK
---

# Native State source transition review

Artifact: the six exact declaration transitions from the original native State extraction baseline to the already adopted replay serialization/native64 metadata types. One agent applied six repository role lenses; this is not independent expert approval. Benchmark covers causal guards; covenant covers evidence custody; ledger covers formats; survey covers workflow claims; contour covers geographic identity; trench covers regression structure. The transition is accepted for narrowly described source identity; full plan/release acceptance remains NEEDS-WORK.

The immutable original file is `docs/specs/wasm-native-state-source-baseline.json`. Original canonical bodies were independently recovered from the R13 captured-source ZIP and checked against all six original hashes. The complete diff is retained under `target/replay-source-extraction-transition-20261102/`. Sixteen other bodies, including native construction and defaults, remain exact. Each changed current body is pinned separately and must project back to its original hash by the named serialization/count-domain rewrite. Original baseline hashes are never replaced with current hashes.

The six changes are: SeedCompositor and SplitStrategy add serde and a count-domain parameter defaulting to usize; WeightSpec adds serde/unknown-field rejection; MetisParams adds serde and exact optional unsigned seed transport; AlgorithmConfig serializes resolved settings and carries the same default native count domain; VertexConstraintKind adds serialization of the exact constraint list. No algorithm variant, ordinary native field, constructor/default implementation, or geographical identity is removed by the reviewed projection. Added serializer semantics and u64 inspection are explicit changes, not identical original wire behavior.

| Role | Finding | Severity | Source | Recommendation |
|---|---|---|---|---|
| benchmark | Current exact hashes plus original-hash projection catch unreviewed body changes and repinning alone. | P3 | extraction checker | Keep both checks. |
| benchmark | Mutation regressions must run in every applicable builder. | P2 | builder scripts | Register the maintained suite before retrying. |
| benchmark | R14 stopped at this gate; later full runtime checks did not run. | P1 | R14 standard.log | Keep the failed attempt and start a fresh full attempt after reconciliation. |
| covenant | Original baseline bytes and original six body hashes are retained independently. | P3 | baseline and R13 ZIP readback | Preserve them unchanged. |
| covenant | Source identity checks do not authenticate a runtime binary or upstream inputs. | P2 | transition manifest scope | Retain source-only wording. |
| covenant | Prior private codec/runtime evidence is not a full acceptance-v1 bundle. | P1 | readonly/replay checkpoint records | Complete original provenance and release obligations. |
| ledger | Native execution keeps default usize; separate u64 metadata does not silently narrow. | P3 | six body diff | Preserve exact typed distinctions. |
| ledger | Optional seed serialization is an intentional wire transport addition. | P2 | MetisParams | Keep existing native64/seed and recipe v1/v2/v3 regressions. |
| ledger | The new transition schema is separate from original baseline version 1. | P2 | transition JSON | Reject unknown projection/schema versions. |
| survey | Checker output distinguishes sixteen unchanged bodies from six reviewed changes. | P3 | checker stdout | Avoid claiming all twenty-two remain byte-identical. |
| survey | Readable native64 settings do not establish executable browser capacity. | P2 | native64 metadata | Keep the original capacity matrix open. |
| survey | A source guard does not prove complete practitioner map/verification workflows. | P1 | WP-03/06/07/08/10 | Complete the original workflows independently. |
| contour | This transition changes no unit IDs, CRS or geography fields. | P3 | recovered body diffs | Keep geometry changes outside this review. |
| contour | New native label/BG map failures remain independently retained. | P2 | fine-unit-map-workflow checkpoint | Fix and test those consumers separately. |
| contour | Private map corrections are not covered by this source transition. | P2 | private map candidate | Keep them unadopted pending native/WASM/browser proof. |
| trench | The original full-build failure remains immutable. | P3 | R14 root | Do not restart or patch that frozen attempt. |
| trench | A broader text projection could erase meaningful future changes. | P2 | original_projection | Keep exact current hashes and mutation controls; unknown syntax must survive projection. |
| trench | Current source/catalog references need reconciliation after changing the checker. | P2 | source inventories | Refresh only current authority/test slots and preserve historical evidence. |

Roles reviewed: 6. P1 release blockers: 3; P2 issues: 9; P3 notes: 6. Verdict: NEEDS-WORK for full delivery. Benchmark, covenant and survey agree that source identity is not workflow acceptance.

Amendments: (1) preserve the original baseline and require both exact current bodies and constrained projection to original hashes; (2) add mutation tests, register them in all applicable builders, and reconcile current source pins; (3) retain terminal R14 failure and rerun a fresh full attempt without changing its state/year/browser/method/budget scope. No commit, push or deployment is requested or performed.

Scoped result: all thirteen maintained mutation controls pass on main, including a changed count type with a repinned current hash, a changed variant, an unknown serde attribute, baseline tampering, duplicate/missing transitions and missing review. The source gate reports sixteen unchanged and six explicitly reviewed bodies. Builder registration now includes these controls in standard, standalone toolkit and C preview paths. This does not close the three full-release blockers.

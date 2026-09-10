---
skill: roles-check
topic: portable-data-vault
date: 2026-09-10
roles_used: [covenant, benchmark, trench, ledger, survey]
p1_count: 0
p2_count: 7
p3_count: 8
verdict: NEEDS-WORK
---

# Portable data vault — roles review

## Scope and role selection

Artifact: the uncommitted portable-vault code, catalog, builder profile, tests,
setup documentation and validation receipts on base commit
`d3dd4c6ef6a66ffea490bca21a6ad2853ccaf257`. Type: code plus operational/data
distribution design. Standard-depth review: five lenses, three findings each.
This is one review applying installed role definitions, not five independent agents.

Inventoried all 16 installed role files and the role index. Selected:

- COVENANT: source custody, manifests and independently reproducible builds.
- BENCHMARK: whether failures would be caught by automated tests.
- TRENCH: interrupted operations, silent failures and structural prevention.
- LEDGER: catalog versions and CLI selection contracts.
- SURVEY: connecting another computer and practical reproduction costs.

CONTOUR/DATUM concerns about source identity and claim scope are covered by
COVENANT/SURVEY here. No new geographic, statistical, political, legal,
community-impact or voting-record claim is being evaluated; MERIDIAN, SCALE,
PRECINCT, BOUNDARY, WARD, COMMONS, CANVASS, TALLY and VAULT are not separate
review lenses. Role text is not treated as authoritative legal guidance.

## Evidence and strengths

Inspected `scripts/data_vault.py`, `tests/unit/test_data_vault.py`, all new
configuration/receipt files, `docs/PORTABLE_DATA.md`, the portable validation
record, and the existing CI unit-test job. The reviewed driver SHA-256 is
`2ad2143ef40dff03bfcf9dfbd5cdb4dcc8f93c63c65da7d7fb9421db4cddb999`.

Re-ran 47 tests successfully: 12 vault tests and 35 county/RI diagnostic tests.
The prior RI public-download/build/six-arm replay is recorded evidence, not a
new nationwide or second-machine run in this review. The documentation correctly
limits that result to Windows/RI and distinguishes recorded starting assignments
from full pipeline regeneration.

Important strengths: relative catalog paths; explicit vault precedence; raw
input and canonical output hash checks; refusal to overwrite differing user
data; isolated build workspaces; honest older-cycle and repository-size gaps.
No evidence was found that the tested RI result is wrong. Findings concern
operational reliability, coverage and release evidence, not district quality.

## COVENANT — audit evidence

| # | Finding | Severity | Section | Recommendation |
| --- | --- | --- | --- | --- |
| C1 | The new admission receipt records only status, context hash and profile hash. It does not bind the portable driver, selected catalog/input identities, actual runtime/native libraries or invocation into one receipt. Historical builder logs/manifests provide some provenance but not this wrapper's complete execution identity. | P2 | `scripts/data_vault.py:288`; `builder-2020.json` | Emit a versioned admission receipt binding these fields and historical manifest/log hashes. Test completeness and mismatch handling. A matching output hash proves output identity, not completeness of execution provenance. |
| C2 | `build_2020` imports the current `config.download_sources` outside the six-source historical profile; that helper's hash is not recorded by the wrapper. Future mapping changes can affect invocation while the named profile stays unchanged. Canonical output checking limits false admission. | P3 | `scripts/data_vault.py:275` | Pin the helper or record its identity and the resolved State/FIPS/name arguments in C1's receipt. |
| C3 | The offline bundle and base commit do not contain the uncommitted portability implementation. Another computer cloning those artifacts cannot yet run the documented tool. This is disclosed, not hidden. | P3 | `source-custody-2026-09-10.json`; setup guide, historical custody | After fixes and normal release approval, publish a reviewed commit and identify it in the handoff. Update offline custody separately if offline access to the new tool is promised. |

## BENCHMARK — regression detection

| # | Finding | Severity | Section | Recommendation |
| --- | --- | --- | --- | --- |
| B1 | No test exercises a successful applied download or its ZIP-member validation/admission path. Current tests cover dry-run and host refusal, not corrupted bytes, duplicate members, interrupted admission or redirects. | P2 | `tests/unit/test_data_vault.py`; `fetch` | Use tiny in-memory ZIP/response fixtures with real hash/admission logic. Assert no canonical admission on failure and deterministic retry behavior. |
| B2 | No test invokes `build_2020` or `ri_replay`; the profile test only checks six revisions and line-ending labels. Builder failure, wrong candidate hash, changed replay metrics/status and output reuse have no automated regression guard here. | P2 | `tests/unit/test_data_vault.py`; `build_2020`, `ri_replay` | Add small fixture-based tests with subprocess/engine boundaries controlled but actual admission and comparison logic exercised. Require non-success and no success receipt on mismatch. |
| B3 | The unit tests do not create and relocate a real junction/symlink. A dry-run path test and retained-directory test cannot prove the advertised link operation works on another platform. | P3 | `link_directory`, vault tests; `.github/workflows/test_pipeline.yml` | Add platform-aware link/relocation tests. Existing CI already runs unit tests on Windows and Ubuntu; extend those tests rather than claiming CI is absent. |

## TRENCH — failure modes

| # | Finding | Severity | Section | Recommendation |
| --- | --- | --- | --- | --- |
| T1 | `copy_checked` writes directly to the canonical destination. An interrupted copy leaves a partial file; a retry refuses the differing destination. The same pattern affects downloaded sources, extracted files and context admission. Integrity checks catch it, but ordinary retry cannot recover. | P2 | `scripts/data_vault.py:85-98` | Stage in a unique same-directory temporary file, verify, then publish using cross-platform no-clobber semantics. Do not use unconditional replacement. Fault-injection tests must preserve existing valid data and allow retry after interruption. |
| T2 | `link --apply` returns 0 even when mappings are missing or an old drive's links are retained. Changing `--vault` does not retarget existing junctions. Automated setup can therefore proceed while consumers still use old/unavailable data. The guide warns about retention but has no machine-enforced readiness gate. | P2 | `scripts/data_vault.py:157-181,310-326` | Add a strict readiness mode or nonzero incomplete-setup status. Offer an explicit, narrowly validated repair path for owned stale links; never replace real directories automatically. |
| T3 | Downloads stream without a byte ceiling, and builds have no wall-time/resource cap. A slow or unexpectedly large response can exhaust removable-drive space; the socket timeout is not an overall operation deadline. | P3 | `scripts/data_vault.py:122-125,284`; setup guide | Add size estimates/free-space preflight and bounded streaming/deadlines, with an explicit large-job override. Preserve the current disclosure until implemented. |

## LEDGER — interface contracts

| # | Finding | Severity | Section | Recommendation |
| --- | --- | --- | --- | --- |
| L1 | Action-inapplicable options are accepted silently. Confirmed: `build-2020 --year 2010 --state RI` dispatches the 2020 builder; likewise replay accepts State/year filters it does not use. This can select a different experiment from the user's request. | P2 | `scripts/data_vault.py:293-320` | Use subparsers/action-specific validation; reject conflicting or unused options before any I/O. Add CLI contract tests. |
| L2 | Catalog schema checking exists in `selected`, but build/replay read the catalog directly, and builder/local configuration schemas are not validated. Future version changes can be interpreted under old assumptions instead of rejected explicitly. | P3 | `selected`, `vault_root`, `build_2020`, `ri_replay` | Centralize versioned loaders with required-field, unique-ID/path, hash and State coverage validation. All entry points should use them. |
| L3 | Mixed valid/invalid State selections silently omit the invalid State. Confirmed: `--state RI TYPO` returns only RI entries. A misspelled intended batch can appear complete. | P2 | `scripts/data_vault.py:60-69` | Validate every requested State against the supported catalog and reject unknown values, even if some entries matched. Test mixed-validity selections. |

## SURVEY — operational adoption

| # | Finding | Severity | Section | Recommendation |
| --- | --- | --- | --- | --- |
| S1 | Existing 2000/2010 contexts can be verified, but fresh recovery without the drive is not implemented for those cycles or other research-track datasets. Thus the user's whole-project recovery ambition remains incomplete. | P3 | Setup guide, final scope paragraph | Publish a per-track recovery matrix and prioritize missing recipes. Keep this an explicitly bounded 2020 foundation until those paths are tested. |
| S2 | The approximately 715-MB tracked tree is not reduced by moving future runs to the vault. Partial clone still materializes the current tree, and the sparse-checkout advice is not a tested copy/paste profile. | P3 | Setup guide, repository size | Set a measured size budget and add manifest/hydration support for legacy large evidence, preserving its verifiers. Provide a tested lean-checkout recipe. No shared-history rewrite is required for current-tree slimming. |
| S3 | An empty data directory on the existing laptop is not a clean-machine install. Six top-level Python packages are pinned, but transitive/native dependencies are not fully locked, and the setup instructions install into the active Python environment without a dedicated venv. | P3 | `requirements-build-2020.txt`; setup guide, fresh build | Document an isolated environment, capture the actual resolved dependency/native-library inventory, and run a clean-environment acceptance test before broadening platform claims. Hash refusal currently limits incorrect admission, not setup failures. |

## Diagnostic probes

Temporary fixtures/mocks only; no production data changed and no network used.

1. Injected `OSError` after two bytes into an eight-byte `copy_checked` operation.
   The two-byte destination remained; a normal retry raised
   `existing destination differs; not overwritten` (T1).
2. Supplied a `missing-vault-directory` result at the link boundary and invoked
   `main` with `link --apply`: exit code 0 (T2; CLI exit behavior, not a real
   junction-creation test).
3. Invoked `main` with `build-2020 --year 2010 --state RI` and a mocked builder:
   the 2020 builder was called and the command returned 0 (L1; no build ran).
4. Called the real catalog selection with States `RI, TYPO`, year 2020,
   contexts group: it accepted just `context-2020-RI` (L3).

These probes confirm current behavior; they have not yet been installed as
permanent regression tests. All 47 existing tests still pass with these defects.

## Synthesis

Roles reviewed: 5. P1 blockers: 0. P2 issues: 7. P3 notes: 8.

Verdict: **NEEDS-WORK before releasing this as a dependable portable setup.**
The scoped RI proof and existing data integrity are not invalidated. No P1 was
identified in this bounded review; this is not a comprehensive security audit.

Top finding: T2 — a successful setup exit is not yet a reliable statement
that repository consumers are connected to the selected vault.

Cross-role consensus: TRENCH and LEDGER find fail-open operational contracts;
BENCHMARK finds the missing tests that let them pass. COVENANT and SURVEY agree
that byte-identical RI output is valuable evidence but not complete independent
environment/release custody. The 15 rows are distinct findings; shared concerns
are not counted again in this synthesis. P3 scope items are roadmap work, not
evidence that the documented narrow result failed.

## Three amendments, in priority order

1. **Make CLI completion trustworthy** (T2, L1, L3): validate action options and
   every State; surface incomplete links as a failed strict readiness check;
   test drive relocation without replacing user directories.
2. **Make admission retryable and test failures** (T1, B1, B2): verified staged
   publication with no-clobber semantics, fault injection, ZIP validation,
   builder/replay rejection tests. No partial file should masquerade as the
   canonical destination after interruption.
3. **Close execution custody** (C1, supported by C2/S3): bind catalog, raw inputs,
   wrapper/helper sources, command and resolved environment into the admission
   receipt; prove the recipe in an isolated environment. Then publish the
   reviewed code through the normal release process (C3).

Review-only changes: this report and TRENCH pitfall records. No implementation
fix, data migration, dependency installation, commit, push or PR action was
performed during this review.

## Remediation pass — 2026-09-10

The review above is preserved as the original assessment. The subsequent user
request was to fix all findings. This section records disposition rather than
retroactively treating the original code as approved.

| Findings | Disposition and evidence |
| --- | --- |
| C1, C2 | Implemented v2 admission receipt binding raw catalog rows, catalog/profile/helper/driver hashes, resolved command and State, actual Python/package/native inventory, thread controls, deadline and subordinate manifest/report/log hashes. A fresh venv produced the canonical RI context and a complete receipt. |
| C3 | Release is the final gate: the reviewed implementation must be committed/published and separately archived on M. See the release record for the actual revision; do not infer completion from the old bundle. |
| B1, B2 | Added real admission/validation tests around tiny download/ZIP/build/replay fixtures. Includes interruption, duplicate/corrupt/missing members, redirects, builder failure/hash mismatch/deadline, missing evidence and replay metric/status/hash mismatches. |
| B3 | Real link creation and relocation test passes on Windows, preserving both source trees and user directories. Added dedicated Windows/Ubuntu, Python 3.11/3.14 CI; cross-platform results depend on that run, not this local assertion. |
| T1 | Same-directory staging and verified no-clobber publication; injected interruption leaves no partial canonical target and retry passes. Racing publication preserves the other writer's file. |
| T2 | Incomplete link plans return 1. Explicit repair retargets only schema-recorded tool-owned links and refuses real/unowned directories. Link ownership is machine-local and ignored. |
| T3 | Streaming byte ceiling, socket/deadline checks, free-space preflight and subprocess build timeout implemented with explicit CLI overrides. No hard memory cap is claimed. |
| L1, L3 | Action subparsers reject inapplicable options before vault access; every requested State is validated, including mixed valid/invalid batches. Regression tests pass. |
| L2 | Central catalog/profile/local-schema loaders reject unsupported versions, missing/invalid fields, duplicate identities/paths, bad hashes and incomplete State coverage. Build/replay use the shared catalog loader. |
| S1 | Added the per-track recovery matrix and prioritized missing recipes as requested by the review. **Scope limitation remains:** this does not implement 2000/2010 fresh rebuilding or all other research tracks. Whole-project drive-loss recovery is not declared complete. |
| S2 | Externalized the two largest traces (about 85 MB); both vault and pinned-Git recovery reproduce original hashes. Added hydration tests, lean-checkout commands and a 650-MB indexed-tree budget. Frozen manifests/verifiers are unchanged. |
| S3 | Created a fresh isolated Windows/Python 3.14 environment, installed dependencies, captured a full resolved lock/native inventory, rebuilt RI to the exact canonical hash and replayed all six arms. Virtual environments are documented as per-machine, not portable across drive letters/OSes. |

Local regression result: 63 tests passed (28 vault/evidence tests plus 35
existing county/RI tests). Original review diagnostics are now represented in
permanent regression coverage. PP-19/20/21 track the structural remedies.

Additional discovery during evidence hydration: the frozen ensemble's analysis
JSON and CSV expected CRLF bytes, while broad checkout attributes forced LF.
Narrow attributes now restore the already-recorded bytes without editing the
manifest or logical data. The old full verifier separately reports source drift
in `block_trace.rs`; this pre-existing historical source mismatch is not fixed
by moving evidence and is not represented as a passing full historical replay.

Current interpretation: the implementation defects have remedies and local
tests; release/CI/lean-clone acceptance is recorded separately. The broader
multi-cycle/data-track recovery ambition remains explicitly bounded by S1.

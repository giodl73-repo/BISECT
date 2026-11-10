---
skill: roles-check
topic: wasm-selected-input-workspace
date: 2026-10-23
depth: standard
roles_used: [meridian, benchmark, covenant, ledger, vault, trench, survey]
p1_count: 0
p2_count: 3
p2_open_count: 0
p3_count: 18
p3_open_count: 18
verdict: APPROVED-WITH-CONDITIONS
---

# Selected input workspace: implementation review

Artifact: [workspace contract](../../../docs/specs/wasm-selected-input-workspace.md),
its linked source/UI/tests and local package. One agent applied seven installed
role lenses; these are not independent reviewers or human signatories. This is
scoped implementation review, not full desktop-parity or release approval.

## Selection

Selection and priority follow [.roles/ROLE.md](../../../.roles/ROLE.md).
Selected profiles and verify questions were read in this review campaign.

| Role | Why selected |
|---|---|
| [MERIDIAN](../../../.roles/meridian.md) | Preserved graph, source order, seed, assignments and backend. |
| [BENCHMARK](../../../.roles/benchmark.md) | Independent references, source-owned fixtures and executable failure tests. |
| [COVENANT](../../../.roles/covenant.md) | Retained original bytes and source/runtime/package evidence. |
| [LEDGER](../../../.roles/ledger.md) | Optional project field, ZIP contract and native output interchange. |
| [VAULT](../../../.roles/vault.md) | Hostile archives, selected-root authority and inert Open. |
| [TRENCH](../../../.roles/trench.md) | Stale asynchronous results, revocation and interruption recovery. |
| [SURVEY](../../../.roles/survey.md) | A complete source-to-result-to-output practitioner journey. |

CONTOUR/SCALE/WARD requirements remain on broader real-data/statistical/policy
claims. BOUNDARY/PRECINCT/DATUM/COMMONS/CANVASS/TALLY need targeted review when
their legal, political, publication, community, certification or election-data
contracts change. This slice changes none of those claims. Role prose is not
legal authority; no legal assessment or election certification is supplied.

## Findings

Closed-scoped means the implemented guard and cited regression pass for this
contract. P3 entries remain implementation/release obligations, including
carried-forward plan conditions; they are not necessarily newly found defects.

| ID | Lens | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|---|
| M1 | MERIDIAN | A new input selection must not rewrite graph order, max-u64 seed, options or assignment labels. | P3 | Source-to-Run | Open broader coverage: three source modes match exact assignments/plan/context on the RI fixture; add other methods/resolutions/vintages before extending that claim. |
| M2 | MERIDIAN | The independent native reader's legacy graph JSON lacks the explicit browser `unit_kind` field. | P3 | Comparison contract | Open broader schema reconciliation: retain both raw JSON texts, assert browser `tract` metadata explicitly and compare all other fields exactly. Do not call the graph texts byte-identical. |
| M3 | MERIDIAN | Directory/ZIP support does not establish C backend selection or every native build's semantics. | P3 | Backend scope | Open: keep this fixture metis-core and the separate optional C browser regression; retain existing native build-feature limitations. |
| B1 | BENCHMARK | Four-unit fixtures and full host-reply equality do not demonstrate native CLI filesystem Run orchestration. | P3 | Native reference | Open: independent file-reader/path cases support those scopes; add full CLI discovery/run/output orchestration before claiming it. |
| B2 | BENCHMARK | Source-owned fixtures remove a retained-data prerequisite, but national/large-state capacity still lacks corresponding evidence. | P3 | Matrix | Open: predeclare real-data rows and record successful, matching-failure and not-run outcomes separately. |
| B3 | BENCHMARK | Local integration receipts are not formal acceptance-v1 bundles with every required source/toolchain/raw-output group. | P3 | Acceptance | Open: migrate evidence without promoting partial rows. Standard build scripts now invoke the new source-generated gates. |
| C1 | COVENANT | Original ZIP/container bytes must remain distinct from derived per-file/job hashes. | P3 | Source custody | Open broader custody: retain original ZIP bytes/hash and exact selected member sources; no source authenticity follows. |
| C2 | COVENANT | Reusing a validated engine is a different claim from rebuilding every artifact from a clean source snapshot. | P3 | Preview build | Open: identify inherited engine bytes and base package, current browser modules and final offline manifest; preserve the wider reproducibility gate. |
| C3 | COVENANT | Successful installation/download cannot authenticate the historical accepted Run or certify an election. | P3 | Output evidence | Open ongoing boundary: retain explicit preparation/execution identities and existing unverified-import/export rules. Complete composed Verify/replay separately. |
| L1 | LEDGER | Dropping the original archive or restoring live selection on Open would break the source-preserving save/load boundary. | P2 | Project/Open | Closed-scoped: optional bounded `nativeWorkspaceArchive` preserves original bytes; Open validates encoding without inflation/selection. Explicit Load saved ZIP then inspection/Run is tested. Old projects keep their meaning. |
| L2 | LEDGER | ZIP32/member/path limits differ from unrestricted desktop folders and from downstream project/job limits. | P3 | Supported scope | Open wider compatibility: document stored/deflated, one root, 64 MiB compressed/expanded, 10,000 entries and 32 levels; do not equate import bounds with successful Run/save capacity. |
| L3 | LEDGER | Native-layout byte agreement is not validation of every external GIS/practitioner schema. | P3 | Interoperability | Open: retain named external-version fixtures before claiming those ecosystems; this slice adds no external schema promise. |
| V1 | VAULT | Archive paths, links, aliases or forged extents could bypass the intended selected-source boundary. | P2 | ZIP worker | Closed-scoped: private input snapshot, bounded worker, path/component aliases, type/extent/header/descriptor/CRC checks; 18 independently generated/mutated hostile cases refuse. Contents stay in memory and are never executed/extracted to disk. |
| V2 | VAULT | OPFS and an injected permission denial do not test a native OS picker or actual OS permission revocation. | P3 | Browser evidence | Open: label the test precisely and add declared browser/OS permission journeys before expanding that claim. |
| V3 | VAULT | Every storage/memory/cancellation point is not covered by one interrupted body write. | P3 | Failure matrix | Open: retain bounded import and current recovery witnesses; extend quota/exhaustion/host-policy cases separately. |
| T1 | TRENCH | A live source mutation during queueing, or late picker after Open, can otherwise authorize stale work. | P2 | Async transitions | Closed-scoped: permission/metadata checks before referenced reads and after queue staging, private selection authority, revision/generation checks and retirement. Browser tests mutate before/during queueing and complete a picker after project Open. |
| T2 | TRENCH | Queued retained bytes are snapshots; they do not automatically track subsequent filesystem edits. | P3 | Workspace contract | Open wider lifecycle: disclose the snapshot boundary and require reselection/reinspection for new inputs; no implicit watcher or OS-wide discovery claim. |
| T3 | TRENCH | A passing scoped journey can be mistaken for closing the whole workspace or parity parent. | P3 | PP-25 / Ledger | Open: keep parents/required contracts partial and full-parity rejection active. Existing PP-25 covers false executable-parity closure; no duplicate pitfall is invented. |
| S1 | SURVEY | Input selection, inspection, queueing, Run, output preparation and installation remain separate explicit actions. | P3 | First-use UX | Open practitioner validation: current help explains the sequence; test with actual users before claiming novice usability across workflows. |
| S2 | SURVEY | The portable path must retain accepted output after a local write fails or is cancelled. | P3 | Recovery | Open broader failures: tested cancellation keeps download available and explicit review/resume writes exact files without another partition; other lifecycle scopes remain. |
| S3 | SURVEY | One state/year journey does not supply national mode, reset/force or acquisition-queue recovery equivalence. | P3 | WP-03/05/08 | Open: keep those ranked contracts separate and preserve the overall plan scope. |

## Synthesis and amendments

Seven lenses; **21 findings: 0 P1, 3 P2 closed for the tested scope, 18 P3 open**.
Verdict: **APPROVED-WITH-CONDITIONS for continued implementation/local preview**.
It is not full workspace, all-state or standard-release approval.

Top finding: a selected live root is authority with a lifetime, not a recorded
path string. VAULT/TRENCH/COVENANT agree that Open, changed inputs and stale
asynchronous completion must not restore or extend it. BENCHMARK/MERIDIAN require
exact output comparisons with the independent-reference scope stated honestly.

Three amendments implemented and tested:

1. Added a common private source adapter for uploads/live roots/ZIPs, with strict
   bounded archive checks and no executable extraction paths.
2. Retained original ZIP bytes as inert optional project data, with explicit
   reload and fresh inspection; reused existing output/recovery rather than
   inventing another installer or project authority.
3. Added pre-read/post-staging source checks and generation retirement, with
   actual worker/OPFS mutation, cancellation, recovery and late-picker tests.

## Evidence and reviewed sources

The source-owned module case passes three input modes, 26 native host replies,
independent native graph field comparisons, identical accepted outputs/maps,
exact installed/read-back bytes, cancellation/recovery and 18 ZIP refusals.
The workbench test exercises portable ZIP and genuine OPFS handles; its injected
picker/permission denial is disclosed. Existing folder discovery passes 144
actual CLI path cases/171 host replies/seven refusals; its existing browser
upload/three-state workflow also passes. Existing single recovery passes eleven
scenarios/31 native guard replies with no recovery partitions.

Local package/evidence identities are recorded by
`target/native-workspace-source/package-evidence.json`; the build retains the
base and runs final packaged workspace, offline/asset and optional C browser
checks. These remain local integration inputs, not acceptance-v1 bundles.
Source declarations/direct-read and eight ledger/four consumption mutation tests
pass. No global completion percentage is inferred.

| Reviewed source | SHA-256 |
|---|---|
| `web/lab/native-workspace-source.js` | `e415f5c7dff59ed036ca3531b435e7d23e8d33846f3a0a2f6928645666b212f2` |
| `web/lab/native-workspace-archive-worker.js` | `7c72c6a95ff11b44bdad6b20b691390743bf574f4cb6f6e2d0d54dd291196259` |
| `web/lab/project.js` | `0a47a66732fc7e9ef7d77fa6c5bc3ebcdb14d4063a16843bf2a10ad2069ea56a` |
| `web/lab/workbench.js` | `190001d615c6e6c0c0f39c9fbf8fb42301a2bac208849bbe64d142646beb08a0` |
| `web/lab/workbench.html` | `f79cf5c8a538ed4b543c545a9cd91256b86316ba6b81109cf2c17e28bc535d2d` |
| `scripts/web/test_native_workspace_source.mjs` | `151897adbe7b9a75db3318722e6797439d24fe14350cd91b7f8f98a6cf2042a1` |
| `scripts/web/test_native_workspace_browser.mjs` | `52bec978ec3e2acc9861a9c02023db019e9c5ae0fb6a7029f27458c53cbbf526` |

HEAD remains `383db340235e21d752a35610aab28e2948b73e57`, with a dirty worktree.
Hashes identify reviewed bytes, not authorship or independent reproduction.
Nothing was committed, published or deployed by this slice.

The distinct selected-root lifetime failure is recorded as
[PP-27](../../../design/pitfalls/pitfalls-pipeline.md#pp-27-selected-root-observations-outlive-their-live-authority),
MITIGATED with the actual mutation/late-picker/recovery regressions. It remains
bounded by the non-atomic filesystem and untested OS permission behaviors above.

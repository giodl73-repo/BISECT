---
skill: roles-check
topic: wasm-cffi-engine-integration
date: 2026-10-22
depth: standard
roles_used: [meridian, benchmark, covenant, ledger, vault, trench, survey]
p1_count: 0
p2_count: 5
p2_open_count: 0
followup_date: 2026-10-23
p3_count: 16
p3_open_count: 16
verdict: APPROVED-WITH-CONDITIONS
---

# Optional C METIS integration: repository role review

Artifact: [integration contract](../../../docs/specs/wasm-cffi-engine-integration.md),
its linked Rust/C/JavaScript implementation and the scoped local package.
Type: implementation and experimental browser workflow. This is one agent
applying seven installed role lenses, not independent reviewers or human
signatories. The profiles and their verify questions were read during this
review campaign; selection and ranking follow [ROLE.md](../../../.roles/ROLE.md).
No legal standards are interpreted or certified by these technical lenses.

## Selection

| Role | Why selected |
|---|---|
| [MERIDIAN](../../../.roles/meridian.md) | Actual backend, seeded assignments and native option/weight semantics. |
| [BENCHMARK](../../../.roles/benchmark.md) | Independent reference, exact errors, fixture provenance and old-suite assumptions. |
| [COVENANT](../../../.roles/covenant.md) | Multiple executable assets and retained runtime identities. |
| [LEDGER](../../../.roles/ledger.md) | ABI, numeric encodings and existing saved-format compatibility. |
| [VAULT](../../../.roles/vault.md) | Memory ranges, imported authority and verified code-loading boundaries. |
| [TRENCH](../../../.roles/trench.md) | Stale initialization, request isolation and structural failure prevention. |
| [SURVEY](../../../.roles/survey.md) | Practical workbench exposure, save/reopen behavior and support wording. |

All sixteen role profiles were considered in the preceding plan review.
CONTOUR/SCALE/WARD conditions remain on the broader plan; these synthetic
fixtures do not establish real source, statistical or jurisdictional claims.
BOUNDARY/PRECINCT/DATUM/COMMONS/CANVASS/TALLY need targeted review if later work
changes their legal, research, community, certification or election-data domains.

## Findings and dispositions

P2 must close before the corresponding release claim; P3 is remaining acceptance
work. Closed-implementation findings apply only to the tested scope. Some open
findings carry forward earlier plan conditions; this is not a count of new bugs.

| ID | Lens | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|---|
| M1 | MERIDIAN | Compiling C support must not route existing default Rust requests into C. | P2 | Backend dispatch | Closed—scoped guard defaults false, and frozen whole-result equality passes before C, after successful C and after failed C. Native builds retain their existing feature selection. |
| M2 | MERIDIAN | A Windows-compatible RNG does not establish every libc/compiler/build profile, and native recursive/N-way dispatch remains build-feature dependent. | P3 | Runtime/build profile | Open—keep msvcrt-v1 explicit, retain the unmodified mismatch and distinguish C-feature/portable native references. Reconcile native per-request backend semantics before claiming one-binary equivalence. |
| M3 | MERIDIAN | NRS recursive mode, other searches/weights, multiscale and realistic states exceed the current 52-success matrix. | P3 | Method coverage | Open—predeclare and run those rows; do not infer them from three connected C consumers. |
| B1 | BENCHMARK | Older tests explicitly expect C-unavailable preparation, which conflicts with the optional feature's new capability. | P2 | Build gates | Closed for the scoped optional preview—feature-aware suites now run on both actual builds in its packaging path, followed by final packaged native/browser gates. See the 2026-10-23 follow-up. Standard release/full parity and broader conditions remain open. |
| B2 | BENCHMARK | A State-parser/compositor/kernel oracle is stronger than the browser adapter but does not execute the complete CLI filesystem workflow. | P3 | Native reference | Open—exercise actual CLI source discovery/output/retry orchestration separately. Native arguments, JSON and raw stdout/stderr are retained for current fixtures. |
| B3 | BENCHMARK | Synthetic 64/300-node cases and source-owned frozen output do not prove actual all-state capacity. | P3 | WP-08 | Open—retain the clean fixture and optional previous-WASM replay; add immutable real-data rows rather than extrapolating from 50 packaged graph packs. |
| C1 | COVENANT | Adding C identities to the old two-field WASM record would break its existing meaning and validators. | P2 | Runtime identity | Closed—main identity stays unchanged; C execution adds a separate runtime-v2 packet. Seven native/WASM receipts and configured-runtime-v1 source ZIP tests pass. |
| C2 | COVENANT | Retaining a runtime packet in a saved result does not complete Verify/replay or failed-run custody. | P3 | WP-06 | Open—compose both C asset identities through remaining verification, replay and failure workflows; no authenticity follows from matching bytes. |
| C3 | COVENANT | Local evidence and package hashes are not a public obtainable release or formal acceptance-v1 bundle. | P3 | WP-10 | Open—complete required bundle/build/release identities, packaged third-party notices and maintainer acceptance before publication. No deployment occurred here. |
| L1 | LEDGER | The bridge covers BISECT's used Graph subset, not every metis 0.2 API/option or native ABI width. | P3 | Cross-memory ABI | Open—keep idx32/real32 and C compile-time option-layout checks; expand only through explicit contract/version changes. |
| L2 | LEDGER | Versioned runtime packet retention does not prove all external exports or old readers accept new C provenance. | P3 | Interoperability | Open—exercise exact named schema/version round trips in each advertised ecosystem. Existing raw/native hash algorithms are not replaced. |
| L3 | LEDGER | Transfer/download limits and k≤vertices are browser scope bounds rather than universal native limits. | P3 | Supported input scope | Open—publish those bounds and distinguish rejection/resource observations from native algorithm failure and tested capacity. |
| V1 | VAULT | Unvalidated pointers or receipt-created module authority could bypass the browser boundary. | P2 | Loading and memory | Closed—private loaded authority, verified snapshots, Rust/JS dimension/range checks, separate C memory and output validation; copied authority and bad/overlapping ranges are refused. Open/selection execute nothing. |
| V2 | VAULT | Fatal C traps, allocation exhaustion and all cancellation points have not been injected in the actual module. | P3 | Failure lifecycle | Open—unexpected traps retire the instance, while normal errors recover. Add real fault injection; do not claim every recovery path proved. |
| V3 | VAULT | Chrome worker success does not prove every browser/host policy permits the verified blob module. | P3 | Browser/hosting matrix | Open—test declared worker/CSP/storage/browser combinations before publishing support. This review does not change hosting headers or deploy code. |
| T1 | TRENCH | Older initialization can complete after newer initialization and replace its engine identity. | P2 | Worker state | Closed—generation token discards stale ready/error outcomes. Real browser test completes both actual instantiations in reverse order and observes one ready reply. |
| T2 | TRENCH | A package containing C files is not proof that every advertised workflow executes with them. | P3 | PP-25 | Open—keep parent evidence partial and formal release closure blocked; the source/ledger guard still rejects full parity. |
| T3 | TRENCH | Same source/seed can conceal a libc RNG mismatch, including label permutations. | P3 | PP-26 | Open broader condition—pitfall is MITIGATED with named profile/sequence/assignment witnesses, not universally SOLVED. |
| S1 | SURVEY | The runtime-load control and request's backend choice are distinct actions; first-use clarity still needs wider practitioner validation. | P3 | Workbench | Open—current help explains the distinction and missing runtime produces a loading error before partitioning. Demonstrate the complete novice journey before broader exposure. |
| S2 | SURVEY | The extension builds from an existing normal laboratory base and does not rerun its entire support matrix. | P3 | Packaging | Open—retain the experimental label and normal Rust default; finish standard build-gate integration rather than treating the wrapper as full release approval. |
| S3 | SURVEY | Native failure and out-of-tolerance candidates must not become successful support counts. | P3 | Support statement | Open ongoing gate—report 52 assignment successes and four matching native failures separately; keep underlying candidate checks and matrix outcomes visible. |

## Synthesis (initial review)

At the initial review: seven lenses, **21 findings**: **0 P1**, **5 P2** (four
closed in the scoped implementation, one release condition open), **16 P3** open
acceptance conditions. The follow-up below closes the scoped B1 condition;
the sixteen broader P3 obligations stay open.

Verdict: **APPROVED-WITH-CONDITIONS for continued implementation and local
experimental review.** It is not standard-release or full-parity approval.
At that point the top remaining issue was feature-aware legacy gates and
broader declared workflow/method evidence before its support claim expands.

MERIDIAN/BENCHMARK/SURVEY agree on scoped executable claims. COVENANT/LEDGER
require identity format preservation. VAULT/TRENCH require explicit authority
and stale-result rejection, beyond a successful C call.

## Three amendments applied

1. Preserved default Rust dispatch and old main-WASM identity; added named C
   selection and a separate versioned runtime packet with native/error witnesses.
2. Added verified snapshot loading, bounded memory transfer, retired trapped
   instances and generation-bound initialization; browser tests exercise inert
   Open, missing-runtime failure, explicit execution and initialization races.
3. Added an experimental build/stage/offline path, accurate support documentation
   and a partial ledger contract. PP-26 is MITIGATED; PP-25/full release stay open.

## Evidence inspected

- Source-owned C matrix: 52 native assignment/weighted-boundary matches, four
  exact native failure messages; State preparation and YAML route checks;
  asset/profile/ABI refusals; frozen default/post-C whole-result witnesses.
- Actual packaged workbench worker: native assignments, lazy three-asset load,
  missing-runtime distinction, retained C hashes, inert Save/Open, retired exports,
  two real out-of-order initializations, 390px width and zero page errors.
- Broader existing Rust path: 56 exact-seed comparisons/four matching failures;
  20 whole N-way controls plus 20 independent native kernel matches; 301 prepared
  cases/312 native comparisons, 296 exports and eight inert reviews. The prepared
  test now compares six exact native failure messages and distinguishes the
  runtime field's Rust-backend refusal from positive C coverage.
- Rust bridge tests: three grouped validation/selection regressions. Inventory
  guards: eight closure and four native-consumption tests. Main/runtime-v1
  identity checks passed. Package: 1,030 local dependency references, 625 exact
  hashes/sizes and 525 shell assets. Fifty state/year packs are packaged data,
  not fifty new C execution proofs.

Local evidence is under `target/cffi-integration/`, `target/prepared-engine/`
and the respective regression directories. The scoped package is
`dist/wasm-cffi-validated`, offline version
`58e56c22b298` (full digest in its manifest). These generated artifacts are not
clean-checkout prerequisites or formal acceptance-v1 bundles. Source/build
identities for the reviewed implementation are appended below.

## Reviewed source identities (initial review)

HEAD `383db340235e21d752a35610aab28e2948b73e57`, dirty tree. Current package version: `58e56c22b29894f2e60286179f172dd3294ba0ab734bcb129ef10c06f8400d65`. Main WASM: `0cb66c3d20fe579c2f32a623d0a11fb7e770c90425f274f533b8a77195fe7246`.

| Source | SHA-256 |
|---|---|
| `Cargo.toml` | `41f495cddeecacbb5e1c3ca1a9a503a1a2972e13608a80ba0381fe333dc5578d` |
| `Cargo.lock` | `9885526258c36673b0097b9a9b978ca2d604f789eae483b92db93000896c14bf` |
| `crates/bisect-metis-sidecar/Cargo.toml` | `272c20084276c19fd151ba72f86f91129c1148c187103d2d4e7680b56d847d2e` |
| `crates/bisect-metis-sidecar/src/lib.rs` | `983ae6ffac742372adeb918f3639e38b33cd123ff43e2d905a30f7cda674f63d` |
| `crates/bisect-metis-sidecar/c/bridge.c` | `73af3c8ae89c3866c36b8a5bc65a216a2c94310b1496ac2de59e5e4034b47820` |
| `crates/bisect-metis-sidecar/c/msvcrt_rng.c` | `48860f6503fd70dbc5baa028898df1c38a04745b0af688ca0755ddafa73976a7` |
| `crates/bisect-apportion/Cargo.toml` | `ad1e0fa11e3d165e64cc9361e85b4bb7008023ba8daf91a4ec44723173b465d5` |
| `crates/bisect-apportion/src/lib.rs` | `7447a63fc82168438d12393f52ae87c6195eca34d569ae7e933340932efb329b` |
| `crates/bisect-apportion/src/split.rs` | `968881a5a208efaa424bf697ec0773a21dc3af26e3d753c7af100b108cc026db` |
| `crates/bisect-runner/Cargo.toml` | `14d6afdb3f2279e51cb750f33e71cddb7e66a0eef894a49a8a567b91331b1015` |
| `crates/bisect-runner/src/bisection_runner/mod.rs` | `6039ac57960b8d77d72582cfc3293d3e9549a72ceee82fdbb31dca1501af3ac2` |
| `crates/bisect-runner/src/bisection_runner/core.rs` | `64560883c11f7a09b534d510594225a374538e425874e9bded5220ff29af6564` |
| `crates/bisect-runner/src/bisection_runner/nway_geo.rs` | `0499007b0aa9f75b6447f9dea7c8677935065db290557d4132973e368f05b549` |
| `crates/bisect-wasm/Cargo.toml` | `569c4ab2158c794abc56ad27e2800888d891b489d3735dfcfc0e9d7db27be171` |
| `crates/bisect-wasm/src/lib.rs` | `5a1aae6cb24d2b7208e5b62d6b7111f27cf5011d046ff66a748f27fabb5bfd11` |
| `crates/bisect-wasm/src/metis_backend.rs` | `5421460947837c871a1508c792aec26bbea42b069e448d801660cb76330a322f` |
| `crates/bisect-wasm/src/engine.rs` | `4731593be46eecd417d5340bc916aff6713b72c51fe60332e2a376bd5e3b02a2` |
| `crates/bisect-wasm/src/native_state_run.rs` | `f31b4ce1205f71cff79c00a50e7cad86e20474a1fa7179a52e1bc04db41b4b25` |
| `crates/bisect-wasm/src/config.rs` | `30db9a20b62ff63b41f1fc40c3ac76c321c911bc76ed5443157f972efde1e02d` |
| `crates/bisect-wasm/src/multiscale_input.rs` | `96b016e5417204062f60bc52e43cb3dc864267f1e3b3c89586ef2dcd3d5a9f0a` |
| `crates/bisect-cli/examples/state_signal_preset_reference.rs` | `c3a9312d8d6ded072010cf539e70d07f375cc8e02ced660807de0b91ca37430d` |
| `crates/bisect-wasm/examples/execute_request.rs` | `9a32b7cff76df63e757f8df7059f5b988b1d356afebd91416b9a8581352d1941` |
| `web/lab/metis-c-sidecar.js` | `ad1ef4c7083ed916fd9def1ef04ccc4f8e3e720aa8e39985abe68b5eb61081b3` |
| `web/lab/wasm-engine.js` | `7bbbd1eb8ff68c39d06dd2c5ee631f4474f1d1ef1c6249e44398e24a2466a90e` |
| `web/lab/wasm-worker.js` | `bfe4f339a428916ea5929728784605fcfe267af0d728b1918847632523b760a4` |
| `web/lab/workbench.html` | `2caa4c1c4d1026d7967bfc6560fecc8459664e435f55e53217502b12d0cb625f` |
| `web/lab/workbench.js` | `7f4d759427419eba91d6a553a240df4888ed691d44fdcecc2da0c98a7708a7cc` |
| `scripts/web/build_metis_sidecar.py` | `73cd29ea932a3756a0243d12329aac4ed66e6c801c740877dbf808a410f81b52` |
| `scripts/web/stage_metis_sidecar.py` | `21eba681bcf563f36f670220cca58cfb9a5f953f97cb7d4002821998d63b66f5` |
| `scripts/web/build_cffi_preview.ps1` | `b773851dcdfa96b2b855993d8eafee80b0390dce89c440e6cb9ed862b45fa607` |
| `scripts/web/build_offline_manifest.py` | `6288897e18f771a966ce5edfe2b8e4bdbd3a983b7dd6680b9dda34f71d890887` |
| `scripts/web/test_cffi_engine.mjs` | `44fee7cb7db8c3626822f1199e5ea2b8b4daa89235a128659ce6e1442a23930d` |
| `scripts/web/test_cffi_engine_browser.mjs` | `b59bed4d2ebba549d22ccba87958dbd4737d14e58fb0ba0a81c99ddbeeb6f711` |
| `scripts/web/fixtures/cffi-engine-case.mjs` | `bef19e1a9cc604db09f2609d37e0648efd8eb5339bffb1a82791bbcba6f0afeb` |
| `scripts/web/fixtures/cffi-core-baseline.json` | `4d84e99ee76241912c0bdb75d1b4a3d8c6974e6896189bcc7011a0bb60ed20c3` |
| `scripts/web/test_prepared_engine.mjs` | `20030a6d81395d4d70cf77423c687f6f0a3ff98ff995037ab628c21abfe010d6` |
| `scripts/web/test_runtime_artifact_wasm.mjs` | `55154455b6f372dbff9d48273f866daa4ed5860c775d20b1a842162e316f3bbc` |
| `docs/specs/wasm-cffi-engine-integration.md` | `257f484ef89800002c91fde37d4ee4a8caf079fef87ab198cf3b357bc4b836ae` |

## B1 follow-up, 2026-10-23

BENCHMARK's feature-aware legacy-gate condition is now closed for the optional
preview wrapper. This is an evidence follow-up to the original finding, not a
new independent seven-reviewer approval. MERIDIAN's default/assignment contract,
COVENANT's artifact identity requirement and TRENCH's stale-evidence concern
were applied to this closure. All sixteen P3 conditions remain open, including
clean fixture portability, wider workflows/methods, fault injection, formal
runtime custody, third-party notices and standard release acceptance.

Command completed successfully:

```powershell
./scripts/web/build_cffi_preview.ps1 -Base dist/wasm-lab-build -Output dist/wasm-cffi-gated -BrowserChecks
```

- Both actual builds passed the seven configuration/State suites. Availability
  differences use the narrow named capability contract, retaining original
  replies and exact source/seed/YAML comparisons. Mutations to YAML, seed,
  availability, backend and extra fields are rejected. Missing C runtime is
  recorded as such, not a matching native algorithm failure.
- The wrapper builds and preserves its portable native oracle before rebuilding
  the native C oracle, then binds the staged WASM to the completed C-feature gate.
  Each requested variant starts not-run; only a completed suite set emits pass.
- Three Rust bridge tests pass. Both the build output and final packaged modules/
  engine pass 52 native assignments/weighted-boundary comparisons and four exact
  native failures, plus default/post-C whole-result equality and asset/profile/
  pointer refusal checks.
- The final package passes 1,030 local dependency references, 625 offline asset
  hashes/sizes, and actual Chrome 155.0.8059.85 workbench/worker execution. Browser
  results bind the loaded main/C identities to package bytes and cover inert
  Open/selection, missing-runtime refusal, Save/Open, retired exports, 390px layout
  and two real initializations completed in reverse order with one ready result.

Fresh package: `dist/wasm-cffi-gated`. Its WASM is
`0cb66c3d20fe579c2f32a623d0a11fb7e770c90425f274f533b8a77195fe7246`;
default build is
`a1243eb799b6f929e27096ea1781a17456200b00e16073edb4ff1a561f719abb`.
Offline version:
`58e56c22b29894f2e60286179f172dd3294ba0ab734bcb129ef10c06f8400d65`.
The packaged app bytes equal the previous validated package; the new result is
integrated regression evidence. No feature/capacity expansion is inferred.

Local receipt identities (not committed acceptance-v1 bundles):

| Evidence | SHA-256 |
|---|---|
| `target/cffi-integration/package-evidence.json` | `e292a633e8689d050bd25249a3b9ee2e89c7a3ca62dbf550ac9445a50bdbdf8c` |
| `target/cffi-integration/engine-evidence.json` | `0680f178b88b4937bd7e1dc3cda95ffc42c2ee47a4cea2f4e5c5b7a95ae5bb1f` |
| `target/cffi-integration/browser/evidence.json` | `83645141c6b236482334037659f5f41eab893acf70aea21b4099360aaaf92fd9` |

The package receipt binds both fresh legacy receipts, their source/test hashes
and the exact offline manifest. Native/browser loaded runtime identities agree.
The native C oracle for this run is
`6d7695d939d27ebed4c21462947ed6ec5d22a92ef22a8df201143148112c084e`;
it is retained separately from the portable reference. Local integration data
prerequisites still need the plan's clean-checkout/immutable-fixture work.

Changed gate sources reviewed for this follow-up:

| Source | SHA-256 |
|---|---|
| `scripts/web/build_cffi_preview.ps1` | `6eeb53d38c49770e3a966700640dd63d4e447038fd0ecb46394d7e28120c41b1` |
| `scripts/web/check_cffi_legacy_gates.ps1` | `e76123513732438e6563bb2a14270344ec488594c2f0d7c90897dc0cc20e0ac2` |
| `scripts/web/test_cffi_engine.mjs` | `41964de206e954eac341c9b7317da98f2c977137f92d80260976fa173cb58ad1` |
| `scripts/web/test_cffi_engine_browser.mjs` | `566c4bf50026d598e816908d1411c3a881d0a284cf218c4434683833dfa3786f` |
| `scripts/web/fixtures/config-capability-comparison.mjs` | `35da09ec309d55969f18524aeb1c855ca21dc9848b46e3ab2ec1863e2a2c909c` |

The per-build receipts pin the six configuration scripts and State script as
well. The original source table above remains historical rather than silently
being rewritten to current bytes. Updated ledger child records stay partial;
this closure does not waive any required parent/full-parity contract. Nothing
was committed or deployed.

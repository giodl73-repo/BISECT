# Resolved native State replay recipe

Status: the local workbench supports explicit selected-file replay. The
maintained Rhode Island 2020 tract regression compares actual `state` and
`replay-native-state` output against both workbench builds using `metis-core`.
It verifies literal assignments, balance and connectivity, original input bytes,
inert Open, changed-source refusal and fresh export. The fixture retains the
derived graph from original TIGER and PL94 data. It exercises replay after graph
preparation. Broader methods, sources, capacity, hosting and WP-01–WP-10
acceptance remain open.

Scoped Rhode Island follow-ups on the same 250-tract fixture reproduced
GeoSection and eight-seed AreaSection in the fresh standard toolkit and in
a C-enabled toolkit using the Rust backend. AreaSection used the original
TIGER geometry and rejected a changed source before execution. A separate
ratio-optimal run used the actual C backend and matched the recorded native
C assignments in Chrome; the Rust-only toolkit refused the C recipe before
execution. The native C AreaSection preset clears the supplied seed. With
the resolved unset seed and eight candidates per ratio, both native State
and the browser's prepared C run reported no feasible root split and
offered no plan. The browser control with an explicit seed of 42 found a
plan; it exercises a different resolved request. These checks use a
retained C build while the fresh C build is pending. Evidence:
`target/parity-full-build-20261106-r25/ri-ratio-recorded-replay-independent-readback.json`,
`target/parity-full-build-20261106-r25/scoped-ri-c-ratio-independent-readback.json`,
and `target/parity-full-build-20261106-r25/scoped-c-area-portable-failure/browser-evidence.json`.

Run `scripts/web/test_native_ri_replay_cli.py` with `--cli` and `--output`.
It prints the native case directory. Run
`scripts/web/test_native_ri_replay_browser.mjs` with `--native-case` set to
that directory, `--site` set to the assembled toolkit, `--profile core` or
`--profile c` to identify the build, and `--output` set to a separate directory.
The historical isolated evidence remains under
`target/native-verify-parameters-20261102/constraint-preflight-phase/`.

Native Verify previously omitted the seed/tolerance it printed, and manifests did
not retain the original general search configuration. Recording configured engine
alone was also insufficient: the native shared recursive dispatcher selects C
when compiled with `c-ffi-engine`, even when configured METIS engine says Rust.
The correction records resolved settings and effective backend separately,
preserving native dispatch rather than changing historical algorithms.

## Versioned wire and bindings

First-attempt non-multiscale producers retain `bisect-native-state-replay-recipe-v2` in the optional
`native_replay_recipe` manifest field. V2 records resolved algorithm/search,
decimal-text u64 seed, exact fractional tolerance, chamber/district/electoral and
resolution settings, loaded graph digest and observed source identities. Its
final member, `recursive_metis_backend`, is mandatory and is either `metis-core`
or `c-ffi`. Despite its historical name, it records the fixed native shared
dispatcher used by both recursive splits and NWay. Actual C-enabled NWay uses C
regardless of the configured engine; the recorded effective backend governs
replay. PFR and methods outside that dispatcher retain their separate semantics.
The original configured-engine interpretation of NWay is contradicted by retained
actual native assignment differences; original bytes are not rewritten.
All-method execution coverage remains pending.

The typed JSON digest uses UTF-8 serde serialization in fixed struct/enum field
order. Top-level order is schema version, resolved state, original working
directory, adjacency path, graph digest, source identities, binding scope and,
for v2, effective recursive backend. V1 omits the new member completely, keeping
its old bytes/digest contract unchanged. Version/member mismatches and unknown
settings refuse. Exact seed values, including maximum u64, remain decimal text.

The recipe digest enters RCTX source hashes before context/certificate creation.
Checks compare manifest scope/seed/tolerance, recipe digest and recomputed context
hash. Stripping a bound recipe or downgrading a v2 record cannot silently enter
legacy replay. Local consistency does not authenticate the producer or execution.

The graph digest covers ordered adjacency, native i64 population, sorted weighted
edges, index/GEOID mapping, counts and centroids, with explicit lexical graph/map
keys. Original binary hashes separately bind every encoded geometry attribute.
Observed graph/companions, explicit auxiliaries and selected geometry are hashed,
including recorded absent companions. This is not an atomic complete snapshot of
all implicit native discovery or authenticated upstream Census data.

## V3 execution and multiscale inputs

Native producers use `bisect-native-state-replay-recipe-v3` when they consume a
multiscale compositor or accept a population retry. The original resolved
algorithm retains its requested seed. After the v2 backend member, v3 appends
mandatory `execution`, ordered as `balance_retry_attempt`, `accepted_seed`,
`multiscale_sampler`. Retry attempts are 0..50 and the accepted seed must equal
the shared wrapping-u64 native schedule; omitted seed remains null at attempt 0.
Accepted seeds are decimal text or null in this recipe. V1/V2 omit execution
entirely and retain their old typed bytes/digests.

Consumed multiscale execution requires a bound native pointer-width sampler.
The current worker implements `rand-0.8.6-smallrng-native64-wilson`, independently
checked against pinned 64-bit native SmallRng, Wilson and complete ReCom traces.
Native32 and unknown legacy sampler history refuse. Existing requests omitting
`native_sampler` keep the independent ChaCha12 portable profile; it is never
substituted for recorded native64 history.

The computational graph remains the parent tract graph. BG sources separately
record requested path, binary, GEOID and centroid identities. Browser selection
is exclusively by recorded hashes over uploaded bytes; paths are labels. The
strict parent/fine population, quotient and connectivity checks remain active.
Initialization uses the shared native ordered WeightSpec/COI pipeline; fine/coarse
sampling and percentile ranking remain unweighted. Final population checking
uses the fine graph and exact fraction; successful native64 builds must also
pass fine connectivity before accepted result/export handoff. The native retry
order is retained. Raw native64 execution without Build also validates its final
result; it cannot bypass the guard by skipping retry orchestration.

V3 recorded replay matches literal fine assignments and bound requested seed,
accepted seed, retry count and sampler. UI and inert Save/Open preserve both
seeds. Exported engine Options use safe JSON integers up to 2^53-1 and decimal
text above that boundary; recipe and retry evidence retain decimal text. Fine
physical boundaries/population/unit identity are exported without invented
geometry, edge classification or fine demographic counts.

## Legacy native version claims

V1 metadata remains unchanged. Its manifest also has a native version label
written by `detect_gpmetis_version()`. The exact labels
`METIS 5.1.0 (vendored C FFI via metis-rs 0.2)` and
`metis-core (pure Rust workspace engine)` identify the native recursive backend
claim without guessing from configured engine or rewriting the recipe.

Resolution returns `legacy-native-version-claim` and `bound_to_recipe: false`.
The UI states that this claim lies outside the v1 recipe hash. Missing or ambiguous
labels refuse when effective recursive dispatch is needed. A recognized version
claim contradicting a bound v2 backend also refuses. These checks do not sign or
authenticate legacy metadata; literal reproduction remains conditional on the
selected record. Negative legacy seeds without exact unsigned metadata refuse.

## Selected-source preparation and replay

`inspect-native-replay-recipe` performs the existing selected ILP/certificate
prechecks, current-profile recipe decoding and manifest/context binding.
`inspect-native-replay-sources` additionally matches every recorded existing
source hash to explicitly selected bytes and checks the shared native reader's
graph digest. File names are labels; recorded native paths never grant browser
filesystem or network authority. Recorded absence remains a historical claim.

`prepare-native-recorded-replay` validates those sources and the original selected
RPLAN, then uses the existing State compositor to prepare a request without
partitioning. `run-native-recorded-replay` repeats validation and explicitly runs
the existing prepared engine/population retry/export boundary, then compares full
GEOID/district assignments literally to that original RPLAN. No district relabeling,
tolerance adjustment, alternate search or best-of selection is used to make a
comparison pass. The diagnostic State command uses synthetic scope flags rather
than original argv and is labeled `synthetic-scope-validation-only; not original argv`.

The engine's optional `balance_tolerance_fraction` retains original native f64
bits. Percent remains display metadata and must equal the recorded fraction times
100 exactly when the fraction is supplied. Generation, multiscale tolerance and
final population acceptance use the effective fraction. Omitting the field keeps
the prior percent-only behavior and absent-field serialization. A causal [2,4]
population boundary verifies exact 1/3 accepts on attempt zero while percent-only
roundtrip is one ULP smaller and exhausts the unchanged retry sequence.

Native Verify checks effective backend compatibility before its running marker
and before replay output effects. A C-enabled native binary does not silently
substitute its compile-time recursive backend for a recorded Rust choice. Browser
core builds refuse unavailable C requests. C-profile browser runs load only the
identity-verified packaged C runtime after explicit selection, preserving the
Windows CRT profile; project data cannot supply executable module authority.

Preparation and replay are separate workbench operations. Results show exact
seed, recorded search/split, configured/effective backend, evidence scope,
tolerance and literal comparison. New output label/timestamp are explicit, never
restored deletion/reset/reprocess controls. Open restores data without running,
loading recorded paths or selecting a runtime. Save/Open retains original bytes.

## Remaining completion requirements

Current actual replay evidence includes the original population/geographic cases
and 16-unit demographic/partisan/character composition controls under Single search. Every native preset and sixteen searches roundtrip
through the codec, but actual all-method backend consumption is not proved by that.
Recorded COI/partisan, demographic and combined character stages now use selected
raw sources and the shared native compositor. Non-population constraints remain guarded until their actual consumed inputs are prepared.
Selected-source standard/adaptive multiscale replay now has scoped synthetic
core/C and browser evidence; this does not complete real-source/method acceptance. The full native64 readonly recipe domain,
complete map/export handoffs and lifecycle/capacity acceptance remain open.

Producer evidence capture remains optional after successful native computation:
pickle-only inputs, unrepresentable unused numeric settings or capture failure
warn and omit an invalid recipe rather than erase a completed plan. Selected
package/byte limits do not establish support for arbitrary browser memory sizes.

The current gates retain 58 cases per core/C profile, 49 actual native replay/
refusal controls, twelve Chrome journeys, eight codec tests, 58 Verify unit tests
plus a separately executed real L2 case, 28 old-source regressions and the causal
precision boundary. Independent same-agent Python reads 116 whole reply cases
and 36 executed original native assignment/source/graph results. Original failed
baselines and the narrower R1 implementation remain preserved.

Source inventories/builders, paired native/WASM producer artifacts, maintained
integration, full amended standard/C packages and every remaining original
host/failure/capacity/raw-source acceptance obligation must still be completed.
Review: `signals/roles/check/wasm-effective-native-backend-replay-roles-check-2026-11-02.md`.

## Historical selected character discovery replay checkpoint - 2026-11-02

Native discovery uses the run working directory, not the configured manifest data
root: `data/{year}/lodes/{state_name}_wac_tract.csv` and
`data/{year}/acs_housing/{state_name}_housing_{year}.csv`. New captures add
`implicit-economic` or `implicit-housing` source identities for configured stages,
including observed bytes/absence when the single-district route skips loading.
Ordinary State discovery is unchanged. Explicit native replay resolves these
auxiliaries from the recorded root and preserves that root over later replay
generations, rather than drifting to the new invocation directory.

`native_auxiliary.character` optionally carries `kind` plus raw `source_name` and
`bytes_base64` (null only for a recorded absence claim). Omission preserves the
earlier COI/partisan serialization. Source names are labels; no project path grants
filesystem authority. Transport is canonical bounded base64. Content/read/native
parser failures retain native warning/fallback behavior; malformed transport and
conflicting explicit/native selections refuse. Missing loaded economic IDs use
zero vectors, missing housing IDs use neutral vectors, and duplicate rows use the
last native row. Empty/error files preserve the prior weighting. Single-district
recorded character weighting skips the reader. Exports retain raw selected bytes
or explicit absence claims and visible receipts explain defaults/fallbacks.

Raw `bisect.native-character-bytes` and existing alignment/source hash fields use
unprefixed hex; canonical `bisect.native-auxiliary-claims` uses `sha256:`. An
absence claim is not a hash of an absent file. Native source observations remain
non-atomic and do not authenticate upstream publication or historical execution.

Actual evidence covers 44 native comparisons to pre-character-adapter images,
88 checks per WASM profile, fourteen Chrome Save/Open journeys, ten hostile
controls, eight two-generation native controls, 49 older native Verify controls,
and independent Python source/alignment/finite-edge/population/BFS readback of 70
executions. Earlier auxiliary/backend/source/precision controls still pass. The
eight real root-drift failures and separate wrong-command harness failure remain.

Combined weight stages, older character records without required identities
(including the unused single-district compatibility route), demographic and
multiscale replay, native64 readonly metadata, complete map/export handoffs,
maintained source/gate/package integration and original full acceptance remain
open. Local image/browser tests do not establish final package or hosting support.
Evidence: `target/native-verify-parameters-20261102/character-replay-phase/`.

## Current selected native composition checkpoint - 2026-11-02

The original CLI `edge_weights` implementations now live in `bisect-core` with
the CLI namespace re-exported. `bisect-runner::native_weighting` shares the exact
State route and stage order: geographic base, minority override, partisan override,
county augmentation, economic character, housing character. A later override
replaces prior weights; character read/parse/empty failures preserve the preceding
composed map. Sparse maps remain sparse. A composer with no added stages uses
native physical fallback; the earlier Unweighted and SingleDistrict routes retain
their original skips. Native zone discovery remains a deferred spatial-join stage;
unused subdivision declarations are not silently promoted into implemented joins.

New captures record `implicit-demographics` when configured by minority weighting,
VRASection or VRA ReCom, along with existing character-discovery roles. The native
demographic path is `data/{year}/demographics/{state_name}_demographics_{year}.csv`.
The actual shared reader preserves numeric zero defaults, duplicate replacement,
GEOID padding, missing-unit zero and native nonfinite fractions. These are native
total-population fractions, not fabricated VAP/CVAP counts. Required minority
loading still fails even if a later partisan stage would override its map.

An optional `native_weighting` WeightSpec uses the `native-composed` engine mode.
Absence preserves ordinary request serialization. It cannot coexist with duplicate
scalar weight controls or typed observation channels. Raw native auxiliary input
adds optional `demographics`, `economic` and `housing` sources; the older single
`character` channel remains readable. Recorded replay binds role/path/hash and
original settings before preparing this carrier. Exports retain its exact WeightSpec
under `native_weighting_spec` and canonical `bisect.native-weighting-spec` identity,
plus original bytes/absence claims under the existing auxiliary provenance.

The nested specification is checked through the native profile guard before
execution, including a partisan declaration whose single-district source is not
read. The initial real bypass, native CLI refusal, corrected host/WASM refusals and
deliberately cancelled owned build are retained. An older recipe need not invent
character source history when the native route does not consume that source;
four original old records now replay literally in native and both WASM profiles.
Recorded existing source identities still rehash, and required consumed identities
remain mandatory. No path label grants browser filesystem or module authority.

Evidence: 42 pre-extraction native outcomes plus two separately declared crossed
data comparisons; 68 checks per WASM profile; 23 policy boundaries per profile;
16 Chrome Save/Open journeys; 35 original weighter tests moved into the shared core;
49 older native Verify controls. Independent Python reads 136 replies and 53
executed original assignments, typed recipe/context/source digests, complete sparse
composition formulas, population and BFS. The crossed data makes both character
stages change edges retained by the partisan override. The prior 88 character,
96 auxiliary and 58 backend controls per profile, 28 source checks and exact-fraction
boundary still pass. Failed native file loads are never fabricated completed plans.

Remaining: source-bound multiscale fine/coarse/block-group replay, non-population
constraints, native64 readonly metadata, actual all-method/real-source coverage,
diagnostic/map/export/follow-up and lifecycle/capacity/host acceptance, maintained
schema/source/pin/gate integration and original amended release obligations. This
private candidate is unadopted and undeployed; local image tests do not establish
final package support or complete upstream/historical source authenticity.


### Native64 readonly metadata candidate — 2026-11-02

Recipe, resolved state, algorithm, split and search types share a count parameter.
Native APIs default to usize; selected-byte metadata uses u64. Enum names, typed
field order and v1/v2/v3 canonical recipe hashes remain unchanged. Inspection does
not allocate a declared search budget. Runtime preparation retains the full
resolved algorithm JSON, translates only consumed options, and leaves execution
subject to checked engine/resource validation. District metadata is retained as
u64 rather than using an unchecked usize cast.

If any unsigned setting exceeds u32::MAX, the binding receipt adds
native64_readonly: exact decimal rows with fits_u32/javascript_safe flags, plus
native serialized search_json and structure_json. This is an additive wide-only
receipt; ordinary old receipts preserve omission and their previous whole-reply
shape. Raw manifest and recipe strings remain authoritative. The UI uses those
strings and decimal rows, avoiding unsafe JavaScript Number reserialization.

The owned pre-refactor native64 decoder supplies 435 canonical text/hash
specimens: 28 pointer-sized settings and the fixed u64 ILP time limit, five
boundaries and v1/v2/v3. These are explicitly metadata-only artificial manifests,
with no completed algorithm or certificate claim. There are also 116 native
invalid-count controls and sixteen actual native NWay reference runs whose
search settings are recorded but not consumed. Their 32 preparation/Run checks
must preserve the original declared algorithm and literal assignments. Maximum
metadata acceptance does not claim execution support for maximum active budgets.

Core WASM passes the 435 exact oracle cases; 336 valid pointer-width cases are
refused by the retained previous image. Independent Python validates exact
integers and native serialization hashes. Four Chrome journeys preserve original
manifest/context/recipe bytes through inert Open/Save/Open and show exact large
values. The phase status records final profile checks and remaining obligations.
The candidate remains unadopted; full integration, capacity/workload boundaries,
fine geometry, all-method/real-source and original release acceptance remain open.
Evidence: target/native-verify-parameters-20261102/readonly-recipe-phase/status.json.


### Recorded computational TIGER inputs — 2026-11-02

Area constraints or CompactBisect capture algorithm-tiger-shp/dbf/shx/prj
identities, including observed absence, for the original native single candidate
`data/{year}/tiger/tracts/tl_{year}_{fips}_tract/tl_{year}_{fips}_tract.shp`.
These observations are separate from report geometry's three-candidate lookup.
Native replay uses the recorded working-directory root. Selected-byte replay
checks exact source hashes and path linkage, then reads native TIGER streams in
memory and shares the original ALAND/GEOID/circular-perimeter alignment. Missing
graph IDs get zero; missing/unreadable input preserves native warning fallback.
CompactBisect recalculates circular perimeters with resolved algorithm weights.
Default AreaSection population/area constraints now reach the existing shared
area partitioner; other unsupported constraint configurations remain explicit.

Older geometry identities are accepted only when their paths exactly identify
the computational candidate. No historical absence or implicit input is invented.
Unknown or inconsistent older evidence refuses. Aliased observations for a
consumed computational TIGER path must agree on presence/hash. Raw report source
identity remains in native context even when full geometry parsing/alignment
fails; this does not assert complete or valid geometry. Fine BG output does not
inherit a tract geometry identity.

The source receipt adds algorithm_geometry only when the algorithm consumes it.
It retains exact aligned areas, the original physical-weight perimeter estimate,
source roles/hashes, native outcome, missing IDs and duplicate count. The UI shows
these observations and preserves raw source bytes through inert Open/Save/Open.
No native path is opened by a browser. These are algorithm inputs, not measured
polygon perimeters or fine BG map geometry.

Evidence includes 42 unchanged native outcomes (28 completions, fourteen failures),
28 native replays from an empty cwd, 112 current/legacy preparation/replay checks
per WASM profile, ten input boundaries per profile and sixteen Chrome journeys.
Independent raw DBF/SHP, RADJ, population/BFS and literal-assignment readback covers
224 replies and 68 executions. Prior native runner/recipe, source/fraction,
multiscale, weighting and native64 metadata checks remain green. The phase status
records exact roots, image hashes, retained red cases and the audit-output naming
incident. Current sources are private and unadopted. Fine geometry/map/export,
general constraints/methods, real-source/capacity and original release acceptance
remain open. Evidence: geometry-replay-phase/status.json.


### AreaSection preconditions and NWay dispatcher correction

AreaSection's computational TIGER read is enabled by the presence of Area in
its declared constraint list. It ignores the computed interleaved constraint
matrix and passes the original population vector to its area partitioner. Thus
Area-only, reordered and duplicate-Area lists retain their exact recorded list
and follow native behavior; lists without Area refuse before partitioning even
when the selected adjacency/report contains area vectors. Outside AreaSection,
the empty-list native population fallback is preserved; other unsupported
non-population matrices remain guarded rather than silently coerced.

NWay also uses the native compile-time C/core dispatcher. Effective backend
resolution now includes NWay; native replay refuses incompatible binaries and
WASM requires explicit trusted C availability when the bound original uses C.
The existing v2/v3 field and canonical serialization remain unchanged. The old
reader's configured-core execution differs from actual C-native NWay assignments;
the corrected core reader refuses, and the C profile is checked literally against
those unchanged original assignments. Native producer data and declarations,
including the earlier incorrect dispatch assumption, remain retained with a
separate correction note. This is a targeted semantic correction, not a stronger
historical authenticity claim or complete constraint/method acceptance.

Current evidence and integration draft:
`target/native-verify-parameters-20261102/constraint-preflight-phase/status.json`.

The maintained native and browser checks now cover the retained Rhode Island
2020 tract graph, 250 units, two districts and NWay seed 42. Both native builds
receive the same public flags; the recipe records the effective compiled backend.
The C-enabled browser uses the verified C runtime only for explicitly selected
runs. The portable build and an unchecked C-enabled run refuse the C record
before execution. The checks also cover changed source bytes, inert Open, exact
browser exports and both directions of native backend mismatch.

The scoped test sources are `scripts/web/test_native_ri_nway_replay_cli.py`,
`scripts/web/test_native_ri_c_replay_browser.mjs` and
`scripts/web/test_native_ri_c_replay_evidence.py`. The verified evidence is
retained in the current local parity capsule.

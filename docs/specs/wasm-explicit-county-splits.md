# Explicit county identities for custom-unit split analysis

WP-09's shared analyzer now counts county splits for declared precinct/imported
units using complete explicit county IDs. Desktop and WASM call the same shared
implementation. Census analysis keeps its existing FIPS-prefix behavior and
report bytes; numeric-looking IDs declared as custom do not enter that path.

For **County and municipal splits**, select the exact RPLAN and matching RCTX,
then Run explicitly. Custom units require `subdivisions.county_ids` covering
every ordered unit. Plan/context kind, state, year, ordered IDs and universe
identity must match; native RCTX hash validation still runs first. Missing,
blank-only, duplicate, misaligned or incomplete identities refuse analysis.
Missing county evidence is not reported as zero splits. Unit/county IDs remain
opaque, preserving their exact spelling, within 200 UTF-8 bytes and without
control characters.

The new wrapper schema is `bisect-explicit-plan-splits-v1`; the old Census
`bisect-plan-splits-v1` response is unchanged. Custom split entries use
`county_id`, `unit_count`, and `municipality_id`, rather than labeling opaque IDs
as FIPS or all units as tracts. Only supplied county names are used. Automatic
FIPS-name lookup and legal standard/compliance assessments are omitted for this
custom path. Counts are operational descriptions, not legal preservation or
source-authenticity findings. Optional geography lookup retains its existing
`county_names`, `place_to_tracts` and `place_names` input keys; membership values
are the exact supplied custom unit IDs.

Every custom report includes `county_identity_sha256`, the SHA-256 of the UTF-8
domain `bisect-explicit-county-identity-v1` followed by a zero byte and the shared
serializer's compact JSON unit-to-county map in sorted key order. This binds the
effective mapping even when only `splits.json` is retained or exported. It is a
separate semantic identity, not a replacement for the RPLAN/RCTX hash algorithms.
The wrapper additionally records the exact input RCTX byte hash. Hashes do not
authenticate the underlying geographic claims.

When the matching RCTX also supplies `subdivisions.municipal_ids`, the wrapper
uses `bisect-explicit-plan-splits-v2` and records a separate municipal membership
hash. Each ordered unit has one opaque municipal ID or a null for unknown
membership. Complete membership permits a true zero split count; any unknown
membership leaves municipal counts unavailable and preserves the county report.
Supply municipal membership through either RCTX or the optional geography
lookup. The browser rejects both sources together. The native and browser
results were independently checked for six report cases and three refusals in
both engine profiles, including inert offline Open. See the [candidate evidence](wasm-current-amendment-evidence.md).

Accepted-plan and label-analysis packages already carry matching RCTX inputs.
They now feed those contexts into split analysis automatically after explicit
Run. The existing three custom State follow-up fixtures compute splits, preserve
the exact accepted plan/context, and complete maps/reports. Their workflows
remain partial because optional election inputs are unavailable. Missing
geometry and other follow-up failures still preserve the accepted partition.

Desktop `bisect analyze` gains `--splits-context <path>` with `--types splits`
alone. The selected target must also contain `plan.rplan` and native canonical
zero-based index-keyed assignments. The CLI verifies the RPLAN/RCTX identity,
state/year and every assignment against the portable plan before using opaque
unit IDs. It does not use the old short-key-to-Census heuristic on this path.
For example, with an installed package under `outputs/vExplicit/2020/plans/ExplicitCounties`:

```powershell
bisect analyze --state RI --year 2020 --version vExplicit --output-dir outputs --label ExplicitCounties --types splits --splits-context outputs/vExplicit/2020/plans/ExplicitCounties/context.rctx --stdout
```

The existing native Analyze year/label restrictions remain; this does not add
arbitrary year or jurisdiction discovery. Calls without the new flag retain
their previous behavior. Plan/geography inputs retain the 3 MiB browser bound,
and matching custom context uses the existing 8 MiB/100,000-unit text-audit
boundary. Larger/original national capacity is not established by this change.

Validation:

- Shared Rust split tests: 33 pass, including four new custom identity tests and
  unchanged Census/policy-snapshot regressions.
- `test_explicit_county_splits.mjs`: eight exact native-host replies, five actual
  desktop CLI cases, eight refusals, an independent membership oracle, and three
  frozen legacy responses. CLI report bytes match WASM plus the CLI's required
  final newline. Both precinct/imported and numeric-looking custom IDs are
  covered. Context-negative fixtures use the native RCTX writer where their
  structure is valid, so missing identity checks are exercised independently
  from hash-tampering rejection.
- Small inputs and frozen legacy witnesses are source-owned. The optional
  `--previous-wasm <path>` tier additionally replays a preserved old artifact;
  absence is recorded as not-run and is not a clean-checkout prerequisite.
- `test_explicit_county_splits_browser.mjs`: matching context selection, explicit
  IDs in tables, safely rendered supplied names, exact report download/retention,
  valid-hash missing-ID refusal, repair, saved identities, inert Open/edits,
  no analysis partition and 390px layout.
- Existing split tests retain six independent native report-byte matches and
  six rejections. Combined analysis package tests pass. The custom State
  workflow retains 40 native replies, three browser-equal runs and independent
  workflow/output ZIP/hash checks, now including the explicit county-map hash.

Both builders gate the new tests and build the native CLI/reference context
writer. Full native workflow parity, broader compatibility/capacity coverage,
formal release evidence and the remaining plan work items remain open.

Scoped roles review: [seven repository lenses](../../signals/roles/check/wasm-explicit-county-splits-roles-check-2026-10-22.md), approved with wider implementation/release conditions. The verified local
package is `dist/wasm-lab-build`, offline version
`d00bd941b135d0f989405735551101c68d51fccbcd426644913cad156e85b80e`, with WASM SHA-256
`a895ce632ec01ff7125fd95deec6d08bca524946f713faab09affeaf05ada1a7`. The packaged browser check,
256-file/1,028-dependency toolkit check and 620-hash offline check pass.

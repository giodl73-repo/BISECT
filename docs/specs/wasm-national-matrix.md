# National retained-graph compatibility campaign

Status: actual native CLI, both direct metis-core WASM tiers and both staged
browser queue/recovery/map tiers passed; acquisition, broader host/lifecycle and
immutable evidence/scoped national acceptance remain open.
Owner: coding agent; milestone acceptance:
BISECT maintainer.

The authoritative `national-50-2020-tract-standard-core-42-v1` campaign is appended
to [the existing real-data matrix](wasm-real-data-matrix.json). It was declared
before execution: all 50 states, 2020 tracts, congressional allocation, 435 total
districts, 84,208 units and 330,759,736 population. DC and territories are explicitly
outside this participation set. It fixes standard bisection, geographic weights,
single search, metis-core, requested seed 42, 0.5% tolerance and four workers.
Native retry order is retained; the accepted seed may differ by state. Missing or
failed states cannot be dropped or replaced by samples. Original source graph
and request identities are pinned per state.

## Native checkpoint

`target/national-matrix/native-1792825774701/evidence.json` records a single
actual `bisect build` invocation for the whole declared set. All 50 states
succeeded; its native index and registry completed. Every retained graph was
first transcribed losslessly into native binary inputs. Independent checks then
verify all units, every full requested nonempty district, population sums, the
0.5% integer bound and graph connectivity for every state. Exact native assignment
files and hashes are retained for later independent WASM/browser comparisons.

The first fixture attempt, `target/national-matrix/native-1792825494017`, failed
with ENOSPC before the CLI invocation; it remains retained and is not a native
algorithm result. The new attempt followed lossless space recovery and an
explicit validation evidence-capacity preflight.

This native checkpoint is a retained-graph tier. It does not prove fresh raw Census
source derivation/authenticity, remote acquisition, WASM partition execution,
browser queue/worker behavior, saved continuation, output installation, national
map rendering or a general all-method/all-vintage/all-resolution claim. Those
require their own actual records before national support can be registered.

## Both actual WASM profiles

`target/national-matrix/wasm-1792827622072/evidence.json` (default) and
`target/national-matrix/wasm-1792827814125/evidence.json` (C-enabled) pass all
50 states and 435 requested districts. Each exact unit assignment matches the
independent native CLI output; population totals, nonempty districts, 0.5% integer
balance and graph connectivity are recomputed independently. Both profiles use
metis-core for this declaration. The requested seed stays 42 and each state's
actual accepted retry is retained in its outcome.

This is direct per-state WASM execution of the complete roster. It does not prove
the browser's national queue/four-worker scheduler, Save/Open/recovery, national
output/map, acquisition or fresh source derivation. Those remain explicit next
acceptance tasks. The declaration records the new evidence digests without
rewriting its predeclared targets or historical snapshots.

## Actual C-enabled browser queue and recovery

The [aggregate-capacity follow-up](wasm-national-browser-capacity.md) fixes actual
queue/archive/export limits exposed by the complete roster. The corrected staged
C-enabled metis-core browser passes four workers, all 50 full receipts, exact
Save/Open, ZIP, completed-prefix cancellation and explicit saved verification
followed by unfinished-only continuation. Altered runtime/source checkpoints
refuse before partition. Default browser validation is running. National maps,
fresh source derivation/acquisition, broader host/API and immutable evidence/scoped
support closure remain open. These records do not establish C-FFI execution or
formal national/full-parity acceptance.

Current update: both staged metis-core profiles now pass the full queue, saved
recovery/export and altered-checkpoint refusal contract. Both also pass the
[separate complete national map GUI](wasm-national-retained-map.md), matching a
fresh native stream reference and preserving exact sources/results on Save/Open.
Source derivation/acquisition, wider host/API/lifecycle and immutable evidence/
scoped registration remain open; this supersedes earlier pending browser/map
statements without rewriting their historical records.

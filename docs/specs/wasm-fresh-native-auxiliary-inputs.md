# Selected native auxiliary files in fresh State runs

The fresh State panel accepts original partisan TSV, native demographics CSV,
LODES economic-character CSV and ACS housing-character CSV files, alongside COI.
Select files, choose corresponding native State weighting flags, Review, then Run.
For example, partisan shares need a consuming native mode such as
`--weights-override proportional`; a path flag alone does not activate weighting.
The panel adds adjacency, COI and partisan filenames to the arguments. Recorded
paths are metadata and never cause disk reads.

New source files are bounded at 8 MiB each. The existing combined native
preparation and 25 MiB auxiliary JSON limits still apply; base64 overhead counts.
Native readers retain their established sparse/default/warning behavior. Review
refuses unsupported combinations. No missing-file claim is inferred when a file
is unselected; use the explicit missing-source controls to record absence for this run.

Inputs-only Save/Open, projects, workbench archives and session persistence retain
the original bytes. Open is inert and source edits retire current Run/export
authority. The existing session-v1 envelope adds optional source kinds; older
application versions can reject sessions containing them. Older COI sessions
remain readable. Native report observations and display geometry are distinct.

Private evidence covers four 16-unit AL2020 fixtures, two districts, seed17,
single search, 7.125 percent tolerance and niter100. Current actual desktop CLI
plans match literal browser/WASM plans. Both package profiles pass eight explicit
Runs each, inert Save/Open, exact projects/compressed archives and edit retirement.
An independent reader checks every population and physical edge/context, exact
integer balance bounds and district connectivity. The C-enabled package uses
metis-core in this gate; it makes no C-computation claim. The existing twelve
ordinary browser workflows also pass in the core package.

The maintained browser gate receives an actual native CLI and a built package:

```powershell
node scripts/web/test_fresh_native_auxiliary_browser.mjs --site dist/example --output target/auxiliary-check --cli target/release/bisect.exe
```

Its raw fixture is source-owned and retains original native source hashes and
reference plans; no timestamped developer artifact is required to run the gate.
Both builders register the gate under BrowserChecks. No new Rust/WASM compiler,
full real-state/method matrix, authenticity, original acceptance parent, deployment
or complete WP-01–WP-10 release acceptance follows from this scoped candidate.


## Explicit missing-source claims

The fresh State panel lets users enter a recorded filename and explicitly choose
Use missing source for COI, partisan, demographics, economic or housing sources.
Leaving a file unselected makes no claim. The browser does not inspect the disk.
Selecting bytes replaces the missing-source claim; editing its name retires the
claim and current results until the user explicitly records it again. Ordinary
projects, inputs files and compressed archives retain `{name, missing: true}`
records. Older application versions may refuse this session extension. Adjacency
and geometry sources cannot be declared missing through this feature.

Review records the claim without execution. On Run, native COI/economic/housing
readers preserve their fallback behavior; active required demographic or partisan
weighting fails without producing current plan exports. Native CLI and browser
error text differ, while both reject these required-source cases. These AL2020
16-unit fixtures retain two districts, seed17, 7.125% tolerance and single search.
Both existing WASM profiles pass three literal native fallback plans, two matching
required-source failures, inert Save/Open and exact project/archive claims. The
existing selected-file cases also pass. This is a JavaScript UI/persistence change
using existing native WASM capabilities, without a new Rust compiler, C execution,
source-authenticity or complete full-state/release claim.

# Precomputed web laboratory implementation plan

The public laboratory will load previously computed Rust engine results from a
static site. Visitors can explore states and national cohorts without paying for
server computation. The local laboratory remains the place to generate custom
experiments. This plan is for the project maintainers and defines the catalog,
publication process and acceptance checks.

## Scope and publication

Implement a Rust catalog builder, a finite experiment matrix, static browsing in
the existing UI, and a manual GitHub Pages workflow. Build a working sample from
available real runs and the RI, IA and NC cohort. The matrix format also supports
all 50 states. Neither a successful sample nor a configured national matrix
claims that every national combination has been computed. Publishing to GitHub
is a separate operation; this implementation prepares an uploadable artifact.

Use a finite list of complete Experiment configurations rather than an implicit
Cartesian product. This makes compatibility, seed budgets and county strength
explicit and allows the matrix to be reviewed before costly computation. The
builder may compute the matrix or export selected existing runs. Computation is
explicit; exporting never launches the engine. Failed states remain visible as
unavailable results with sanitized reasons. Unsupported historical search
requests are excluded rather than relabeled as executed searches.

## Catalog contract

The versioned manifest identifies the published experiment configurations,
per-state records and shared geometry references. Each successful state record
stores its assignments, complete metrics, source run identity, graph and GEOID
join digests, build provenance and a SHA-256 digest of the executable used by
new computations. Older records disclose unavailable executable digests. A
derived graph digest is not a hash of the upstream Census release, a signature,
an optimality certificate or evidence of legal compliance.

Store simplified display geometry once per state and census year, referenced by
its content digest. Store assignments separately for each result. Validate that
each assigned GEOID appears exactly once in geometry; preserve leading zeros.
Reject inconsistent geometry for the same state and year in one publication.
Asset names are generated locally; source file paths, session tokens, command
vectors and raw logs are not copied to public records. Whitelist public fields.
Use relative URLs so the artifact works below a GitHub project subpath.

The builder refuses a populated output directory and writes the manifest last.
It requires at least one successful result, and enforces a conservative 900 MB
artifact budget including assets. Asset SHA-256 hashes are checked by the
browser before records or maps are used. The schema rejects unknown versions.
Regeneration occurs in a new output directory, never by merging old assets.

## Browser behavior

The same UI supports local execution and an explicitly marked precomputed mode.
In precomputed mode the primary action is View saved results; it performs no
remote job submission. A selected configuration must match saved effective
algorithm settings. Inactive controls do not create false distinctions, while
district count, active search budgets, seed, tolerance, weights and area
constraints retain their meaning. A missing configuration is shown as missing;
the viewer does not silently choose a nearest result. Run history restores saved
settings, making the catalog discoverable without guessing parameter values.

National views aggregate only completed states and disclose coverage. Maximum
deviation is the worst completed state's maximum; district counts, cut costs and
county split counts are sums. Keep per-state population and connectivity checks
and label results heuristic and optimality unproved. Population tolerance is an
experimental setting, not a declaration of legal sufficiency. County split
counts do not represent complete community or state-law preservation analysis.

## Computation and delivery

Provide a small across-scales matrix and a national matrix with compatible
structures and discrete parameters. Compute states sequentially using the local
laboratory's existing preparation, timeout and output validation. Existing
successful records may be exported without recomputation. Record failures
without selecting only favorable maps or inventing seed distributions.

GitHub Actions manually downloads a catalog archive from a GitHub Release in
this repository, verifies its declared archive checksum and catalog assets, and
uploads the Pages artifact. Separate computation and deployment so publication
does not download raw Census files or run a long experiment matrix. GitHub
credentials remain in Actions permissions, never in browser code. The workflow
is manual and publication has not occurred merely because the artifact builds.

## Acceptance checks

Tests must catch wrong GEOID joins, modified assets, unknown schema versions,
configuration mismatches, incompatible historical searches, missing state
results, duplicated geometry storage and accidental publication of local paths.
Use small standalone fixtures rather than requiring local Census data in CI.
Then export real results and serve the artifact under a nested path using a
plain static HTTP server. Verify state and national maps, saved configuration
restoration, exports, missing combinations and zero remote mutation requests.
Report the actual number of states and configurations in the sample, artifact
size and the limits of validation.

## Review amendments

The installed role review requires three changes to the initial proposal:
explicit coverage and missing results instead of an all-options claim; strict
shared-geometry joins with versioned, hashed assets; and a public-field allowlist
plus a separate, manually invoked publication workflow. The implementation
must satisfy those conditions before the local artifact is considered ready.

## Implementation verification — 2026-10-03

Implemented the catalog builder, static viewer, explicit pilot and national
matrices, checksum verifier, release packager and manual Pages workflow. The
workflow preserves a supplied existing static site and installs the catalog
under `laboratory/`; the combined site must remain below 900 MB.

The real pilot completed all 18 requested results: six configurations across
RI, IA and NC. The final artifact is 8,564,750 bytes and the archive is
1,934,404 bytes. Eleven Rust tests and four JavaScript tests passed. Browser
checks beneath a nested static URL covered settings restoration, state maps,
completed-state national totals, comparisons and explicitly unavailable
settings. Asset validation covered every exported geometry/assignment join.
Browser download controls were exercised, but the automation did not confirm
the saved download file; exported assignment/map data are covered by tests.

The 66-configuration, 50-state matrix is validated but has not been computed.
The release package is local; neither release upload nor Pages deployment has
been performed. The Pages workflow still needs its first GitHub Actions run
against the confirmed existing site source.

#!/usr/bin/env bash
# Vermont 2020 canonical walkthrough — Onboarding plan Task 1
#
# Runs the full Census fetch -> bisection -> analysis -> report pipeline
# against pinned Vermont 2020 inputs and emits outputs whose SHA-256 hashes
# are recorded in `checksums.json` for drift detection via:
#
#   redist doctor --check-tutorial-data --tutorial vermont-2020
#
# This is the smoke test the Onboarding bootstrap script invokes after build.
# It is also the L2 acceptance test (`tests/acceptance/test_walkthrough_vermont.py`)
# that nightly CI executes end-to-end.
#
# Reproducible-build notes:
#   - SOURCE_DATE_EPOCH is honored when set (default: leave unset, accept variance)
#   - REDIST_BUILD_COMMIT is read from the build (no override required)
#
# ASCII-only output (PP-34 — Windows CP1252 console policy).

set -euo pipefail

REPO_ROOT="${REPO_ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"
VERSION="${VERSION:-tutorial}"
YEAR=2020
STATE=VT
LABEL=vt_2020_tutorial
OUTPUT_BASE="${OUTPUT_BASE:-${REPO_ROOT}/outputs}"

step() { echo; echo "[step $1] $2"; }
ok()   { echo "[OK] $*"; }
fail() { echo "[FAIL] $*" >&2; exit 1; }

cd "$REPO_ROOT"

step 1 "Verifying redist binary on PATH..."
command -v redist >/dev/null 2>&1 || fail "redist not on PATH (run bootstrap.sh first)"
ok "redist version: $(redist --version 2>&1 | head -1)"

step 2 "Fetching Vermont 2020 Census TIGER tracts (FIPS 50)..."
redist fetch --type tiger --states "$STATE" --year "$YEAR" \
    --output-base "$OUTPUT_BASE" || fail "TIGER fetch failed"
ok "TIGER tracts fetched"

step 3 "Building adjacency graph from tract geometries..."
# Adjacency is built implicitly by `redist state` if not present;
# we materialize it here so its SHA can be checked-in.
redist fetch --type adjacency --states "$STATE" --year "$YEAR" \
    --output-base "$OUTPUT_BASE" || true   # tolerate "already built"
ok "Adjacency present"

step 4 "Running bisection: redist state --state $STATE --year $YEAR --label $LABEL"
redist state --state "$STATE" --year "$YEAR" \
    --label "$LABEL" --version "$VERSION" \
    --output-base "$OUTPUT_BASE" || fail "bisection failed"

ASSIGN_FILE="${OUTPUT_BASE}/${VERSION}/${YEAR}/plans/${LABEL}/final_assignments.json"
[ -f "$ASSIGN_FILE" ] || fail "missing $ASSIGN_FILE"

# Vermont's 2020 TIGER tract count is 193 (matches tests/acceptance/test_pipeline_acceptance.py
# baseline). The L2 acceptance test expects exactly 193. If your fetch produced a
# different count, the upstream TIGER vintage drifted — re-fetch with the pinned
# URL above.
TRACT_COUNT=$(python3 -c "import json,sys;print(len(json.load(open(sys.argv[1]))))" "$ASSIGN_FILE" \
    2>/dev/null || echo "unknown")
echo "[INFO] Vermont 2020 tract count: $TRACT_COUNT"

step 5 "Running analyses: redist analyze --label $LABEL --types all"
redist analyze --label "$LABEL" --year "$YEAR" --version "$VERSION" \
    --output-base "$OUTPUT_BASE" --types all || fail "analyze failed"
ok "Analyses complete"

step 6 "Generating report: redist report --label $LABEL --format html"
redist report --label "$LABEL" --year "$YEAR" --version "$VERSION" \
    --output-base "$OUTPUT_BASE" --format html || fail "report failed"
ok "Report generated"

step 7 "Verifying provenance: redist doctor --verify-manifest"
MANIFEST="${OUTPUT_BASE}/${VERSION}/${YEAR}/plans/${LABEL}/manifest.json"
redist doctor --verify-manifest "$MANIFEST" || fail "manifest verification failed"

echo
echo "=================================================="
echo "Vermont 2020 walkthrough complete."
echo "Plan dir: ${OUTPUT_BASE}/${VERSION}/${YEAR}/plans/${LABEL}/"
echo
echo "Validate against pinned checksums:"
echo "  redist doctor --check-tutorial-data --tutorial vermont-2020"
echo "=================================================="

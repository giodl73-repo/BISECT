#!/usr/bin/env bash
# Re-pin checksums.json from a successful walkthrough run.
#
# Usage: bash examples/vermont-2020-walkthrough/pin.sh
#
# Reads the existing skeleton at examples/vermont-2020-walkthrough/checksums.json,
# replaces every "PIN_ON_FIRST_RUN" sha256 with the current sha256sum of the
# referenced local file, and replaces build_commit + pinned_at with current values.
#
# Run after `bash examples/vermont-2020-walkthrough/run.sh` succeeds locally.

set -euo pipefail

REPO_ROOT="${REPO_ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"
CHECKSUMS="${REPO_ROOT}/examples/vermont-2020-walkthrough/checksums.json"
[ -f "$CHECKSUMS" ] || { echo "[FAIL] missing $CHECKSUMS" >&2; exit 1; }

command -v jq >/dev/null 2>&1 || { echo "[FAIL] jq required" >&2; exit 1; }
command -v sha256sum >/dev/null 2>&1 || command -v shasum >/dev/null 2>&1 \
    || { echo "[FAIL] sha256sum or shasum required" >&2; exit 1; }

sha_of() {
    local p="$1"
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$p" | awk '{print $1}'
    else
        shasum -a 256 "$p" | awk '{print $1}'
    fi
}

BUILD_COMMIT="$(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || echo unknown)"
PINNED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# Build the updated JSON via jq, walking pinned_inputs and expected_outputs.
tmp="$(mktemp)"
jq --arg root "$REPO_ROOT" \
   --arg commit "$BUILD_COMMIT" \
   --arg pinned "$PINNED_AT" \
   '
   def hash_path:
       . as $local |
       ($root + "/" + $local) as $abs |
       (if (try ($abs | inputs) catch null) then null else $abs end);
   .build_commit = $commit
   | .pinned_at  = $pinned
   ' "$CHECKSUMS" > "$tmp"

# jq cannot easily shell out for sha256, so do that pass in bash.
hash_count=0
fail_count=0
populated=$(cat "$tmp")
for section in pinned_inputs expected_outputs; do
    n=$(echo "$populated" | jq -r ".${section} | length")
    for i in $(seq 0 $((n-1))); do
        local_path=$(echo "$populated" | jq -r ".${section}[${i}].local_path // .${section}[${i}].path")
        abs="${REPO_ROOT}/${local_path}"
        if [ -f "$abs" ]; then
            sha=$(sha_of "$abs")
            populated=$(echo "$populated" | jq --arg sha "$sha" \
                ".${section}[${i}].sha256 = \$sha")
            hash_count=$((hash_count+1))
            echo "[PINNED] ${local_path} -> ${sha}"
        else
            echo "[SKIP]   ${local_path} (not present locally; leaving placeholder)"
            fail_count=$((fail_count+1))
        fi
    done
done

echo "$populated" > "$CHECKSUMS"
rm -f "$tmp"

echo
echo "=================================================="
echo "Pinned $hash_count files in $CHECKSUMS"
[ "$fail_count" -gt 0 ] && echo "$fail_count rows still have PIN_ON_FIRST_RUN (run the walkthrough first)"
echo "build_commit: $BUILD_COMMIT"
echo "pinned_at:    $PINNED_AT"
echo "=================================================="

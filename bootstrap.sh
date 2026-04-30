#!/usr/bin/env bash
# bootstrap.sh — Linux/macOS one-shot setup for the redist project.
#
# Onboarding plan Task 3 (https://github.com/.../docs/superpowers/plans/2026-04-30-onboarding-and-tutorials.md).
# Goals: get a clean machine from `git clone` to first useful run in <= 10 minutes wall-clock.
#
# What this does:
#   1. Detects rustup; installs if missing
#   2. Builds redist in release mode (uses pinned toolchain from rust-toolchain.toml)
#   3. PATH preflight (PP-18): verify binary at expected path BEFORE mutating PATH
#   4. PATH update + verify with `command -v redist`
#   5. Optional: --with-python builds the redist_py wheel via maturin and verifies the import
#   6. Optional: --with-api-key prompts for DATAVERSE_API_KEY and validates it via one round-trip
#               (PP-19) before writing to ~/.config/redist/credentials.toml
#   7. Real smoke test (PP-20): runs the Vermont walkthrough and asserts tract count
#
# ASCII-only output (PP-34 Windows console policy; harmless on Linux/macOS).
#
# Usage:
#   bash bootstrap.sh [--with-python] [--with-api-key] [--skip-smoke]

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
WITH_PYTHON=0
WITH_API_KEY=0
SKIP_SMOKE=0
for arg in "$@"; do
    case "$arg" in
        --with-python)  WITH_PYTHON=1 ;;
        --with-api-key) WITH_API_KEY=1 ;;
        --skip-smoke)   SKIP_SMOKE=1 ;;
        --help|-h)
            grep '^# ' "$0" | sed 's/^# //'
            exit 0
            ;;
        *) echo "[FAIL] Unknown flag: $arg (try --help)" >&2; exit 1 ;;
    esac
done

step() { echo; echo "[step $1] $2"; }
ok()   { echo "[OK] $*"; }
fail() { echo "[FAIL] $*" >&2; exit 1; }

cd "$REPO_ROOT"

# ── Step 1: rustup ───────────────────────────────────────────────────────────
step 1 "Checking rustup..."
if ! command -v rustup >/dev/null 2>&1; then
    echo "rustup not found; installing (this prompts for user confirmation)..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
    # shellcheck disable=SC1091
    . "${HOME}/.cargo/env"
fi
ok "rustup: $(rustup --version 2>&1 | head -1)"

# ── Step 2: pinned toolchain (rust-toolchain.toml in redist/) ────────────────
step 2 "Installing pinned toolchain..."
(cd redist && rustup show >/dev/null)
ok "rustc: $(cd redist && rustc --version)"

# ── Step 3: build redist ─────────────────────────────────────────────────────
step 3 "Building redist (release, --locked)..."
(cd redist && cargo build --release --locked --bin redist) \
    || fail "cargo build failed; see output above"

# ── Step 4: PATH preflight (PP-18) ───────────────────────────────────────────
step 4 "PATH preflight..."
EXPECTED_BIN="${REPO_ROOT}/redist/target/release/redist"
[ -x "$EXPECTED_BIN" ] || fail "build succeeded but binary not at expected path: $EXPECTED_BIN"
ok "binary at $EXPECTED_BIN"

# ── Step 5: add to PATH for this shell + persist hint ────────────────────────
step 5 "PATH update..."
export PATH="${REPO_ROOT}/redist/target/release:${PATH}"
command -v redist >/dev/null || fail "redist still not on PATH after update"
ok "redist on PATH: $(command -v redist)"
echo
echo "    To make this permanent, add to your shell rc (~/.bashrc, ~/.zshrc):"
echo "      export PATH=\"${REPO_ROOT}/redist/target/release:\$PATH\""

# ── Step 6 (optional): Python wheel via maturin ──────────────────────────────
if [ "$WITH_PYTHON" -eq 1 ]; then
    step 6 "Building redist_py PyO3 wheel via maturin..."
    command -v python3 >/dev/null || fail "python3 required for --with-python"
    if ! command -v maturin >/dev/null 2>&1; then
        echo "maturin not found; installing via pip..."
        python3 -m pip install --user --quiet maturin || fail "maturin install failed"
    fi
    (cd redist/python/redist_py && maturin develop --release) \
        || fail "maturin build failed"
    python3 -c "import redist_py; print('redist_py:', redist_py.__version__)" \
        || fail "redist_py import failed after build"
    ok "redist_py importable"
fi

# ── Step 7 (optional): API key with round-trip validation (PP-19) ────────────
if [ "$WITH_API_KEY" -eq 1 ]; then
    step 7 "Dataverse API key setup..."
    CRED_DIR="${HOME}/.config/redist"
    CRED_FILE="${CRED_DIR}/credentials.toml"
    mkdir -p "$CRED_DIR"
    chmod 700 "$CRED_DIR" 2>/dev/null || true
    echo "Enter your Harvard Dataverse API key (or press Enter to skip):"
    read -r -s DATAVERSE_API_KEY
    echo
    if [ -z "$DATAVERSE_API_KEY" ]; then
        echo "[SKIP] no API key entered"
    else
        echo "Validating API key with one Dataverse round-trip..."
        # Minimal endpoint that requires auth: list user's data
        HTTP=$(curl -s -o /dev/null -w '%{http_code}' \
            -H "X-Dataverse-key: $DATAVERSE_API_KEY" \
            'https://dataverse.harvard.edu/api/users/:me' || echo 000)
        if [ "$HTTP" != "200" ]; then
            fail "API key validation failed (HTTP $HTTP); not writing to $CRED_FILE"
        fi
        # Atomic write: tmp + rename
        tmp="$(mktemp "${CRED_DIR}/.cred.XXXXXX")"
        printf '[dataverse]\napi_key = "%s"\n' "$DATAVERSE_API_KEY" > "$tmp"
        chmod 600 "$tmp"
        mv "$tmp" "$CRED_FILE"
        ok "API key validated and written to $CRED_FILE"
    fi
fi

# ── Step 8: smoke test (PP-20: real run, not --print-only) ───────────────────
if [ "$SKIP_SMOKE" -eq 1 ]; then
    echo; echo "[step 8] Smoke test SKIPPED (--skip-smoke)"
else
    step 8 "Smoke test (real run)..."
    SMOKE_DIR="$(mktemp -d -t redist-bootstrap-smoke-XXXXXX)"
    trap 'rm -rf "$SMOKE_DIR"' EXIT
    redist state --state VT --year 2020 --label bootstrap_test \
        --output-base "$SMOKE_DIR" --version v1 \
        || fail "smoke test bisection failed"
    ASSIGN="${SMOKE_DIR}/v1/2020/plans/bootstrap_test/final_assignments.json"
    [ -f "$ASSIGN" ] || fail "smoke test produced no final_assignments.json"
    if command -v python3 >/dev/null 2>&1; then
        TRACT_COUNT=$(python3 -c "import json,sys;print(len(json.load(open(sys.argv[1]))))" "$ASSIGN")
        ok "smoke test produced $TRACT_COUNT tract assignments"
    else
        ok "smoke test produced $ASSIGN (python3 not available; tract count not verified)"
    fi
fi

# ── Done ──────────────────────────────────────────────────────────────────────
echo
echo "=================================================="
echo "Bootstrap complete."
echo
echo "Try:"
echo "  redist state --state VT --year 2020"
echo "  redist doctor --state VT --year 2020"
echo "  bash examples/vermont-2020-walkthrough/run.sh"
echo
echo "First-time docs: docs/quickstart/"
echo "=================================================="

import json
from pathlib import Path
import sys

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from run_block_ensemble_expansion_v3 import (
    BASE_SEED,
    ORDER,
    PROTOCOL_ID,
    command,
    expected_next,
    new_ledger,
    validate_ledger,
    validate_trace,
)
from verify_block_ensemble_expansion_v3 import verify_package
from verify_block_ensemble_v3_readiness import (
    binding_sha256,
    expected_probes,
    probe_command,
    sha256,
)


def test_v3_identity_seed_and_schedule_are_fresh() -> None:
    ledger = new_ledger()

    assert ledger["schema_version"] == "nrs-block-ensemble-expansion-ledger-v3"
    assert ledger["protocol_id"] == PROTOCOL_ID
    assert PROTOCOL_ID.endswith("-v3")
    assert BASE_SEED == 20260812
    assert expected_next(ledger, "preflight") == ORDER[0] == "NH:wilson"


def test_v3_rejects_closed_v2_ledger() -> None:
    ledger = new_ledger()
    ledger["schema_version"] = "nrs-block-ensemble-expansion-ledger-v2"
    ledger["protocol_id"] = "nrs-v0.3-block-ensemble-expansion-v2"

    with pytest.raises(ValueError, match="v3 ledger schema mismatch"):
        validate_ledger(ledger)


def test_v3_command_freezes_compiled_contract(tmp_path: Path) -> None:
    preflight = command(
        tmp_path / "block_trace.exe",
        "NH",
        "wilson",
        "preflight",
        tmp_path / "preflight.json",
    )
    governed = command(
        tmp_path / "block_trace.exe",
        "GA",
        "kruskal",
        "primary",
        tmp_path / "governed.json",
    )

    assert preflight[preflight.index("--base-seed") + 1] == "20260812"
    assert preflight[preflight.index("--execution-class") + 1] == (
        "excluded-expansion-v3-preflight"
    )
    assert governed[governed.index("--execution-class") + 1] == "governed-stage2-v3"
    assert governed[governed.index("--districts") + 1] == "14"


def test_v3_trace_rejects_v2_identity() -> None:
    trace = {
        "schema_version": "nrs-block-ensemble-trace-v1",
        "status": "complete",
        "execution_class": "excluded-expansion-v2-preflight",
        "state": "NH",
        "year": 2020,
        "districts": 2,
        "sampler": "wilson",
        "chains": 1,
        "steps_per_chain": 25,
        "population_tolerance": 0.005,
        "base_seed": 20260811,
        "snapshot_stride": 10,
        "chain_traces": [],
    }

    with pytest.raises(ValueError, match="v3 trace execution_class drift"):
        validate_trace(trace, "NH", "wilson", "preflight")


def test_v3_verifier_rejects_closed_predecessor_artifact(tmp_path: Path) -> None:
    package = tmp_path / "v3"
    package.mkdir()
    (package / "ledger.json").write_text(json.dumps(new_ledger()), encoding="utf-8")
    (package / "foreign.json").write_text(
        json.dumps({"protocol_id": "nrs-v0.3-block-ensemble-expansion-v2"}),
        encoding="utf-8",
    )

    with pytest.raises(ValueError, match="closed predecessor artifact"):
        verify_package(package)


def test_compiled_probe_matrix_covers_all_shapes_without_io_paths(tmp_path: Path) -> None:
    executable = tmp_path / "block_trace.exe"
    probes = expected_probes(executable)

    assert len(probes) == 14
    assert sum(probe["kind"] == "positive" for probe in probes) == 12
    assert sum(probe["kind"] == "negative" for probe in probes) == 2
    for probe in probes:
        argv = probe["argv"]
        assert argv[-2:] == ["--contract-only", "true"]
        assert "--rctx" not in argv
        assert "--assignments" not in argv
        assert "--output" not in argv


def test_probe_command_keeps_wrong_seed_visible(tmp_path: Path) -> None:
    argv = probe_command(
        tmp_path / "block_trace.exe",
        "NH",
        "wilson",
        "excluded-expansion-v3-preflight",
        25,
        1,
        20260811,
    )

    assert argv[argv.index("--base-seed") + 1] == "20260811"


def test_text_bindings_are_line_ending_portable(tmp_path: Path) -> None:
    lf = tmp_path / "contract.py"
    crlf = tmp_path / "contract-copy.py"
    lf.write_bytes(b"first\nsecond\n")
    crlf.write_bytes(b"first\r\nsecond\r\n")

    assert sha256(lf) != sha256(crlf)
    assert binding_sha256(lf) == binding_sha256(crlf)


def test_binary_bindings_remain_byte_exact(tmp_path: Path) -> None:
    first = tmp_path / "runner.exe"
    second = tmp_path / "runner-copy.exe"
    first.write_bytes(b"first\nsecond\n")
    second.write_bytes(b"first\r\nsecond\r\n")

    assert binding_sha256(first) != binding_sha256(second)

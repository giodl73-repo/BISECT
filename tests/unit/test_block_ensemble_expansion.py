from pathlib import Path
import sys

PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from run_block_ensemble_expansion import (
    ORDER,
    compress_trace,
    expected_next,
    load_trace,
    new_ledger,
    validate_trace,
)


def test_schedule_requires_full_preflight_before_primary() -> None:
    ledger = new_ledger()
    assert expected_next(ledger, "preflight") == "NH:wilson"
    assert expected_next(ledger, "primary") is None
    ledger["completed"]["preflight"] = list(ORDER)
    assert expected_next(ledger, "preflight-replay") == "NH:wilson"
    assert expected_next(ledger, "primary") is None
    ledger["completed"]["preflight-replay"] = list(ORDER)
    assert expected_next(ledger, "primary") == "NH:wilson"


def test_schedule_requires_full_primary_before_replay() -> None:
    ledger = new_ledger()
    ledger["completed"]["preflight"] = list(ORDER)
    ledger["completed"]["preflight-replay"] = list(ORDER)
    assert expected_next(ledger, "replay") is None
    ledger["completed"]["primary"] = list(ORDER)
    assert expected_next(ledger, "replay") == "NH:wilson"


def test_preflight_trace_shape_is_validated() -> None:
    trace = {
        "schema_version": "nrs-block-ensemble-trace-v1",
        "status": "complete",
        "execution_class": "excluded-expansion-preflight",
        "state": "NH",
        "year": 2020,
        "districts": 2,
        "sampler": "wilson",
        "chains": 1,
        "steps_per_chain": 25,
        "population_tolerance": 0.005,
        "base_seed": 20260810,
        "snapshot_stride": 10,
        "chain_traces": [
            {
                "chain_index": 0,
                "metrics": [
                    {"step": step, "max_population_deviation": 0.001}
                    for step in range(1, 26)
                ],
                "snapshots": [{"step": 10}, {"step": 20}],
            }
        ],
    }

    validate_trace(trace, "NH", "wilson", "preflight")

    trace["districts"] = 3
    try:
        validate_trace(trace, "NH", "wilson", "preflight")
    except ValueError as error:
        assert "districts" in str(error)
    else:
        raise AssertionError("district drift was accepted")


def test_deterministic_gzip_round_trip(tmp_path: Path) -> None:
    raw = tmp_path / "trace.json"
    first = tmp_path / "first.json.gz"
    second = tmp_path / "second.json.gz"
    value = {"chain_traces": [{"metrics": [{"step": 1}]}]}
    raw.write_text(__import__("json").dumps(value), encoding="utf-8")

    compress_trace(raw, first)
    compress_trace(raw, second)

    assert first.read_bytes() == second.read_bytes()
    assert load_trace(first) == value

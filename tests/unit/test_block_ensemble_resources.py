from pathlib import Path
import sys

PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from measure_block_ensemble_resources import calculate_budgets, normalize_in_place


def measurement(sampler: str, wall: float, peak: int, size: int) -> dict:
    return {
        "sampler": sampler,
        "status": "pass",
        "normalized_trace_match": True,
        "wall_seconds": wall,
        "peak_rss_bytes": peak,
        "scratch_trace_size_bytes": size,
    }


def test_normalization_removes_only_runtime() -> None:
    trace = {
        "chain_traces": [
            {"metrics": [{"step": 1, "runtime_ms": 9.5, "accepted": True}]}
        ]
    }

    normalize_in_place(trace)

    assert trace == {
        "chain_traces": [
            {"metrics": [{"step": 1, "runtime_ms": 0.0, "accepted": True}]}
        ]
    }


def test_budget_formula_rounds_up_mechanically() -> None:
    records = [
        measurement("wilson", 10.0, 100, 100),
        measurement("kruskal", 20.0, 200, 200),
    ]

    result = calculate_budgets(records, {"RI": 100, "NH": 200, "NM": 300, "GA": 500})

    assert result["projected_compute_seconds"] == 300.0
    assert result["authorized_compute_budget_hours"] == 1
    assert result["authorized_memory_budget_bytes"] == 256 * 1024**2
    assert result["authorized_retained_storage_gib"] == 1
    assert result["expansion_protocol_draft_eligible"] is True


def test_budget_formula_closes_gate_above_hard_ceiling() -> None:
    records = [
        measurement("wilson", 10_000.0, 1024**3, 1024**3),
        measurement("kruskal", 10_000.0, 1024**3, 1024**3),
    ]

    result = calculate_budgets(records, {"RI": 1, "NH": 2, "NM": 3, "GA": 5})

    assert result["expansion_protocol_draft_eligible"] is False


def test_budget_formula_rejects_failed_measurement() -> None:
    records = [
        measurement("wilson", 10.0, 100, 100),
        measurement("kruskal", 20.0, 200, 200),
    ]
    records[0]["normalized_trace_match"] = False

    try:
        calculate_budgets(records, {"RI": 100, "NH": 200, "NM": 300, "GA": 500})
    except ValueError as error:
        assert "exact replay" in str(error)
    else:
        raise AssertionError("failed measurement was accepted")

import json
from pathlib import Path
import sys

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from run_block_ensemble_expansion_v2 import (
    BASE_SEED,
    EXECUTABLE,
    ORDER,
    PACKAGE,
    PROTOCOL_ID,
    artifact_paths,
    capacity_admitted_launch,
    close_ledger_failure,
    command,
    expected_next,
    new_ledger,
    next_admission_path,
    require_bound_executable,
    require_official_package,
    validate_ledger,
    validate_trace,
)
from verify_block_ensemble_expansion_v2 import verify_package


def trace_fixture(state: str = "NH", sampler: str = "wilson") -> dict:
    return {
        "schema_version": "nrs-block-ensemble-trace-v1",
        "status": "complete",
        "execution_class": "excluded-expansion-v2-preflight",
        "state": state,
        "year": 2020,
        "districts": 2,
        "sampler": sampler,
        "chains": 1,
        "steps_per_chain": 25,
        "population_tolerance": 0.005,
        "base_seed": BASE_SEED,
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


def write_ledger(package: Path, ledger: dict) -> None:
    package.mkdir()
    (package / "ledger.json").write_text(
        json.dumps(ledger), encoding="utf-8"
    )


def test_v2_schedule_requires_complete_fresh_stage_zero() -> None:
    ledger = new_ledger()
    assert expected_next(ledger, "preflight") == "NH:wilson"
    assert expected_next(ledger, "primary") is None
    ledger["completed"]["preflight"] = list(ORDER)
    assert expected_next(ledger, "preflight-replay") == "NH:wilson"
    ledger["completed"]["preflight-replay"] = list(ORDER)
    assert expected_next(ledger, "primary") == "NH:wilson"


def test_v2_ledger_rejects_v1_identity() -> None:
    ledger = new_ledger()
    ledger["schema_version"] = "nrs-block-ensemble-expansion-ledger-v1"
    ledger["protocol_id"] = "nrs-v0.3-block-ensemble-expansion-v1"

    with pytest.raises(ValueError, match="v2 ledger schema mismatch"):
        validate_ledger(ledger)


def test_v2_ledger_rejects_skipped_schedule_position() -> None:
    ledger = new_ledger()
    ledger["completed"]["preflight"] = ["NH:kruskal"]

    with pytest.raises(ValueError, match="frozen-order prefix"):
        validate_ledger(ledger)


def test_v2_trace_rejects_v1_seed_and_execution_class() -> None:
    trace = trace_fixture()
    trace["base_seed"] = 20260810
    trace["execution_class"] = "excluded-expansion-preflight"

    with pytest.raises(ValueError, match="execution_class drift"):
        validate_trace(trace, "NH", "wilson", "preflight")


def test_v2_command_freezes_seed_and_execution_class(tmp_path: Path) -> None:
    args = command(
        tmp_path / "runner.exe",
        "NH",
        "wilson",
        "primary",
        tmp_path / "trace.json",
    )

    assert args[args.index("--base-seed") + 1] == str(BASE_SEED)
    assert args[args.index("--execution-class") + 1] == "governed-stage2-v2"
    assert args[args.index("--chains") + 1] == "4"
    assert args[args.index("--steps") + 1] == "2000"


def test_v2_artifacts_are_namespaced_by_phase(tmp_path: Path) -> None:
    preflight = artifact_paths(tmp_path, "NH", "wilson", "preflight")
    primary = artifact_paths(tmp_path, "NH", "wilson", "primary")
    replay = artifact_paths(tmp_path, "NH", "wilson", "replay")

    assert primary["final_trace"].name == "governed-nh-wilson.json.gz"
    assert replay["committed_trace"] == primary["final_trace"]
    assert len({preflight["resource"], primary["resource"], replay["resource"]}) == 3


def test_rejected_admission_can_advance_to_new_custody_name(tmp_path: Path) -> None:
    first = next_admission_path(tmp_path, "NH", "wilson", "preflight")
    assert first.name == "admission-preflight-nh-wilson-attempt-01.json"
    first.write_text("rejected custody", encoding="utf-8")

    second = next_admission_path(tmp_path, "NH", "wilson", "preflight")
    assert second.name == "admission-preflight-nh-wilson-attempt-02.json"
    assert first.read_text(encoding="utf-8") == "rejected custody"


def test_empty_v2_package_verifies_without_execution(tmp_path: Path) -> None:
    package = tmp_path / "v2"
    ledger = new_ledger()
    write_ledger(package, ledger)

    assert verify_package(package) == ledger


def test_v2_verifier_rejects_v1_protocol_artifact(tmp_path: Path) -> None:
    package = tmp_path / "v2"
    write_ledger(package, new_ledger())
    (package / "foreign.json").write_text(
        json.dumps({"protocol_id": "nrs-v0.3-block-ensemble-expansion-v1"}),
        encoding="utf-8",
    )

    with pytest.raises(ValueError, match="v1 protocol artifact present"):
        verify_package(package)


def test_fresh_ledger_has_no_v1_completion_or_claim() -> None:
    ledger = new_ledger()

    assert ledger["protocol_id"] == PROTOCOL_ID
    assert all(not values for values in ledger["completed"].values())
    assert ledger["retained_bytes"] == 0
    assert ledger["runner_wall_seconds"] == 0.0


def test_v2_cli_boundary_rejects_nonofficial_package(tmp_path: Path) -> None:
    with pytest.raises(ValueError, match="v2 package path must be"):
        require_official_package(tmp_path)

    assert require_official_package(PACKAGE) == PACKAGE.resolve()


def test_v2_cli_boundary_rejects_unbound_executable(tmp_path: Path) -> None:
    with pytest.raises(ValueError, match="v2 executable path must be"):
        require_bound_executable(tmp_path / "other.exe")


def test_v2_cli_boundary_accepts_readiness_bound_executable() -> None:
    assert require_bound_executable(EXECUTABLE) == EXECUTABLE.resolve()


def test_terminal_failure_closes_ledger_without_completion() -> None:
    ledger = new_ledger()
    key = expected_next(ledger, "preflight")

    close_ledger_failure(ledger, key, "preflight", "runner returned 9")

    assert ledger["status"] == "failed"
    assert ledger["failures"] == [
        {"key": "NH:wilson", "phase": "preflight", "reason": "runner returned 9"}
    ]
    assert ledger["completed"]["preflight"] == []
    assert expected_next(ledger, "preflight") is None


def test_v2_process_creation_delegates_to_admission_boundary(tmp_path: Path) -> None:
    observed = {}

    def fake_launcher(**kwargs):
        observed.update(kwargs)
        return 23

    def forbidden_direct_run(*args, **kwargs):
        raise AssertionError("test callback should be passed through, not invoked directly")

    result = capacity_admitted_launch(
        package=tmp_path,
        ledger_path=tmp_path / "ledger.json",
        admission_path=tmp_path / "admission.json",
        run_command=["runner", "--frozen"],
        monitored_run=forbidden_direct_run,
        launcher=fake_launcher,
    )

    assert result == 23
    assert observed["package"] == tmp_path
    assert observed["admission_record"] == tmp_path / "admission.json"
    assert observed["command"] == ["runner", "--frozen"]
    assert observed["run"] is forbidden_direct_run

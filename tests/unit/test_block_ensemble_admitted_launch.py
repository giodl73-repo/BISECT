import json
from collections import namedtuple
from pathlib import Path
import sys

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from check_block_ensemble_host_capacity import GIB
from launch_block_ensemble_admitted import launch_if_admitted


Usage = namedtuple("Usage", "total used free")


def write_ledger(path: Path, retained_bytes: int = 0) -> None:
    path.write_text(json.dumps({"retained_bytes": retained_bytes}), encoding="utf-8")


def test_rejection_is_recorded_without_process_creation(tmp_path: Path) -> None:
    ledger = tmp_path / "ledger.json"
    record = tmp_path / "admission.json"
    write_ledger(ledger)
    launched = False

    def forbidden_run(*args, **kwargs):
        nonlocal launched
        launched = True
        raise AssertionError("rejected launch created a process")

    returncode = launch_if_admitted(
        package=tmp_path,
        ledger_path=ledger,
        admission_record=record,
        command=["runner"],
        cwd=tmp_path,
        disk_usage=lambda path: Usage(20 * GIB, 13 * GIB, 7 * GIB),
        run=forbidden_run,
    )

    assert returncode == 1
    assert launched is False
    assert json.loads(record.read_text())["status"] == "reject"


def test_passing_record_exists_before_process_creation(tmp_path: Path) -> None:
    ledger = tmp_path / "ledger.json"
    record = tmp_path / "admission.json"
    write_ledger(ledger)

    def observed_run(command, cwd, check):
        report = json.loads(record.read_text())
        assert report["process_launch_authorized"] is True
        assert command == ["runner", "--future-protocol"]
        assert cwd == tmp_path.resolve()
        assert check is False
        return type("Completed", (), {"returncode": 17})()

    returncode = launch_if_admitted(
        package=tmp_path,
        ledger_path=ledger,
        admission_record=record,
        command=["runner", "--future-protocol"],
        cwd=tmp_path,
        disk_usage=lambda path: Usage(20 * GIB, 12 * GIB, 8 * GIB),
        run=observed_run,
    )

    assert returncode == 17
    assert json.loads(record.read_text())["status"] == "pass"


def test_existing_admission_record_fails_closed(tmp_path: Path) -> None:
    ledger = tmp_path / "ledger.json"
    record = tmp_path / "admission.json"
    write_ledger(ledger)
    record.write_text("custody", encoding="utf-8")

    with pytest.raises(FileExistsError, match="already exists"):
        launch_if_admitted(
            package=tmp_path,
            ledger_path=ledger,
            admission_record=record,
            command=["runner"],
            cwd=tmp_path,
        )

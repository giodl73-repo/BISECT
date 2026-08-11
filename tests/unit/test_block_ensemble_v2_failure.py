import json
from pathlib import Path
import shutil
import sys


PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from measure_block_ensemble_resources import sha256
from run_block_ensemble_expansion_v2 import PACKAGE
from verify_block_ensemble_v2_failure import verify_terminal_failure


def test_v2_terminal_failure_verifies_portably(tmp_path: Path) -> None:
    package = tmp_path / "package"
    shutil.copytree(PACKAGE, package)
    admission_path = package / "admission-preflight-nh-wilson-attempt-01.json"
    admission = json.loads(admission_path.read_text())
    admission["package_path"] = str(package.resolve())
    admission["ledger_path"] = str((package / "ledger.json").resolve())
    admission_path.write_text(json.dumps(admission), encoding="utf-8")
    resource_path = package / "resource-preflight-nh-wilson.json"
    resource = json.loads(resource_path.read_text())
    resource["admission_record_sha256"] = sha256(admission_path)
    resource_path.write_text(json.dumps(resource), encoding="utf-8")

    resource = verify_terminal_failure(package)

    assert resource["status"] == "fail"
    assert resource["returncode"] == 1
    assert resource["state"] == "NH"
    assert resource["sampler"] == "wilson"


def test_v2_failure_verifier_uses_retained_source_hash() -> None:
    resource = json.loads(
        (PACKAGE / "resource-preflight-nh-wilson.json").read_text(encoding="utf-8")
    )

    assert resource["runner_source_sha256"] != sha256(
        PROJECT_ROOT / "crates/bisect-ensemble/examples/block_trace.rs"
    )
    assert verify_terminal_failure()["runner_source_sha256"] == resource[
        "runner_source_sha256"
    ]

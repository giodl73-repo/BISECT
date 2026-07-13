#!/usr/bin/env python3
"""Verify the committed certified proof-backend prototype package."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical_hash(value: object) -> str:
    encoded = json.dumps(
        value, ensure_ascii=False, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    return f"sha256:{hashlib.sha256(encoded).hexdigest()}"


def main() -> None:
    root = Path("docs/examples/certified-proof-backend/path8-root")
    manifest = json.loads((root / "manifest.json").read_text(encoding="utf-8"))
    if manifest["schema_version"] != "certified-proof-backend-prototype-package-v1":
        raise SystemExit("unsupported certified proof prototype schema")
    declared = set(manifest["files"])
    actual = {
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_file() and path.name != "manifest.json"
    }
    if declared != actual:
        raise SystemExit("certified proof prototype inventory mismatch")
    for relative, expected_hash in manifest["files"].items():
        if sha256(root / relative) != expected_hash:
            raise SystemExit(f"certified proof prototype hash mismatch: {relative}")
    source_tree = Path(
        "docs/examples/certified-recursive/path8-k4/output/certified-bisection-tree.json"
    )
    if sha256(source_tree) != manifest["source_tree_sha256"]:
        raise SystemExit("certified proof source-tree hash mismatch")
    if sha256(Path(manifest["compiler_path"])) != manifest["compiler_sha256"]:
        raise SystemExit("certified proof compiler hash mismatch")
    if manifest["proof_generator_status"] != "smoke-verified-proof-not-bundled":
        raise SystemExit("certified proof generator posture drift")

    statuses: dict[str, list[str]] = {
        "optimal": [],
        "compact-optimal": [],
        "suboptimal": [],
        "compact-suboptimal": [],
    }
    for discovery_kind in statuses:
        for request_path in sorted((root / discovery_kind).glob("*.request.json")):
            request = json.loads(request_path.read_text(encoding="utf-8"))
            opb_path = request_path.with_name(request_path.name.replace(".request.json", ".opb"))
            if request["opb_sha256"] != f"sha256:{sha256(opb_path)}":
                raise SystemExit(f"OPB request hash mismatch: {request_path}")
            projection = {
                key: request[key]
                for key in (
                    "schema_version",
                    "instance_hash",
                    "discovery_id",
                    "stage",
                    "connectivity_encoding",
                    "exact_right_population",
                    "status",
                    "opb_sha256",
                    "variable_count",
                    "constraint_count",
                    "proof_format",
                    "proof_status",
                    "solver_command_template",
                    "claim",
                )
            }
            if request["request_id"] != canonical_hash(projection):
                raise SystemExit(f"proof request ID mismatch: {request_path}")
            statuses[discovery_kind].append(request["status"])
    if statuses["optimal"] != ["unsat-proof-required"] * 3:
        raise SystemExit("optimal proof requests are not all UNSAT claims")
    if statuses["compact-optimal"] != ["proof-required-unclassified"] * 3:
        raise SystemExit("compact optimal requests are not unclassified proof queries")
    if "sat-counterexample-exists" not in statuses["suboptimal"]:
        raise SystemExit("suboptimal discovery does not expose a SAT counterexample")
    if statuses["compact-suboptimal"] != ["proof-required-unclassified"] * 3:
        raise SystemExit("compact suboptimal requests are not unclassified proof queries")
    print("Certified proof backend verification: PASS")


if __name__ == "__main__":
    main()

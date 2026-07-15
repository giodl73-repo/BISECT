from __future__ import annotations

import importlib.util
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = (
    ROOT / "scripts/research/build_operational_recursive_tree_extended.py"
)
SPEC = importlib.util.spec_from_file_location("operational_tree_extended", MODULE_PATH)
assert SPEC and SPEC.loader
TREE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(TREE)


def test_extended_builder_forwards_larger_seed_frontier(
    monkeypatch, tmp_path: Path
) -> None:
    observed: list[int] = []

    def fake_floor_discovery(
        _bisect: Path,
        _context: Path,
        _districts: int,
        _out_dir: Path,
        _preferred_seed: int,
        _population_floor: int,
        max_seed: int,
    ) -> tuple[dict, int, list[dict]]:
        observed.append(max_seed)
        return {}, 1, []

    def fake_build(*_args: object) -> None:
        TREE.BASE.run_floor_discovery(
            tmp_path / "bisect",
            tmp_path / "context.rctx",
            2,
            tmp_path / "root",
            1,
            0,
            16,
        )

    monkeypatch.setattr(TREE.BASE, "run_floor_discovery", fake_floor_discovery)
    monkeypatch.setattr(TREE.BASE, "build", fake_build)
    monkeypatch.setattr(TREE, "rewrite_manifest_provenance", lambda *_args: None)

    TREE.build(
        tmp_path / "bisect",
        tmp_path / "context.rctx",
        tmp_path / "package",
        2,
        1,
        (2, 3),
        32,
    )

    assert observed == [32]
    assert TREE.BASE.run_floor_discovery is fake_floor_discovery


def test_extended_manifest_binds_base_and_frontier(tmp_path: Path) -> None:
    out_dir = tmp_path / "package"
    out_dir.mkdir()
    manifest_path = out_dir / "manifest.json"
    manifest_path.write_text(
        json.dumps({"builder_path": "old", "builder_sha256": "old"}),
        encoding="utf-8",
    )

    TREE.rewrite_manifest_provenance(out_dir, 32)

    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    assert manifest["builder_path"] == TREE.SCRIPT.as_posix()
    assert manifest["builder_sha256"] == TREE.BASE.sha256(ROOT / TREE.SCRIPT)
    assert manifest["base_builder_path"] == TREE.BASE_SCRIPT.as_posix()
    assert manifest["base_builder_sha256"] == TREE.BASE.sha256(
        ROOT / TREE.BASE_SCRIPT
    )
    assert manifest["seed_frontier_max"] == 32

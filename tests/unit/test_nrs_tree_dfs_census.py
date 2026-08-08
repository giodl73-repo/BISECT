from pathlib import Path
import sys


PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from run_nrs_tree_dfs_census import discovery_path, method_bool


def test_method_bool_reads_fallback_fields() -> None:
    method = (
        "prefix; nrs-v0-2-fallback-activated=true; "
        "nrs-v0-3-fallback-activated=false; suffix"
    )

    assert method_bool(method, "nrs-v0-2-fallback-activated")
    assert not method_bool(method, "nrs-v0-3-fallback-activated")


def test_discovery_path_maps_root_and_child() -> None:
    package = Path("package")

    assert discovery_path(package, "") == (
        package / "nodes/root/certified-discovery.json"
    )
    assert discovery_path(package, "010") == (
        package / "nodes/node-010/certified-discovery.json"
    )

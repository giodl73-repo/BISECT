"""Independently read retained LabelImport witnesses without running their code."""

import argparse
import hashlib
import json
from pathlib import Path
from zipfile import ZipFile


MAX = 18_446_744_073_709_551_615
JS_UNSAFE = 9_007_199_254_740_993
DIRECT = "runs/width_import/2020/"
BROWSER = "runs/browser_import/2020/"
WIDE_CASES = {
    "csv-beyond-wasm32": ("vermont", 4_294_967_296),
    "csv-beyond-js-integer": ("vermont", JS_UNSAFE),
    "csv-u64-max": ("vermont", MAX),
    "geojson-wide": ("vermont", MAX),
    "shapefile-character-wide": ("vermont", MAX),
    "context-alignment-refuses-large-district": ("washington", MAX),
}
ORDINARY = {"csv-control", "geojson-overflow-row-omitted"}
INVALID = {"csv-u64-overflow", "csv-negative", "csv-fraction", "shapefile-character-overflow"}


def unique(pairs):
    result = dict(pairs)
    assert len(pairs) == len(result), "duplicate JSON field"
    return result


def decode(data):
    return json.loads(data, object_pairs_hook=unique)


def read(path):
    return decode(path.read_bytes())


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def archived(path):
    with ZipFile(path) as archive:
        members = archive.infolist()
        assert 0 < len(members) < 100 and sum(m.file_size for m in members) < 1_000_000
        assert len({m.filename for m in members}) == len(members)
        return {m.filename: archive.read(m) for m in members}


def district_values(raw):
    entries = decode(raw)
    assert entries and all(type(value) is int and 0 <= value <= MAX for value in entries.values())
    return entries


def compare(name, candidate, native, clock):
    if name == ".bisect":
        assert decode(candidate) == decode(native), name
    elif name.endswith("/index.json"):
        a, b = map(decode, (candidate, native))
        assert a.pop("created") == clock and b.pop("created")
        assert a == b, name
    else:
        assert candidate == native, name


def direct(root, image):
    evidence = read(root / "evidence.json")
    assert evidence["status"] == "passed-native-label-import-width"
    assert evidence["wasm_sha256"] == image["wasm_sha256"]
    native = evidence["native"]
    assert sha(Path(native["path"])) == native["sha256"]
    rows = {r["id"]: r for r in evidence["rows"]}
    assert set(rows) == set(WIDE_CASES) | ORDINARY | INVALID and len(evidence["rows"]) == len(rows)
    checked = {}
    for name, record in rows.items():
        folder = root / name
        assert not record["legacy_width_refusal"]
        assert record["native_exit"] == int(name in INVALID)
        if name in INVALID:
            assert not (folder / (DIRECT + "index.json")).exists()
            assert not (folder / "package.zip").exists()
            assert not (folder / "result.json").exists()
            assert (folder / "native.stderr").stat().st_size
            continue
        result = read(folder / "result.json")
        assert result["execution_performed"] is False
        files = archived(folder / "package.zip")
        native_outputs = {".bisect"} | {
            p.relative_to(folder).as_posix() for p in (folder / "runs").rglob("*") if p.is_file()
        }
        assert set(files) == set(result["files"]) == native_outputs
        installs = [p for p in root.glob("reviewed-import-*")
                    if all((p / file).is_file() and (p / file).read_bytes() == data
                           for file, data in files.items())]
        assert installs, name
        for file, data in files.items():
            assert data == result["files"][file].encode("utf-8")
            compare(file, data, (folder / file).read_bytes(), "2026-11-06T00:00:00Z")
            if file.endswith("/assignments.json"):
                district_values(data)
        if name in WIDE_CASES:
            state, expected = WIDE_CASES[name]
            assignments = district_values(files[f"{DIRECT}{state}/assignments.json"])
            assert expected in assignments.values()
            checked[name] = max(assignments.values())
        if name == "geojson-overflow-row-omitted":
            assert district_values(files[DIRECT + "vermont/assignments.json"]) == {"50001000200": 2}
    align = read(root / "context-alignment-refuses-large-district/alignment-refusal-request.json")
    assert align["operation"] == "prepare-label-import-plan" and align["input"]["state_name"] == "washington"
    assert align["input"]["expected_index_sha256"] == hashlib.sha256(
        archived(root / "context-alignment-refuses-large-district/package.zip")[DIRECT + "index.json"]
    ).hexdigest()
    return native["sha256"], checked


def browser(log, image, native, wide):
    root = Path(decode(log.read_text(encoding="utf-8").strip().splitlines()[-1])["root"])
    evidence = read(root / "evidence.json")
    assert evidence["wasm_sha256"] == image["wasm_sha256"] and evidence["native_sha256"] == native
    assert sha(root / "bisect.exe") == native
    assert evidence["actual_worker_opfs"] and evidence["confirmed_writes"] and evidence["explicit_continuation"]
    assert evidence["baseline_saved_open_inert"] and evidence["cancel_after_close"]
    assert evidence["portable_without_directory_api"] and evidence["os_picker_tested"] is False
    assert evidence["exact_wide_zip_and_workspace"] is wide and evidence["source_tree"] is False
    site = Path(image["site"]).resolve()
    assert Path(evidence["package"]).resolve() == site
    assets = {}
    for item in evidence["served_assets"]:
        asset = Path(item["path"]).resolve()
        assert asset.is_relative_to(site) and sha(asset) == item["sha256"]
        assert asset.stat().st_size == item["byte_length"]
        assets[asset.relative_to(site).as_posix()] = item["sha256"]
    assert assets["bisect_wasm.wasm"] == image["wasm_sha256"]
    assert assets["label-import-files.js"] == image["label_import_asset_sha256"]

    opfs = read(root / "actual-files.json")
    outcome = read(root / "native-outcome.json")
    assert outcome["status"] == 0 and set(opfs) == set(outcome["files"])
    for name, candidate in opfs.items():
        native_bytes = (root / "native" / name).read_bytes()
        assert native_bytes == outcome["files"][name].encode("utf-8"), name
        compare(name, candidate.encode("utf-8"), native_bytes, "2026-10-23T12:34:56Z")
    assert opfs["private.txt"] == "unlisted file preserved"
    assert opfs[BROWSER + "washington/assignments.json"] == "unrelated state preserved"
    saved = read(root / "reviewed-source-baseline.bisect")
    assert saved["operation"] == "import-label-assignments"
    assert "authority" not in saved["files"]["labelImportApplicationBaseline"]
    assert read(root / "interrupted.json")["status"] == "cancelled"
    expected = {"alabama": ("01001000100", JS_UNSAFE if wide else 0),
                "vermont": ("50001000100", MAX if wide else 4_294_967_295)}
    zipped = archived(root / "wide-package.zip") if wide else None
    project = read(root / "original.bisect")
    for state, (geoid, district) in expected.items():
        name = f"{BROWSER}{state}/assignments.json"
        assert district_values(opfs[name]) == {geoid: district}
        assert project["result"]["files"][name] == opfs[name]
        if zipped is not None:
            assert zipped[name] == opfs[name].encode("utf-8")
    return {"root": str(root), "assignment_ids": {k: v for k, (_, v) in expected.items()},
            "asset_count": len(assets), "workspace_files": len(opfs), "wide_zip": wide}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    candidate = read(args.build / "status.json")
    assert candidate["phase"] == "passed-native-label-import-width-and-both-staged-profiles"
    profiles = {}
    for profile in ("core", "c"):
        folder = args.build / profile
        image = candidate["outputs"][profile]
        assert read(folder / "image.json") == image
        assert sha(Path(image["site"]) / "bisect_wasm.wasm") == image["wasm_sha256"]
        native, checked = direct(folder / "direct", image)
        profiles[profile] = {
            "wasm": image["wasm_sha256"], "native": native, "direct_cases": 12,
            "direct_literal_maxima": checked,
            "browser_wide": browser(folder / "browser-wide.log", image, native, True),
            "browser_ordinary": browser(folder / "browser-ordinary.log", image, native, False),
        }
    with args.output.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump({"status": "passed-independent-native-label-import-width-readback",
                   "profiles": profiles, "executed_witnesses": 0, "full_parity": False}, stream, indent=2)
        stream.write("\n")
    print("Read back both profiles’ actual CLI files, selected-root application, and portable downloads.")


if __name__ == "__main__":
    main()

"""Independently verify native graph components and the browser download."""

import argparse
import csv
import hashlib
import json
from pathlib import Path
from zipfile import ZipFile


LABEL = "context_fixture"
RUN = f"runs/{LABEL}/2020"
ANALYSIS = f"analysis/{LABEL}/2020"
REPORT = f"{ANALYSIS}/washington/contiguity.json"
POSITIVES = {"connected-one-based": 1, "connected-zero-based": 0, "disconnected-one-based": 1}
REFUSALS = {"wrong-basis", "missing-context-unit", "wide-district"}


def unique(pairs):
    values = dict(pairs)
    assert len(values) == len(pairs), "Duplicate JSON fields"
    return values


def read(path):
    return json.loads(path.read_bytes(), object_pairs_hook=unique)


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def oracle(context, assignments, offset):
    ids = context["units"]["unit_ids"]
    assert len(ids) == len(set(ids)) and set(assignments) == set(ids)
    adjacency = [set() for _ in ids]
    for u, row in enumerate(context["graph"]["adjacency"]):
        for edge in row:
            v = edge["to"]
            adjacency[u].add(v)
            adjacency[v].add(u)
    districts = sorted(set(assignments.values()))
    assert districts == list(range(offset, offset + len(districts)))
    expected = []
    for ordinal, label in enumerate(districts, 1):
        remaining = {i for i, name in enumerate(ids) if assignments[name] == label}
        count = len(remaining)
        components = []
        while remaining:
            queue = [min(remaining)]
            remaining.remove(queue[0])
            for u in queue:
                for v in sorted(adjacency[u] & remaining):
                    remaining.remove(v)
                    queue.append(v)
            components.append(sorted(queue))
        components.sort(key=lambda c: (-len(c), c[0]))
        disconnected = [ids[i] for c in components[1:] for i in c]
        expected.append({"district": ordinal, "contiguous": len(components) == 1,
                         "tract_count": count, "component_count": len(components),
                         "disconnected_tracts": disconnected})
    return expected


def audit(native, browser, site):
    n, b = read(native / "evidence.json"), read(browser / "evidence.json")
    assert n["status"] == "passed-native-context-label-contiguity"
    assert b["status"] == "passed-actual-browser-context-label-contiguity"
    assert sha(Path(n["native"]["path"])) == n["native"]["sha256"] == b["native_sha256"]
    assert sha(site / "bisect_wasm.wasm") == n["wasm"]["sha256"] == b["wasm_sha256"]
    assert b["inert_open"] and b["actual_worker"] and b["exact_native_zip_report_and_index"]
    assert b["os_picker_tested"] is False
    for name, asset in b["served"].items():
        path = (site / name).resolve()
        assert path.is_relative_to(site.resolve())
        assert path.stat().st_size == asset["bytes"] and sha(path) == asset["sha256"]
    results = {r["id"]: r for r in n["outcomes"]}
    downloads = {r["id"]: r for r in b["cases"]}
    assert len(results) == len(n["outcomes"]) == 6 and set(results) == set(POSITIVES) | REFUSALS
    assert set(downloads) == set(POSITIVES) and len(b["cases"]) == len(downloads)
    proven = {}
    for name, offset in POSITIVES.items():
        p = native / name
        report, index = p / REPORT, p / ANALYSIS / "index.json"
        selected = p / RUN / "washington/assignments.json"
        with (p / "supplied.csv").open(encoding="utf-8", newline="") as stream:
            original = {r["GEOID"]: int(r["district"]) for r in csv.DictReader(stream)}
        assignments = read(selected)
        assert assignments == original
        expected = oracle(read(p / "context.rctx"), assignments, offset)
        actual = read(report)
        assert [{k: row[k] for k in e} for row, e in zip(actual["districts"], expected, strict=True)] == expected
        assert all(len(r["disconnected_tract_counties"]) == len(r["disconnected_tracts"]) for r in actual["districts"])
        assert actual["all_contiguous"] == all(e["contiguous"] for e in expected) == results[name]["contiguous"]
        assert read(index)["run_index_sha256"] == sha(p / RUN / "index.json")
        assert read(index)["states"] == {"washington": {"status": "ok", "contiguity": "ok"}}
        assert read(p / ".bisect")[LABEL]["analyzed"] == ["2020"]
        downloaded = browser / (name + ".zip")
        assert sha(downloaded) == downloads[name]["zip_sha256"]
        assert sha(report) == downloads[name]["report_sha256"] == results[name]["report_sha256"]
        assert sha(index) == downloads[name]["analysis_index_sha256"]
        saved = read(browser / (name + "-saved.bisect"))
        assert saved["result"]["files"][REPORT].encode() == report.read_bytes()
        with ZipFile(downloaded) as z:
            assert len(z.namelist()) == len(set(z.namelist())) and z.testzip() is None
            assert z.read(REPORT) == report.read_bytes()
            assert z.read(f"{ANALYSIS}/index.json") == index.read_bytes()
        proven[name] = {"report_sha256": sha(report), "components": [e["component_count"] for e in expected]}
    for name in REFUSALS:
        p = native / name
        assert results[name]["status"] == "matching-refusal-no-analysis-effects"
        assert read(p / "desktop-analysis.json")["status"] == 1 and not (p / "analysis").exists()
        assert read(p / ".bisect")[LABEL] == {"built": ["2020"], "analyzed": [], "reported": []}
    return {"status": "passed-independent-context-contiguity-readback", "cases": proven,
            "refusals": sorted(REFUSALS), "native_sha256": b["native_sha256"], "wasm_sha256": b["wasm_sha256"],
            "browser": b["browser"], "full_parity": False}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("native", "browser", "site", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    result = audit(args.native, args.browser, args.site)
    with args.output.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(result, stream, indent=2)
        stream.write("\n")
    print(result["status"])

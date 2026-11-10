"""Independently check native prepared-plan ZIP bytes and retained source bindings."""
import base64
import hashlib
import json
from pathlib import Path
import sys
import zipfile

root = Path("target/label-import-plan-browser" if "--browser" in sys.argv else "target/label-import-plan-wasm")
checked = 0
digest = lambda value: hashlib.sha256(value).hexdigest()
for archive in root.glob("*.zip"):
    project = json.loads(archive.with_suffix(".bisect").read_text(encoding="utf-8"))
    result = project["result"]
    assert result["schema_version"] == "bisect-label-import-plan-v1"
    source = project["files"]["labelImportInputs"]
    selection = project["files"]["labelImportContext"]
    with zipfile.ZipFile(archive) as zipped:
        assert len(zipped.infolist()) == 5
        assert set(zipped.namelist()) == set(result["files"]) == {
            "imported.rplan", "context.rctx", "import-index.json", "imported-assignments.json", "import-record.json"
        }
        assert zipped.testzip() is None
        for entry in zipped.infolist():
            assert entry.filename == Path(entry.filename).name
            value = zipped.read(entry)
            assert value == result["files"][entry.filename].encode("utf-8")
            assert digest(value) == result["file_sha256"][entry.filename]
            checked += 1
        context_bytes = zipped.read("context.rctx")
        assert context_bytes == base64.b64decode(selection["context"]["base64"], validate=True)
        context = json.loads(context_bytes)
        plan = json.loads(zipped.read("imported.rplan"))
        assert plan["plan"]["units"] == context["units"]
        record = json.loads(zipped.read("import-record.json"))
        assert record == result["bindings"] == plan["extensions"]["org.bisect.label-import"]
        for key in ("source", "dbf", "shx"):
            if source[key] is not None:
                assert record[key + "_sha256"] == digest(base64.b64decode(source[key]["base64"], validate=True))
        assert record["context_sha256"] == digest(context_bytes)
        assert record["index_sha256"] == selection["index_sha256"] == digest(zipped.read("import-index.json"))
        assert record["assignments_sha256"] == digest(zipped.read("imported-assignments.json"))
        assignments = json.loads(zipped.read("imported-assignments.json"))
        offset = 1 if selection["district_basis"] == "one-based" else 0
        assert plan["plan"]["assignment"] == [assignments[unit] - offset for unit in context["units"]["unit_ids"]]
        assert set(plan["plan"]["assignment"]) == set(range(plan["plan"]["k"]))
        assert plan["provenance"]["producer"] == {"method": "external-label-import-context-alignment"}
        assert result["execution_performed"] is False
assert checked > 0
print(f"Imported-plan {'browser' if '--browser' in sys.argv else 'direct'} ZIPs: {checked} exact files, CRCs, source/context/index bindings and native district order passed.")

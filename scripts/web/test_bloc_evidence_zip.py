"""Verify the produced evidence archive without extracting or executing anything."""
import base64
import hashlib
import json
from pathlib import Path
from zipfile import ZipFile
from jsonschema import Draft202012Validator

root = Path("target/bloc-analysis-wasm")
fixture = json.loads((root / "fixture.json").read_text(encoding="utf-8"))
request = fixture["base"]
schema = json.loads(Path("crates/bisect-analysis/schemas/bloc_voting.schema.json").read_text(encoding="utf-8"))
validator = Draft202012Validator(schema)
validator.validate(fixture["output"]["report"])
for path in root.glob("native-*/reports/bloc_voting.json"):
    validator.validate(json.loads(path.read_text(encoding="utf-8")))
with ZipFile(root / "evidence.zip") as archive:
    assert archive.testzip() is None
    names = archive.namelist()
    assert len(names) == len(set(names)) == 6
    assert all(not name.startswith("/") and ".." not in name.split("/") for name in names)
    assert archive.read("inputs/precincts.tsv") == request["precincts_tsv"].encode("utf-8")
    assert archive.read("inputs/annotations/" + request["annotation_source_name"]) == request["annotation_csv"].encode("utf-8")
    record = json.loads(archive.read("run-record.json"))
    validator.validate(record["report"])
    assert record["report"]["bootstrap"]["seed"] == "18446744073709551615"
    for name, text in record["files"].items():
        actual = archive.read("reports/" + name)
        assert actual == text.encode("utf-8")
        assert hashlib.sha256(actual).hexdigest() == record["file_sha256"][name]
    for name, encoded in request["attestation_files"].items():
        actual = archive.read("inputs/annotations/" + name)
        assert actual == base64.b64decode(encoded, validate=True)
        assert hashlib.sha256(actual).hexdigest() == record["attestation_sha256"][name]
    assert all(info.date_time == (1980, 1, 1, 0, 0, 0) for info in archive.infolist())
    assert all((info.external_attr >> 16) == 0o100644 for info in archive.infolist())
print("Bloc evidence ZIP: all source/report/document bytes and hashes, maximum-u64 seed, CRCs, safe paths and fixed non-executable permissions verified.")

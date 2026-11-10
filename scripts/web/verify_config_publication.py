"""Read back the Config publication checkpoint and its inherited CLI identities."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        while chunk := stream.read(1024 * 1024):
            h.update(chunk)
    return h.hexdigest()


def main():
    verified, sources, closures = {}, {}, []
    for name in ['target/config-cli-contract/closure.json', 'target/config-publication/closure.json']:
        path = ROOT / name
        data = json.loads(path.read_text(encoding='utf-8'))
        for row in data['files']:
            ref, source = row['snapshot'], row['source']
            blob = (ROOT / ref['path']).resolve()
            assert blob.is_relative_to(ROOT) and blob.is_file()
            assert source['sha256'] == ref['sha256'] and source['byte_length'] == ref['byte_length']
            if blob not in verified:
                assert blob.stat().st_size == ref['byte_length'] and digest(blob) == ref['sha256']
                verified[blob] = ref['sha256']
            assert verified[blob] == ref['sha256']
            sources[source['path'], source['sha256']] = ref
        closures.append({'path': name, 'sha256': digest(path), 'mappings': len(data['files'])})
    runtime = []
    for name in ['run-dXAXKG', 'run-gfSL1V']:
        data = json.loads((ROOT / 'target/config-cli-contract' / name / 'evidence.json').read_text(encoding='utf-8'))
        for key in ['native', 'wasm']:
            ref = data[key]
            retained = sources[ref['path'], ref['sha256']]
            runtime.append({'original': ref, 'retained': retained})
    record = dict(status='passed-checkpoint-and-inherited-CLI-readback', closures=closures,
                  runtime_identities=runtime, verified_blob_paths=len(verified),
                  current_sources_replaced_by_historical=False, actual_os_picker_grant=False,
                  whole_builder_passed=False, full_parity=False, deployed=False,
                  verifier_sha256=digest(Path(__file__)))
    out = ROOT / 'target/config-publication/readback.json'
    if out.exists():
        assert json.loads(out.read_text(encoding='utf-8')) == record
    else:
        out.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'path': out.relative_to(ROOT).as_posix(), 'sha256': digest(out),
                      'verified_blob_paths': len(verified), 'status': record['status']}))


if __name__ == '__main__':
    main()

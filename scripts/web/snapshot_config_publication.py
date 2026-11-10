"""Freeze bounded Config publication evidence without repinning older receipts."""
import hashlib
import json
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'target/config-publication'
PACKAGE = ROOT / 'dist/wasm-config-publication-final'
CASES = ['publication-0M80aF', 'publication-Btq9Kh',
         'publication-browser-Pj4aCS', 'publication-browser-9Ti8KI']


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        while chunk := stream.read(1024 * 1024):
            h.update(chunk)
    return h.hexdigest()


def main():
    destination = OUT / 'closure.json'
    if destination.exists():
        raise SystemExit('Existing closure is immutable; select a new checkpoint.')
    files = set()
    evidence = []
    for name in CASES:
        folder = ROOT / 'target/config-cli-contract' / name
        path = folder / 'evidence.json'
        data = json.loads(path.read_text(encoding='utf-8'))
        assert data['status'].startswith('passed-config-publication-')
        assert len(data.get('rows', data.get('outcomes'))) == (7 if 'browser' in name else 17)
        for ref in data['sources']:
            source = ROOT / ref['path']
            assert digest(source) == ref['sha256'], ref['path']
            files.add(source)
        if 'test_sha256' in data:
            assert digest(ROOT / 'scripts/web/test_config_publication_browser.mjs') == data['test_sha256']
            assert data['actual_os_picker_grant'] is False
        reference = ROOT / data['reference']['path']
        assert digest(reference) == data['reference']['sha256']
        files.update(p for p in reference.parent.rglob('*') if p.is_file())
        files.update(p for p in folder.rglob('*') if p.is_file())
        evidence.append(path.relative_to(ROOT).as_posix())
    manifest = json.loads((PACKAGE / 'offline-manifest.json').read_text(encoding='utf-8'))
    assert len(manifest['assets']) == 729
    for name, ref in manifest['assets'].items():
        path = PACKAGE / name
        assert path.stat().st_size == ref['bytes'] and digest(path) == ref['sha256'], name
        files.add(path)
    files.add(PACKAGE / 'offline-manifest.json')
    for name in ['README.md', 'docs/specs/wasm-config-publication.md',
                 'docs/specs/wasm-config-cli-contract.md', 'docs/specs/wasm-desktop-parity-plan.md',
                 'docs/specs/wasm-capability-inventory.json', 'docs/specs/wasm-engine-option-inventory.json',
                 'docs/specs/wasm-practitioner-actions.json', 'docs/specs/wasm-fetch-source-coverage.json',
                 'design/pitfalls/README.md', 'design/pitfalls/pitfalls-pipeline.md',
                 'signals/roles/check/wasm-config-publication-roles-check-2026-10-27.md',
                 'scripts/web/snapshot_config_publication.py', 'scripts/web/test_config_publication.mjs',
                 'scripts/web/test_config_publication_browser.mjs', 'scripts/web/check_cffi_legacy_gates.ps1',
                 'scripts/web/build_cffi_preview.ps1', 'crates/bisect-cli/src/algo_config.rs']:
        files.add(ROOT / name)
    files.update(p for p in OUT.rglob('*') if p.is_file() and 'snapshot' not in p.parts)
    fetch_log = (OUT / 'fetch.log').read_text(encoding='utf-8-sig').strip()
    fetch = json.loads(fetch_log)
    assert fetch['cases'] == fetch['native_plan_comparisons'] == fetch['native_handoff_comparisons'] == 42
    files.update(p for p in Path(fetch['evidence']).parent.rglob('*') if p.is_file())
    for name in ['ledger', 'inventory', 'project', 'offline', 'assets']:
        assert (OUT / (name + '.log')).stat().st_size > 0
    stores = [ROOT / p for p in ['target/native-format-cli/full-build/snapshot-scope/blobs',
              'target/native-format-cli/snapshot', 'target/fletch-cli-contract/snapshot',
              'target/ceremony-source-start/snapshot-final', 'target/config-cli-contract/snapshot']]
    previous = json.loads((ROOT / 'target/config-cli-contract/closure.json').read_text(encoding='utf-8'))
    reusable = {r['snapshot']['sha256']: ROOT / r['snapshot']['path'] for r in previous['files']}
    blobs = OUT / 'snapshot'
    blobs.mkdir(exist_ok=True)
    rows, verified, new_bytes = [], {}, 0
    for path in sorted(files):
        identity, size = digest(path), path.stat().st_size
        blob = verified.get(identity)
        if blob is None:
            choices = [reusable.get(identity), *(p / identity for p in stores), blobs / identity]
            blob = next((p for p in choices if p and p.is_file() and p.stat().st_size == size
                         and digest(p) == identity), None)
            if blob is None:
                blob = blobs / identity
                assert not blob.exists()
                shutil.copyfile(path, blob)
                assert digest(blob) == identity
                new_bytes += size
            verified[identity] = blob
        assert blob.stat().st_size == size
        rows.append({'source': {'path': path.relative_to(ROOT).as_posix(), 'sha256': identity,
                                'byte_length': size},
                     'snapshot': {'path': blob.relative_to(ROOT).as_posix(), 'sha256': identity,
                                  'byte_length': size}})
    record = dict(status='passed-bounded-config-publication-checkpoint', files=rows,
                  accepted_evidence=evidence, offline_version=manifest['version'],
                  new_blob_bytes=new_bytes, actual_os_picker_grant=False,
                  new_whole_builder_passed=False, full_parent_acceptance=False, deployed=False,
                  scope='Selected-root Config publication. Owned Windows facade plus real browser OPFS APIs '
                        'with injected picker. Earlier receipts remain historical; no OS transaction/host claim.')
    destination.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'closure': destination.relative_to(ROOT).as_posix(), 'sha256': digest(destination),
                      'files': len(rows), 'new_blob_bytes': new_bytes}))


if __name__ == '__main__':
    main()

"""Freeze national correction evidence with explicit historical source versions and verified inheritance."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
from evidence_blob_store import ROOT, SnapshotBlobs, digest, local


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--record', required=True)
    parser.add_argument('--base', required=True)
    parser.add_argument('--base-sha256', required=True)
    parser.add_argument('--dry-run', action='store_true')
    parser.add_argument('--output', help='New snapshot directory below target/national-matrix/package-snapshots')
    parser.add_argument('--exclude-log', action='append', default=[], help='Declare an unbound live output log; required pinned evidence cannot be excluded')
    parser.add_argument('--reuse-copies', action='append', default=[], help='Reuse only digest-verified immutable copies from an earlier owned snapshot attempt')
    parser.add_argument('--completed-log', action='append', default=[], help='Retain an additional terminal producer log outside the captured trees')
    args = parser.parse_args()
    record = local(args.record)
    bundle = json.loads(record.read_text(encoding='utf-8'))
    assert bundle['status'] == 'passed' and bundle['deployed'] is False and bundle['full_parity'] is False
    assert all(s['exit_code'] == s['expected_exit_code'] for s in bundle['stages'])
    base = SnapshotBlobs(args.base, args.base_sha256)
    print('Verify inherited evidence', flush=True)
    base.verify()
    files = {}
    historical = []
    excluded_logs = {local(name) for name in args.exclude_log}
    assert all(p.is_relative_to(ROOT/'target') and p.suffix == '.log' for p in excluded_logs)
    completed_logs = {local(name) for name in args.completed_log}
    assert all(p.is_file() and p.is_relative_to(ROOT/'target') and p.suffix == '.log' for p in completed_logs)
    assert not (completed_logs & excluded_logs), 'A completed input cannot be the live output log'

    def include(name, expected=None, size=None, allow_historical=True):
        path = local(name)
        if path in excluded_logs:
            assert expected is None, 'A required evidence pin cannot be excluded'
            return
        identity = digest(path) if path.is_file() else None
        logical = path.relative_to(ROOT).as_posix()
        if expected is not None and identity != expected:
            assert allow_historical, 'Current delivery artifact changed: '+logical
            # Current source paths do not overwrite older execution identities.
            # The existing verified blob must supply those exact historical bytes.
            if expected in base.rows:
                path = base.blob(expected, size)
            else:
                retained = next((p for p,h,n in files.values() if h == expected and (size is None or n == size)), None)
                assert retained is not None, 'Missing historical bytes: '+logical+' / '+expected
                path = retained
            identity = expected
            logical = '__evidence_versions__/'+identity+'/'+logical
            historical.append(dict(original_path=local(name).relative_to(ROOT).as_posix(),sha256=identity,stored_path=logical))
        assert path.is_file(), str(path)
        length = path.stat().st_size
        if size is not None:
            assert length == size, 'Artifact size differs: '+str(path)
        if logical in files:
            assert files[logical][1:] == (identity,length), 'Conflicting unversioned evidence: '+logical
        files[logical] = (path,identity,length)

    def pins(value, allow_historical=True):
        if isinstance(value, dict):
            if isinstance(value.get('path'), str) and isinstance(value.get('sha256'), str):
                include(value['path'],value['sha256'],value.get('byte_length'),allow_historical)
            else:
                for nested in value.values():
                    pins(nested,allow_historical)
        elif isinstance(value, list):
            for nested in value:
                pins(nested,allow_historical)

    def tree(name):
        folder = local(name)
        assert folder.is_dir() and folder.is_relative_to(ROOT)
        for path in folder.rglob('*'):
            parts = path.relative_to(folder).parts
            if any(p.startswith('profile') or p.startswith('evidence-rollup-') or p=='package-snapshots' for p in parts):
                continue
            assert not path.is_symlink() and not path.is_junction(), 'Unexpected reparse point: '+str(path)
            if path.is_file():
                include(path)

    seen_roots = set()
    def roots(value):
        if isinstance(value, dict):
            if isinstance(value.get('root'), str):
                try:
                    folder = local(value['root'])
                except ValueError:
                    folder = None
                if folder and folder.is_dir() and folder.is_relative_to(ROOT/'target') and folder not in seen_roots:
                    seen_roots.add(folder)
                    tree(folder)
            for nested in value.values():
                roots(nested)
        elif isinstance(value, list):
            for nested in value:
                roots(nested)

    include(record)
    for log in completed_logs:
        include(log)
    pins(bundle['source_files'],False)
    pins(bundle['package_files'],False)
    pins(bundle['stages'],False)
    pins(bundle['national_witnesses'],False)
    pins(bundle['engine_record'],False)
    engines = json.loads(local(bundle['engine_record']['path']).read_text(encoding='utf-8'))
    pins(engines,False)
    roots(engines)
    tree(record.parent)
    tree(local(bundle['engine_record']['path']).parent)
    for stage in bundle['stages']:
        for line in local(stage['log']['path']).read_text(encoding='utf-8').splitlines():
            if line.startswith('{'):
                try:
                    roots(json.loads(line))
                except json.JSONDecodeError:
                    pass
    for profile in ['default','c-sidecar']:
        folder = ROOT/'target/cffi-legacy-gates'/profile
        gate = json.loads((folder/'build.json').read_text(encoding='utf-8-sig'))
        assert gate['status'] == 'passed'
        pins(gate['source_files'])
        roots(gate)
        tree(folder)
    for name in ['target/cffi-legacy-gates/state-core-reference.exe','target/cffi-legacy-gates/default.wasm','target/cffi-legacy-gates/c-sidecar.wasm']:
        include(name)
    tree('target/national-matrix')
    tree('target/full-chamber-matrix')
    for native_record in (ROOT/'target/national-matrix').glob('map-engine-*/evidence.json'):
        pins(json.loads(native_record.read_text(encoding='utf-8')))
    matrix = json.loads((ROOT/'docs/specs/wasm-real-data-matrix.json').read_text(encoding='utf-8'))
    pins(matrix.get('national_campaigns',[]))
    pins(matrix.get('followup_campaigns',[]))
    for root in ['target/generated-report-preview-browser','target/commission-preview-browser','target/tui-verify-browser','target/native-output-verification-browser','target/native-workspace-browser','target/explicit-c-state-batch-browser','target/cffi-engine-browser','target/tui-verify-wasm','target/explicit-c-state-batch']:
        if (ROOT/root).is_dir():
            tree(root)
    for log in (ROOT/'target/build-application').glob('national-*.log'):
        include(log)
    for log in (ROOT/'target/build-application').glob('*large-context*.log'):
        include(log)
    include('target/build-application/accepted-context-3mib-refusal.log')
    for name in ['README.md','docs/specs/wasm-desktop-parity-plan.md','docs/specs/wasm-real-data-matrix.json','docs/specs/wasm-national-matrix.md','docs/specs/wasm-national-browser-capacity.md','docs/specs/wasm-national-retained-map.md','docs/specs/wasm-large-context-followup.md','docs/specs/wasm-capability-inventory.json','docs/specs/wasm-engine-option-inventory.json','docs/specs/wasm-practitioner-actions.json','docs/specs/wasm-fetch-source-coverage.json','scripts/web/validate_national_browser_package.py','scripts/web/snapshot_national_browser_delivery.py','scripts/web/rollup_evidence_snapshot.py','scripts/web/evidence_blob_store.py','scripts/web/test_state_batch_zip.mjs','scripts/web/test_national_matrix_browser.mjs','scripts/web/test_national_batch_zip.mjs','scripts/web/test_national_saved_refusal_browser.mjs','scripts/web/test_national_matrix_map.mjs','scripts/web/test_national_matrix_map_browser.mjs','scripts/web/build_wasm.ps1','scripts/web/build_wasm_lab.ps1','scripts/web/check_cffi_legacy_gates.ps1','scripts/web/wasm_practitioner_inventory.py','design/pitfalls/pitfalls-pipeline.md','design/pitfalls/README.md']:
        include(name)
    rows, blobs = [], {}
    for name,(path,identity,length) in sorted(files.items()):
        inherited = identity in base.rows
        if inherited:
            base.blob(identity,length)
        else:
            blobs[identity] = path
        rows.append(dict(path=name,sha256=identity,byte_length=length,blob=identity,storage='base' if inherited else 'local'))
    reuse_roots = [local(p) for p in args.reuse_copies]
    assert all(p.is_dir() and p.is_relative_to(ROOT/'target/national-matrix/package-snapshots') for p in reuse_roots)
    reusable, rejected = {}, []
    for identity,path in blobs.items():
        for folder in reuse_roots:
            copied = folder/'blobs'/identity
            if not copied.exists():
                continue
            assert not copied.is_symlink() and not copied.is_junction()
            if copied.stat().st_size != path.stat().st_size or digest(copied) != identity:
                rejected.append(dict(path=copied.relative_to(ROOT).as_posix(),expected_sha256=identity,status='rejected-incomplete-or-different-copy'))
                continue
            reusable[identity] = copied
            break
    needed = sum(p.stat().st_size for h,p in blobs.items() if h not in reusable)
    print(json.dumps(dict(files=len(rows),unique_blobs=len({r['sha256'] for r in rows}),new_blobs=len(blobs),new_logical_bytes=sum(p.stat().st_size for p in blobs.values()),reused_verified_blobs=len(reusable),remaining_copy_bytes=needed,historical_versions=len(historical),available_bytes=shutil.disk_usage(ROOT).free)),flush=True)
    if args.dry_run:
        return
    assert shutil.disk_usage(ROOT).free >= needed+1024*1024*1024, 'Insufficient evidence capacity including 1 GiB validation-host reserve; no old evidence is deleted'
    destination = local(args.output) if args.output else ROOT/'target/national-matrix/package-snapshots'/Path(bundle['package']).name
    assert destination.is_relative_to(ROOT/'target/national-matrix/package-snapshots')
    assert not destination.exists(), 'Preserve previous snapshots'
    (destination/'blobs').mkdir(parents=True)
    if os.name == 'nt':
        subprocess.run(['compact.exe','/C','/I','/Q',str(destination)],check=True,capture_output=True)
        subprocess.run(['compact.exe','/C','/I','/Q',str(destination/'blobs')],check=True,capture_output=True)
    for identity,path in blobs.items():
        target = destination/'blobs'/identity
        if identity in reusable:
            # Only completed, digest-verified immutable copy objects qualify.
            # Mutable raw inputs are never linked into evidence storage.
            os.link(reusable[identity],target)
        else:
            shutil.copyfile(path,target)
        assert digest(target) == identity, 'Evidence copy differs: '+str(path)
    data = dict(schema_version='bisect-national-browser-snapshot-v1',storage_format='verified-inherited-blobs-v1',base_snapshot=dict(path=base.path.relative_to(ROOT).as_posix(),sha256=args.base_sha256),package=bundle['package'],default_package=bundle['default_package'],record=dict(path=record.relative_to(ROOT).as_posix(),sha256=digest(record)),files=rows,historical_versions=historical,excluded_live_logs=[p.relative_to(ROOT).as_posix() for p in sorted(excluded_logs)],reused_verified_copy_roots=[p.relative_to(ROOT).as_posix() for p in reuse_roots],rejected_reuse_copies=rejected,scope=bundle['scope'],full_parity=False,deployed=False)
    snapshot = destination/'snapshot.json'
    snapshot.write_text(json.dumps(data,indent=2),encoding='utf-8')
    identity = digest(snapshot)
    checked = SnapshotBlobs(snapshot,identity).verify()
    print(json.dumps(dict(path=snapshot.relative_to(ROOT).as_posix(),sha256=identity,byte_length=snapshot.stat().st_size,verification=checked)),flush=True)


if __name__ == '__main__':
    main()

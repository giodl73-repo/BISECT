"""Freeze reviewed import package/evidence with verified inherited blob storage."""
import argparse
import json
import shutil
from pathlib import Path
from evidence_blob_store import ROOT, local, digest, SnapshotBlobs


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--record', required=True)
    parser.add_argument('--dry-run', action='store_true')
    args = parser.parse_args()
    record = local(args.record)
    bundle = json.loads(record.read_text(encoding='utf-8'))
    assert bundle['browser_status'] == 'passed' and bundle['deployed'] is False and bundle['full_parity'] is False
    assert all(stage['exit_code'] == 0 for stage in bundle['stages'])
    inventory = json.loads((ROOT/'docs/specs/wasm-capability-inventory.json').read_text(encoding='utf-8'))
    child = next(c for e in inventory['entries'] if e['id']=='bisect.Mv' for c in e['parity_record']['contracts'] if c['contract_id'].endswith('/reviewed-rename-application'))
    reference = child['implementation_evidence']['immutable_local_snapshot']
    base = SnapshotBlobs(reference['path'], reference['sha256'])
    base.verify()
    package = local(bundle['package'])
    assert package.is_relative_to(ROOT/'dist')
    destination = ROOT/'target/label-import-application/package-snapshots'/package.name
    assert not destination.exists(), 'Preserve prior snapshots'
    files = {}

    def include(name, expected=None, size=None):
        path = local(name)
        assert path.is_file(), str(path)
        identity, length = digest(path), path.stat().st_size
        if expected is not None:
            assert identity == expected, 'Changed artifact: '+str(path)
        if size is not None:
            assert length == size
        files[path.relative_to(ROOT).as_posix()] = (path,identity,length)

    def pins(value):
        if isinstance(value,dict):
            if 'path' in value and 'sha256' in value:
                include(value['path'],value['sha256'],value.get('byte_length'))
            else:
                for nested in value.values():
                    pins(nested)
        elif isinstance(value,list):
            for nested in value:
                pins(nested)

    def tree(folder):
        directory = local(folder)
        assert directory.is_dir()
        for path in directory.rglob('*'):
            relative = path.relative_to(directory)
            if any(part.startswith('profile') or part=='package-snapshots' for part in relative.parts):
                continue
            if path.is_file():
                include(path)

    include(record)
    for key in ['engine_record','native_cli','native_host','source_files','stages','offline_manifest']:
        pins(bundle[key])
    tree(package)
    tree(record.parent)
    # Includes original source/binary witnesses, failed extraction attempts,
    # raw native replies, genuine PDF cases and owned browser captures.
    tree('target/label-import-application')
    tree('target/label-import-application-browser')
    engines = json.loads(local(bundle['engine_record']['path']).read_text(encoding='utf-8'))
    pins(engines['source_files'])
    for profile in engines['profiles'].values():
        pins(profile)
        for kind in ['direct','batch','rename']:
            tree(profile[kind]['root'])
    for name, gate in bundle['legacy_gates'].items():
        pins(gate['record'])
        pins(gate['wasm'])
        folder = local(gate['record']['path']).parent
        evidence = json.loads((folder/'build.json').read_text(encoding='utf-8-sig'))
        assert evidence['status']=='passed'
        pins(evidence['source_files'])
        imported=evidence['label_import_evidence']
        assert imported['status']=='passed' and imported['browser_status']=='passed'
        tree(imported['node']['root'])
        tree(imported['browser']['root'])
        rename=evidence['label_rename_evidence']
        assert rename['status']=='passed' and rename['browser_status']=='passed'
        tree(rename['node']['root'])
        tree(rename['browser']['root'])
        include(folder.parent/'state-core-reference.exe',evidence['native_oracle_sha256'])
        tree(folder)
        for key in ['direct','batch','browser']:
            tree(evidence['native_summary_evidence'][key]['root'])
        if evidence.get('post_build_native_summary_browser_revalidation'):
            pins(evidence['post_build_native_summary_browser_revalidation'])
        tree(evidence['label_repository_evidence']['root'])
        tree(evidence['fetch_archive_evidence']['root'])
        pdf = evidence['report_pdf_evidence']
        assert pdf['status']=='passed'
        tree(pdf['root'])
    for key in ['final_browser','rename_browser','summary_browser','pdf_browser','count_browser','label_browser']:
        folder = local(bundle[key]['root'])
        tree(folder)
        evidence = json.loads((folder/'evidence.json').read_text(encoding='utf-8'))
        assert evidence['source_tree'] is False and local(evidence.get('package',evidence.get('static_root')))==package/'toolkit'
        pins(evidence['served_assets'])
    # Preserve the exact current-runtime fixture producer outputs as well as their browsers.
    tree('target/tui-verify-wasm')
    tree('target/explicit-c-state-batch')
    for folder in ['target/generated-report-preview-browser','target/commission-preview-browser','target/tui-verify-browser','target/native-output-verification-browser','target/native-workspace-browser','target/explicit-c-state-batch-browser']:
        tree(folder)
    for name in ['docs/specs/wasm-label-import-application.md','scripts/web/snapshot_label_import_delivery.py','scripts/web/evidence_blob_store.py','scripts/web/test_evidence_blob_store.py','design/pitfalls/pitfalls-pipeline.md','design/pitfalls/README.md']:
        include(name)
    for file in (ROOT/'crates/bisect-report/tests/fixtures/label-summary').iterdir():
        if file.is_file():
            include(file)
    include('crates/bisect-report/tests/label_summary_security.rs')
    tree('docs/fixtures/import-label')
    practitioner = ROOT/'docs/specs/wasm-practitioner-actions.json'
    include(practitioner)
    pins(json.loads(practitioner.read_text(encoding='utf-8'))['authority'])
    rows, blobs = [], {}
    for name,(path,identity,length) in sorted(files.items()):
        inherited = identity in base.rows
        if inherited:
            base.blob(identity,length)
        else:
            blobs[identity]=path
        rows.append(dict(path=name,sha256=identity,byte_length=length,blob=identity,storage='base' if inherited else 'local'))
    capacity = dict(files=len(rows),unique_blobs=len({row['sha256'] for row in rows}),new_blobs=len(blobs),new_bytes=sum(path.stat().st_size for path in blobs.values()),logical_bytes=sum(row['byte_length'] for row in rows),available_bytes=shutil.disk_usage(ROOT).free,reserve_bytes=32*1024*1024)
    if capacity['new_bytes']+capacity['reserve_bytes']>capacity['available_bytes']:
        raise SystemExit('Snapshot capacity refusal before writes: '+json.dumps(capacity))
    if args.dry_run:
        print(json.dumps(capacity))
        return
    destination.mkdir(parents=True)
    (destination/'blobs').mkdir()
    try:
        for identity,source in blobs.items():
            target=destination/'blobs'/identity
            partial=target.with_suffix('.partial')
            shutil.copy2(source,partial)
            assert digest(partial)==identity
            partial.rename(target)
        for name,(_,identity,_) in files.items():
            assert digest(ROOT/name)==identity, 'Source changed while freezing: '+name
        manifest=destination/'snapshot.json'
        manifest.write_text(json.dumps(dict(schema_version='bisect-label-import-snapshot-v1',storage_format='verified-inherited-blobs-v1',base_snapshot=reference,package=bundle['package'],offline_version=bundle['offline_version'],package_record=dict(path=record.relative_to(ROOT).as_posix(),sha256=digest(record)),storage=capacity,files=rows,full_parity=False,deployed=False,registration_boundary='Final roles review, support/ledger registration and snapshot links follow this execution freeze; excluded to avoid circular hashes. No formal acceptance-v1 promotion.'),indent=2)+'\n',encoding='utf-8')
        verified=SnapshotBlobs(manifest).verify()
        print(json.dumps(dict(snapshot=manifest.relative_to(ROOT).as_posix(),sha256=digest(manifest),verified=verified,storage=capacity)))
    except Exception as error:
        (destination/'failed-attempt.json').write_text(json.dumps(dict(status='failed',error=str(error),storage=capacity),indent=2)+'\n',encoding='utf-8')
        raise


if __name__=='__main__':
    main()

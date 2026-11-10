"""Freeze PDF delivery evidence with verified references to an immutable base.

No existing snapshot is replaced. Registration/review references are excluded to
avoid circular hashes. This developer format grants no browser write authority.
"""
import argparse
import json
import shutil
from pathlib import Path
from evidence_blob_store import ROOT, SnapshotBlobs, digest, local


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--record', required=True)
    parser.add_argument('--dry-run', action='store_true')
    parser.add_argument('--max-storage-bytes', type=int)
    args = parser.parse_args()
    record = local(args.record)
    bundle = json.loads(record.read_text(encoding='utf-8'))
    package = local(bundle['package'])
    assert package.is_relative_to(ROOT / 'dist')
    assert bundle['browser_status'] == 'passed' and bundle['full_parity'] is False and bundle['deployed'] is False
    assert all(stage['exit_code'] == 0 for stage in bundle['stages'])
    baseline = bundle['baseline_snapshot']
    base = SnapshotBlobs(baseline['path'], baseline['sha256'])
    base.verify()
    destination = ROOT / 'target/report-pdf-delivery/package-snapshots' / package.name
    assert not destination.exists(), 'Preserve existing snapshots; choose a fresh package'
    files = {}

    def include(name, expected=None, size=None):
        path = local(name)
        assert path.is_file(), str(path)
        identity = digest(path)
        length = path.stat().st_size
        if expected is not None:
            assert identity == expected, 'Changed evidence: ' + str(path)
        if size is not None:
            assert length == size, 'Changed size: ' + str(path)
        files[path.relative_to(ROOT).as_posix()] = (path, identity, length)

    def tree(name):
        folder = local(name)
        assert folder.is_dir(), str(folder)
        for path in folder.rglob('*'):
            # Disposable browser profiles are neither inputs nor evidence.
            relative = path.relative_to(folder)
            if any(part.startswith('profile') for part in relative.parts):
                continue
            if path.is_file():
                include(path)

    def pins(value):
        if isinstance(value, dict):
            if 'path' in value and 'sha256' in value:
                include(value['path'], value['sha256'], value.get('byte_length'))
            else:
                for nested in value.values():
                    pins(nested)
        elif isinstance(value, list):
            for nested in value:
                pins(nested)

    include(record)
    for field in ['source_files', 'results', 'stages', 'offline_manifest', 'native_cli', 'native_host', 'baseline_native_source_pins_verified']:
        pins(bundle[field])
    tree(record.parent)
    tree(package)
    for profile in bundle['profiles'].values():
        include(profile['wasm'], profile['identity'])
        pins(profile['baseline_gate'])
        if 'count_node' in profile:
            count_node = profile['count_node']
            count_evidence = json.loads((local(count_node['root']) / 'evidence.json').read_text(encoding='utf-8'))
            assert count_evidence['wasm']['sha256'] == profile['identity']
            assert count_evidence['exact_native_transcript_cases'] == 3
            assert count_evidence['native_write_failure_exit'] == 2
            assert count_evidence['native']['sha256'] == bundle['count_transcript_delivery']['native_cli']['sha256']
            tree(count_node['root'])
            count_browser = profile['count_browser']
            assert count_browser['wasm_sha256'] == profile['identity']
            tree(count_browser['root'])
    runs = [profile['current_browser'] for profile in bundle['profiles'].values()] + [bundle['final_browser']]
    for run in runs:
        folder = local(run['root'])
        evidence = json.loads((folder / 'evidence.json').read_text())
        assert evidence['wasm']['sha256'] == run['wasm_sha256']
        assert evidence['actual_pdf_cases'] == 5 and evidence['native_result_matches'] == 5
        assert evidence['native_cli_pdf_dependency_cases'] == 2
        for flag in ['inert_open', 'save_not_inferred', 'dom_mutation_refused', 'late_pdf_read_cannot_replace_newer_observation', 'independent_pdf_inspection']:
            assert evidence[flag] is True
        inspection = json.loads((folder / 'pdf-inspection.json').read_text())
        assert len(inspection['rows']) == 5
        for row in inspection['rows']:
            assert all(row[key] is True for key in ['all_visible_text_preserved', 'native_restricted_text_equal', 'media_box_matches', 'numbered_bottom_margin_footers', 'no_active_or_external_actions'])
        pins(evidence['served_assets'])
        tree(folder)
    final = json.loads((local(bundle['final_browser']['root']) / 'evidence.json').read_text())
    assert final['source_tree'] is False and local(final['package']) == package / 'toolkit'
    assert digest(package / 'toolkit/bisect_wasm.wasm') == final['wasm']['sha256']
    tree(bundle['label_browser']['root'])
    if 'count_transcript_delivery' in bundle:
        count = bundle['count_transcript_delivery']
        pins(count['native_cli'])
        folder = local(count['final_browser']['root'])
        evidence = json.loads((folder / 'evidence.json').read_text(encoding='utf-8'))
        assert evidence['source_tree'] is False and local(evidence['package']) == package / 'toolkit'
        assert evidence['exact_native_cli_transcript'] and evidence['actual_worker_and_opfs']
        assert evidence['wasm']['sha256'] == final['wasm']['sha256']
        pins(evidence['served_assets'])
        tree(folder)
        include('docs/specs/wasm-rcount-transcript-installation.md')
        tree('target/count-transcript-delivery')
        build_identity = json.loads(local('target/count-transcript-delivery/native-build-identity.json').read_text(encoding='utf-8'))
        assert build_identity['native_cli']['sha256'] == count['native_cli']['sha256'] and build_identity['build_exit'] == 0
        pins(build_identity['source_files'])
    for ref in bundle['results']:
        tree(local(ref['path']).parent)
    for name in ['scripts/web/evidence_blob_store.py', 'scripts/web/test_evidence_blob_store.py', 'scripts/web/snapshot_report_pdf_delivery.py', 'docs/specs/wasm-report-pdf-delivery.md', 'output/pdf/synthetic-browser-report-example.pdf']:
        include(name)
    # Preserve failed-run diagnostics without duplicating profiles or engines.
    for name in ['run-1792793535385/pdf-inspection.log', 'run-1792796330610', 'build-1792797135060407500/failed-build.json', 'retained-binary-aliases.json', 'archive-filesystem-compression.log', 'cache-compression-identities.json', 'cache-compression-identities-2.json']:
        path = ROOT / 'target/report-pdf-delivery' / name
        if path.is_file():
            include(path)
        elif path.is_dir():
            for child in path.iterdir():
                if child.is_file() and child.suffix in ['.json', '.log', '.txt']:
                    include(child)
    rows = []
    blobs = {}
    for name, (path, identity, length) in sorted(files.items()):
        inherited = identity in base.rows
        if inherited:
            base.blob(identity, length)
        else:
            blobs[identity] = path
        rows.append(dict(path=name, sha256=identity, byte_length=length, blob=identity, storage='base' if inherited else 'local'))
    required = sum(path.stat().st_size for path in blobs.values())
    free = shutil.disk_usage(ROOT).free
    if args.max_storage_bytes is not None:
        assert args.max_storage_bytes >= 0
        free = min(free, args.max_storage_bytes)
    capacity = dict(files=len(rows), unique_blobs=len({row['sha256'] for row in rows}), new_blobs=len(blobs), new_bytes=required, logical_bytes=sum(row['byte_length'] for row in rows), available_bytes=free, reserve_bytes=16 * 1024 * 1024)
    if required + capacity['reserve_bytes'] > free:
        raise SystemExit('Snapshot capacity refusal before creating output: ' + json.dumps(capacity))
    if args.dry_run:
        print(json.dumps(capacity))
        return
    destination.mkdir(parents=True)
    (destination / 'blobs').mkdir()
    try:
        for identity, source in blobs.items():
            target = destination / 'blobs' / identity
            partial = target.with_suffix('.partial')
            shutil.copy2(source, partial)
            assert digest(partial) == identity
            partial.rename(target)
        for name, (_, identity, _) in files.items():
            assert digest(ROOT / name) == identity, 'Source changed during freeze: ' + name
        manifest = destination / 'snapshot.json'
        data = dict(schema_version='bisect-report-pdf-snapshot-v1', storage_format='verified-inherited-blobs-v1', base_snapshot=baseline, package=bundle['package'], offline_version=bundle['offline_version'], package_record=dict(path=record.relative_to(ROOT).as_posix(), sha256=digest(record)), storage=capacity, files=rows, full_parity=False, deployed=False, registration_boundary='Final implementation review and ledger registration excluded to avoid circular hashes. No acceptance-v1 or full-parity promotion.')
        manifest.write_text(json.dumps(data, indent=2) + '\n', encoding='utf-8')
        verified = SnapshotBlobs(manifest).verify()
        print(json.dumps(dict(snapshot=manifest.relative_to(ROOT).as_posix(), sha256=digest(manifest), verified=verified, storage=capacity)))
    except Exception as error:
        (destination / 'failed-attempt.json').write_text(json.dumps(dict(status='failed', error=str(error), storage=capacity), indent=2) + '\n', encoding='utf-8')
        raise


if __name__ == '__main__':
    main()

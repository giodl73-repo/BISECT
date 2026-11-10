"""Seal exact fresh-engine national checks without rewriting historical evidence."""
import argparse
import json
from pathlib import Path
from evidence_blob_store import ROOT, digest, local


def pin(name):
    path = local(name)
    return dict(path=path.relative_to(ROOT).as_posix(), sha256=digest(path), byte_length=path.stat().st_size)


def verify(ref):
    actual = pin(ref['path'])
    assert actual['sha256'] == ref['sha256'], ref['path']
    if 'byte_length' in ref:
        assert actual['byte_length'] == ref['byte_length'], ref['path']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('config')
    parser.add_argument('output')
    args = parser.parse_args()
    config = json.loads(local(args.config).read_text(encoding='utf-8'))
    package = json.loads(local(config['package_validation']).read_text(encoding='utf-8'))
    assert package['status'] == 'passed' and package['source_unchanged']
    records = []
    for profile in ['default', 'c-sidecar']:
        for ref in package['packages'][profile]:
            verify(ref)
        group = config['profiles'][profile]
        engine = next(r for r in package['packages'][profile] if r['path'].replace('\\', '/').endswith('/toolkit/bisect_wasm.wasm'))
        package_pins = {local(r['path']): r for r in package['packages'][profile]}
        loaded = {}
        for kind, name in group.items():
            record = json.loads(local(name).read_text(encoding='utf-8'))
            expected = {'wasm': 'passed-national-wasm-tier', 'browser': 'passed-national-browser-queue-tier'}.get(kind, 'passed')
            assert record['status'] == expected, (profile, kind, record['status'])
            if kind == 'wasm':
                assert len(record['outcomes']) == 50
                assert record['wasm']['sha256'] == engine['sha256']
                assert all(r['exact_native_cli_assignments'] and r['independent_population_and_connectivity'] for r in record['outcomes'])
            if kind in ['browser', 'map_browser']:
                assert record['wasm_sha256'] == engine['sha256']
                assert local(record['package']) == local(engine['path']).parent
                for asset in record['served_assets']:
                    verify(asset)
                    assert package_pins[local(asset['path'])]['sha256'] == asset['sha256']
            if kind == 'browser':
                assert record['summary']['succeeded'] == 50
                assert all(record[k] for k in ['full_receipts_exact', 'archive_roundtrip', 'open_inert', 'zip_crc', 'completed_prefix_cancelled', 'saved_verification_before_resume', 'unfinished_only_resume'])
            if kind == 'map':
                assert record['wasm']['sha256'] == engine['sha256']
                assert (record['states'], record['units'], record['districts'], record['exact_native_stream_replies']) == (50, 84208, 435, 52)
            if kind == 'comparison':
                assert record['states'] == 50 and all(r['status'] == 'passed-exact-whole-reply' for r in record['outcomes'])
            if kind == 'refusals':
                assert len(record['outcomes']) == 2 and all(r['partition_dispatches'] == 0 and r['resume_refused'] and r['export_refused'] for r in record['outcomes'])
            loaded[kind] = record
            records.append(dict(profile=profile, kind=kind, evidence=pin(name)))
        assert set(loaded) == {'wasm', 'browser', 'map', 'map_browser', 'comparison', 'refusals'}
        assert Path(loaded['browser']['native_checked_reference']['path']).resolve() == local(group['wasm']).parent
        assert Path(loaded['map_browser']['reference']['path']).resolve() == local(group['map']).parent
        assert loaded['comparison']['current']['sha256'] == digest(local(group['wasm']))
        assert loaded['browser']['native_checked_reference']['sha256'] == digest(local(group['wasm']))
        assert loaded['map_browser']['reference']['sha256'] == digest(local(group['map']))
        assert Path(loaded['refusals']['source']['path']).resolve() == local(group['browser']).parent / 'cancelled.bisect-workbench'
    native = json.loads(local(config['native']).read_text(encoding='utf-8'))
    assert native['status'] == 'passed-native-national-tier'
    assert native['summary'] == dict(total=50, succeeded=50, failed=0)
    native_map = json.loads(local(config['native_map']).read_text(encoding='utf-8'))
    assert native_map['status'] == 'passed' and native_map['source_unchanged']
    for ref in native_map['source_files']:
        verify(ref)
    verify(native_map['native_sequence'])
    record = dict(schema_version='bisect-national-engine-refresh-v1', status='passed', package_validation=pin(config['package_validation']), native=pin(config['native']), native_map=pin(config['native_map']), records=records, config=pin(args.config), validator=pin(__file__), deployed=False, acceptance_v1=False, scope='Both exact sealed timing engines; retained 2020 tract standard metis-core seed 42 national generation, all 50 states/435 districts, complete native and historical reply equality, actual browser queue/recovery/refusals and native/rendered retained simplified maps. C-enabled national execution uses metis-core. No fresh acquisition, other vintages/methods/hosts, CLI directory discovery, certification or full parent acceptance.')
    output = local(args.output)
    assert not output.exists()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(pin(output)))


if __name__ == '__main__':
    main()

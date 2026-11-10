"""Bind native naming references to both exact staged browser previews."""
import argparse
import json
from pathlib import Path
from evidence_blob_store import ROOT, digest, local


def pin(name):
    p = local(name)
    return dict(path=p.relative_to(ROOT).as_posix(), sha256=digest(p), byte_length=p.stat().st_size)


def read(name):
    return json.loads(local(name).read_text(encoding='utf-8'))


def verify(ref):
    actual = pin(ref['path'])
    assert actual['sha256'] == ref['sha256'], ref['path']
    assert 'byte_length' not in ref or actual['byte_length'] == ref['byte_length'], ref['path']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('config')
    parser.add_argument('output')
    args = parser.parse_args()
    config = read(args.config)
    native = read(config['native'])
    assert native['status'] == 'passed' and len(native['outcomes']) == 4
    verify(native['native'])
    known_native = read(config['prior_native_authority'])
    assert known_native['status'] == 'passed-native-national-tier'
    assert native['native']['sha256'] == known_native['native']['sha256']
    for row in native['outcomes']:
        assert row['exit_code'] == 0
        for ref in row['files']:
            verify(ref)
    records = {}
    for profile in ['default', 'c-sidecar']:
        paths = config['profiles'][profile]
        stage, target, inspection, browser = [read(paths[k]) for k in ['stage', 'targets', 'inspection', 'browser']]
        assert stage['status'] == 'passed-stage-build' and stage['profile'] == profile
        assert all(c['exit_code'] == 0 for c in stage['checks'])
        verify(stage['wasm'])
        for ref in stage['package_files']:
            verify(ref)
        package = local(stage['package'])
        for source in (ROOT / 'web/lab').glob('*.js'):
            assert digest(source) == digest(package / 'toolkit' / source.name), source
        assert target['status'] == inspection['status'] == browser['status'] == 'passed'
        assert target['actual_native_namespace_label_and_manifest_matches'] == 4
        assert target['native_reference']['sha256'] == digest(local(config['native']))
        assert all(c.startswith('inspect-') for c in target['read_only_operations'])
        assert len(inspection['outcomes']) == 9 and inspection['direct_state_name_guard_retained']
        assert len(browser['cases']) == 7 and all(c['passed'] for c in browser['cases'])
        assert browser['profile'] == profile and browser['package_unchanged']
        assert browser['inert_open_and_explicit_reinspection'] and browser['markup_rendered_as_text']
        assert browser['package_validation']['sha256'] == digest(local(paths['stage']))
        assert local(browser['package']) == package / 'toolkit'
        assert {stage['wasm']['sha256'], target['wasm']['sha256'], inspection['wasm']['sha256'], browser['wasm_sha256']} == {stage['wasm']['sha256']}
        records[profile] = {key: pin(name) for key, name in paths.items()}
    result = dict(schema_version='bisect-state-identity-output-preview-v1', status='passed-naming-preview', config=pin(args.config), native=pin(config['native']), prior_native_authority=pin(config['prior_native_authority']), profiles=records, validator=pin(__file__), deployed=False, acceptance_v1=False, scope='Actual four native CLI naming trees; both fresh WASM naming/target inspections, nine guard cases each, seven real browser preview/Save/Open cases each, exact staged source/assets/offline closure. Raw names, default/explicit labels, internal spaces, native year preflight and incomplete/collision guards. No state-name execution mapping, named auxiliary source handoff, live folder authority, full registered/package/parent acceptance, C partition claim or deployment.')
    output = local(args.output)
    assert not output.exists()
    output.parent.mkdir(parents=True)
    output.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(pin(output)))


if __name__ == '__main__':
    main()

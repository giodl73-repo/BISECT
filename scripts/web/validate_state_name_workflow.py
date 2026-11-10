"""Validate the exact staged named-State journeys and fresh registered gates."""
import argparse
import json
from pathlib import Path
from evidence_blob_store import ROOT, digest, local


def pin(name):
    p = local(name)
    return dict(path=p.relative_to(ROOT).as_posix(),sha256=digest(p),byte_length=p.stat().st_size)


def read(name):
    return json.loads(local(name).read_text(encoding='utf-8-sig'))


def verify(ref):
    current = pin(ref['path'])
    assert current['sha256'] == ref['sha256'], ref['path']
    assert 'byte_length' not in ref or current['byte_length'] == ref['byte_length'], ref['path']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('config')
    parser.add_argument('output')
    args = parser.parse_args()
    config = read(args.config)
    records, run_ids = {}, set()
    for profile in ['default','c-sidecar']:
        paths = config['profiles'][profile]
        stage, browser, ignored, gate = [read(paths[k]) for k in ['stage','browser','ignored','registered']]
        assert stage['status'] == 'passed-stage-build' and stage['profile'] == profile
        for ref in stage['package_files']:
            verify(ref)
        assert all(c['exit_code'] == 0 for c in stage['checks'])
        for source in (ROOT/'web/lab').glob('*.js'):
            assert digest(source) == digest(local(stage['package'])/'toolkit'/source.name), source
        assert gate['status'] == 'passed' and gate['profile'] == profile
        assert gate['engine_build']['mode'] == 'fresh-cargo-build' and gate['engine_build']['recompiled_in_this_runner']
        for ref in gate['source_files']:
            verify(ref)
        run_ids.add(gate['run_id'])
        node_path = local(gate['state_name_workflow_evidence']['node']['root'])/'evidence.json'
        node = read(node_path)
        assert node['status'] == 'passed' and len(node['outcomes']) == 3
        assert node['partitions'] == 6 and node['source_auxiliary_and_label_changes_refused']
        assert all(row['native_assignments_match'] and row['manifest_name_and_source_hash_match'] and row['installed'] and row['reviewed_reset_before_partition'] and row['imported_binding_grants_no_authority'] for row in node['outcomes'])
        assert browser['status'] == 'passed' and browser['profile'] == profile and browser['package_unchanged']
        assert len(browser['cases']) == 3 and all(c['native_assignments_match'] and c['exact_seed'] and c['inert_open_and_explicit_replay'] for c in browser['cases'])
        assert browser['no_implicit_folder_selection'] and browser['package_validation']['sha256'] == digest(local(paths['stage']))
        assert ignored['status'] == 'passed' and ignored['native_assignments_and_default_label_match'] and ignored['unsafe_requested_name_ignored'] and ignored['exact_u64_seed']
        assert {stage['wasm']['sha256'],node['wasm']['sha256'],browser['wasm_sha256'],ignored['wasm']['sha256'],gate['wasm_sha256']} == {stage['wasm']['sha256']}, profile+' engine mismatch'
        for raw in [node,ignored]:
            verify(raw['wasm'])
            verify(raw['native_reference'])
            reference = read(raw['native_reference']['path'])
            assert reference['status'] == 'passed'
            verify(reference['native'])
            for ref in reference.get('files',[]):
                verify(ref)
            for row in reference.get('outcomes',[]):
                for ref in row['files']:
                    verify(ref)
        records[profile] = {**{key:pin(name) for key,name in paths.items()},'registered_node':pin(node_path),'engine':stage['wasm']}
    assert len(run_ids) == 1, 'Both profiles must come from the same registered run'
    zips = read(config['zip_validation'])
    assert zips['status'] == 'passed' and len(zips['outcomes']) == 6
    assert {local(row['path']).parent for row in zips['outcomes']} == {local(group['browser']).parent for group in config['profiles'].values()}
    for archive in zips['outcomes']:
        verify(archive)
        assert archive['exact_native_assignments'] and archive['exact_unsigned_seed'] and archive['zip_crc']
    result = dict(schema_version='bisect-state-name-workflow-validation-v1',status='passed-selected-source-workflow',profiles=records,config=pin(args.config),zip_validation=pin(config['zip_validation']),validator=pin(__file__),registered_run_id=next(iter(run_ids)),deployed=False,acceptance_v1=False,scope='Portable selected native-binary State naming: three retained four-unit native adjacency references per profile plus an actual ignored-name lookup reference; original flags/auxiliary identity rechecked, native-equal core assignments/labels/namespaces/adjacency metadata, exact u64 manifest text, owned Windows filesystem facade install/data-only reset, imported/stale/forged refusals, both actual worker UI/ZIP/Open/replay/reapply journeys and fresh registered dual/browser gates. Automatic named CSV discovery, actual C dispatch, native picker permissions, wider sources/methods/workloads and full flag/parent acceptance remain open.')
    output = local(args.output)
    assert not output.exists()
    output.parent.mkdir(parents=True)
    output.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(pin(output)))


if __name__ == '__main__':
    main()

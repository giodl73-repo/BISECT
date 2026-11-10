"""Read back actual native RI NWay and recorded C replay browser artifacts."""

import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import subprocess


def read(path):
    return json.loads(path.read_bytes())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def literal(data):
    plan = json.loads(data)['plan']
    return dict(zip(plan['units']['unit_ids'], (v + 1 for v in plan['assignment']), strict=True))


def checked_native(root, backend):
    proof = read(root / 'evidence.json')
    assert proof['status'] == 'passed-actual-state-and-replay-native-on-original-ri-tracts'
    assert proof['effective_backend'] == backend and proof['seed'] == '42'
    manifest_path = Path(proof['original_manifest']).resolve(strict=True)
    assert manifest_path.is_relative_to(root.resolve())
    manifest = manifest_path.read_bytes()
    assert sha(manifest) == proof['original_manifest_sha256']
    recipe = json.loads(manifest)['native_replay_recipe']
    assert recipe['recursive_metis_backend'] == backend
    assert recipe['state']['algorithm']['metis']['seed'] == '42'
    plan = (manifest_path.parent / 'plan.rplan').read_bytes()
    assert sha(plan) == proof['original_plan_sha256']
    replay = Path(proof['native_replay_plan']).resolve(strict=True)
    assert replay.is_relative_to(root.resolve())
    assert literal(plan) == literal(replay.read_bytes()) == proof['assignments']
    assert len(proof['assignments']) == 250 and sum(proof['district_populations']) == 1097379
    assert proof['original_root_unchanged'] and proof['changed_source_no_output']
    assert [proof['calls'][n]['exit'] for n in ('state', 'native-replay', 'changed-source')] == [0, 0, 1]
    assert b'Native replay source changed or is unavailable' in (root / 'changed-source.stderr').read_bytes()
    assert not (root / 'changed-source/results').exists()
    assert sha(Path(proof['native']['path']).read_bytes()) == proof['native']['sha256']
    return proof, manifest, recipe


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('native-core', 'native-c', 'browser-core', 'browser-c', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args()
    core, _, core_recipe = checked_native(args.native_core, 'metis-core')
    c, manifest, recipe = checked_native(args.native_c, 'c-ffi')
    assert core['native']['sha256'] != c['native']['sha256']
    assert core['fixture_sha256'] == c['fixture_sha256']
    assert core['source_graph_sha256'] == c['source_graph_sha256']
    assert core_recipe['state']['algorithm'] == recipe['state']['algorithm']
    assert core['assignments'].keys() == c['assignments'].keys()
    # Each plan uses labels 1 and 2. A label permutation would change every unit.
    changed = sum(value != c['assignments'][unit] for unit, value in core['assignments'].items())
    assert changed == 27
    assert core['district_populations'] == [548778, 548601]
    assert c['district_populations'] == [547966, 549413]

    controls = args.output.parent / 'wrong-native-backend'
    controls.mkdir()
    rejected = {}
    for label, binary, original, case in [('core-reads-c', core, c, args.native_c),
                                          ('c-reads-core', c, core, args.native_core)]:
        dest = controls / (label + '-output')
        argv = ['replay-native-state', '--manifest', original['original_manifest'],
                '--output-dir', str(dest.resolve()), '--label', 'WrongBackend']
        env = dict(os.environ, BISECT_MANIFEST=str((case / 'source-manifest.json').resolve()))
        run = subprocess.run([binary['native']['path'], *argv], cwd=controls, env=env,
                             capture_output=True, timeout=180)
        (controls / (label + '.stderr')).write_bytes(run.stderr)
        assert run.returncode == 1 and not dest.exists(), run.stderr
        assert b'Recorded effective recursive METIS backend differs from this native binary' in run.stderr
        rejected[label] = dict(argv=argv, exit=run.returncode, stderr_sha256=sha(run.stderr))

    browser = {}
    for profile, path in [('core', args.browser_core), ('c', args.browser_c)]:
        proof = read(path / 'evidence.json')
        assert proof['status'] == 'passed-real-ri-c-backend-browser-boundary' and proof['mode'] == profile
        assert proof['open_inert'] and not proof['external'] and not proof['errors']
        assert proof['recorded_backend'] == 'c-ffi' and proof['units'] == 250
        assert proof['actual_native_cli'] == c['native']['sha256']
        assert proof['original_manifest_sha256'] == sha(manifest)
        unsupported = read(path / 'unselected-browser.json')
        assert unsupported['status'] == 'preparation-failed' and unsupported['execution_performed'] is False
        assert unsupported['recorded_paths_opened'] is False
        assert proof['served']['/bisect_wasm.wasm'] == proof['wasm_sha256']
        browser[profile] = dict(receipt_sha256=sha((path / 'evidence.json').read_bytes()), wasm=proof['wasm_sha256'])
        if profile == 'core':
            assert not any(name.startswith('/c-metis/') for name in proof['served'])
            continue
        assert proof['unchecked_after_cached_c_refused'] and proof['changed_source_no_execution']
        assert read(path / 'unchecked-after-open.json') == unsupported
        refused = read(path / 'changed-browser.json')
        assert refused['status'] == 'source-check-failed' and refused['execution_performed'] is False
        replay = read(path / 'actual-browser.json')
        assert replay['status'] == 'match' and replay['execution_performed'] is True
        assert replay['recorded_paths_opened'] is False
        assert replay['comparison']['regenerated_assignments'] == replay['comparison']['reference_assignments'] == c['assignments']
        identity = replay['runtime_artifacts']['metis_sidecar']
        assert identity == proof['sidecar_identity'] and identity['runtime'] == 'msvcrt-v1'
        for kind in ('javascript', 'wasm'):
            item = identity[kind]
            assert proof['served']['/c-metis/' + item['file']] == item['sha256']
        plan, context = (path / 'replay.rplan').read_bytes(), (path / 'replay.rctx').read_bytes()
        assert literal(plan) == c['assignments']
        assert plan == replay['replay']['export']['document_json'].encode()
        assert context == replay['replay']['export']['context_json'].encode()
        saved = read(path / 'saved.bisect')['files']
        assert base64.b64decode(saved['packageVerificationInputs']['manifest']['base64'], validate=True) == manifest
        sources = saved['nativeReplaySources']['sources']
        assert {sha(base64.b64decode(s['base64'], validate=True)) for s in sources} == {
            s['sha256'].removeprefix('sha256:') for s in recipe['sources']}
        browser[profile].update(sidecar=identity, export_plan_sha256=sha(plan), export_context_sha256=sha(context))
    assert browser['core']['wasm'] != browser['c']['wasm']
    result = dict(status='passed-maintained-ri-recorded-c-replay-readback', changed_literal_labels=changed,
                  source_graph_sha256=c['source_graph_sha256'], native_core=core['native']['sha256'],
                  native_c=c['native']['sha256'], wrong_native_backend=rejected, browser=browser, full_parity=False)
    with args.output.open('x', encoding='utf8', newline='\n') as f:
        json.dump(result, f, indent=2)
        f.write('\n')
    print(result['status'])


if __name__ == '__main__':
    main()

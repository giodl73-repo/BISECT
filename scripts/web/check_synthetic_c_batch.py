"""Compare the original two browser plans to the actual C-enabled State CLI."""
import hashlib
import io
import json
import os
from pathlib import Path
import struct
import subprocess
import sys
import zipfile


def read(path):
    return json.loads(path.read_bytes())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def assignment(plan):
    plan = plan['plan']
    return dict(zip(plan['units']['unit_ids'], (d + 1 for d in plan['assignment']), strict=True))


def main(root, cli, source_manifest):
    root, cli = root.resolve(strict=True), cli.resolve(strict=True)
    dest = root / 'desktop-cli'
    dest.mkdir()
    manifest = read(source_manifest)
    manifest.update(local_outputs_dir=str(dest / 'inputs'), local_data_dir=str(dest / 'data'))
    manifest_file = dest / 'source-manifest.json'
    manifest_file.write_text(json.dumps(manifest), encoding='utf8')
    browser = read(root / 'browser-c/evidence.json')
    core = read(root / 'browser-core/evidence.json')
    assert browser['mixed_runtime_refused_before_state_dispatch'] and core['online'] is None
    assert all(x['open_inert'] and len(x['cold']['failures']) == 2 and x['cold']['slots'] == 2
               for x in (browser, core))
    offline = (root / 'browser-c/offline.zip').read_bytes()
    assert offline == (root / 'browser-c/online.zip').read_bytes()
    assert sha(offline) == browser['offline']['zip_sha256']
    checks = {}
    with zipfile.ZipFile(io.BytesIO(offline)) as batch:
        assert batch.testzip() is None
        assert set(batch.namelist()) == {'RI/prepared-run.zip', 'VT/prepared-run.zip', 'batch-receipt.json'}
        receipt = json.loads(batch.read('batch-receipt.json'))
        assert receipt['workers']['effective'] == 2 and receipt['summary']['succeeded'] == 2
        for state, districts in (('RI', 2), ('VT', 1)):
            case = dest / state
            case.mkdir()
            ref = read(root / f'{state}-native-request.json')
            graph = ref['graph']
            n = len(graph['geoids'])
            blob = bytearray(b'RADJ' + struct.pack('<III', 3, n, len(graph['edges'])))
            blob += struct.pack(f'<{n}q', *graph['population'])
            for row in graph['adjacency']:
                blob += struct.pack(f'<I{len(row)}I', len(row), *row)
            blob += struct.pack('<I', len(graph['edges']))
            for edge in graph['edges']:
                blob += struct.pack('<IId', *edge)
            blob += struct.pack('<I', 1)
            for field in ('exterior_perimeters', 'areas'):
                blob += struct.pack(f'<{n}d', *graph[field])
            adj = case / 'source.adj.bin'
            adj.write_bytes(blob)
            (case / 'source_geoids.json').write_text(json.dumps(dict(enumerate(graph['geoids']))), encoding='utf8')
            (case / 'source_centroids.json').write_text(json.dumps({'centroids': graph['centroids']}), encoding='utf8')
            args = ['state', *ref['state_args'], '--year', '2020', '--chamber', 'congressional',
                    '--label', 'ExplicitC', '--version', 'v1', '--output-dir', str(case / 'v1'),
                    '--adjacency', str(adj), '--manifest', '--force']
            proc = subprocess.run([cli, *args], cwd=case, env=dict(os.environ, BISECT_MANIFEST=str(manifest_file)),
                                  capture_output=True, timeout=120)
            (case / 'native.stdout').write_bytes(proc.stdout)
            (case / 'native.stderr').write_bytes(proc.stderr)
            assert proc.returncode == 0, proc.stderr.decode(errors='replace')
            package = case / 'v1/2020/plans/ExplicitC'
            recipe = read(package / 'manifest.json')['native_replay_recipe']
            assert recipe['recursive_metis_backend'] == 'c-ffi'
            assert recipe['state']['algorithm']['metis']['seed'] == '7'
            native = assignment(read(package / 'plan.rplan'))
            pinned = json.loads(read(root / f'{state}-native-reply.json')['stdout'])['assignments']
            with zipfile.ZipFile(io.BytesIO(batch.read(f'{state}/prepared-run.zip'))) as output:
                assert output.testzip() is None
                request = output.read('request.json')
                req = json.loads(request)
                opts = req['options']
                assert req['graph'] == graph and opts['districts'] == districts
                assert (opts['seed'], opts['metis_engine'], opts['metis_runtime']) == ('7', 'c-ffi', 'msvcrt-v1')
                assert sha(request) == browser['offline']['states'][state]['request_sha256']
                actual = json.loads(output.read('final_assignments.json'))
                assert actual == assignment(json.loads(output.read('plan.rplan'))) == native == pinned
                assert set(actual) == set(graph['geoids']) and set(actual.values()) == set(range(1, districts + 1))
                checked = json.loads(output.read('prepared-run-receipt.json'))['prepared']
                runtime = browser['offline']['states'][state]
                assert checked.get('runtime_artifacts') == runtime['runtime']
                assert bool(runtime['runtime']) == (districts > 1)
                assert runtime['loaded_worker_runtime']['metis_sidecar']['runtime'] == 'msvcrt-v1'
                checks[state] = {'native_cli_arguments': args, 'districts': districts, 'units': n,
                                 'request_sha256': sha(request), 'native_recipe': recipe['state']['algorithm']}
    result = {'status': 'passed-original-synthetic-c-batch-cli-and-zip', 'states': checks,
              'native_executable_sha256': sha(cli.read_bytes()), 'zip_sha256': sha(offline),
              'mixed_runtime_refused_before_state_dispatch': True, 'full_parity': False}
    with (root / 'independent.json').open('x', encoding='utf8') as f:
        json.dump(result, f, indent=2)


if __name__ == '__main__':
    main(*map(Path, sys.argv[1:]))

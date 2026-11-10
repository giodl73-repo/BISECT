"""Run actual native C METIS State on two retained graphs via independent RADJ encoding."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess


def sha(data):
    return hashlib.sha256(data).hexdigest()


def native_batch(assets, cli, manifest_path, out):
    out = out.resolve()
    cli = cli.resolve(strict=True)
    out.mkdir()
    catalog = json.loads((assets / 'catalog.json').read_bytes())
    manifest = json.loads(manifest_path.read_bytes())
    manifest.update(local_outputs_dir=str(out / 'inputs'), local_data_dir=str(out / 'data'))
    for subdir in ('inputs', 'data'):
        (out / subdir).mkdir()
    manifest_file = out / 'source-manifest.json'
    manifest_file.write_text(json.dumps(manifest), encoding='utf8')
    records = {}
    for state in ('RI', 'NH'):
        folder = out / state
        folder.mkdir()
        ref = catalog['graphs'][state + ':2020']['graph_ref']
        raw = (assets / ref).read_bytes()
        assert sha(raw) == catalog['assets'][ref]
        (folder / 'graph.json').write_bytes(raw)
        graph = json.loads(raw)
        units = len(graph['geoids'])
        assert (graph['state'], graph['year']) == (state, '2020')
        assert units == len(graph['population']) == len(graph['adjacency'])
        binary = bytearray(b'RADJ' + struct.pack('<III', 3, units, len(graph['edges'])))
        binary += struct.pack(f'<{units}q', *graph['population'])
        for neighbors in graph['adjacency']:
            binary += struct.pack('<I', len(neighbors)) + struct.pack(f'<{len(neighbors)}I', *neighbors)
        binary += struct.pack('<I', len(graph['edges']))
        for u, v, w in graph['edges']:
            binary += struct.pack('<IId', u, v, w)
        binary += struct.pack('<I', 1)
        for field in ('exterior_perimeters', 'areas'):
            binary += struct.pack(f'<{units}d', *graph[field])
        adjacency = folder / 'source.adj.bin'
        adjacency.write_bytes(binary)
        (folder / 'source_geoids.json').write_text(json.dumps(dict(enumerate(graph['geoids']))), encoding='utf8')
        (folder / 'source_centroids.json').write_text(json.dumps({'centroids': graph['centroids']}), encoding='utf8')
        dest = folder / 'outputs/v1'
        args = ['state', '--state', state, '--year', '2020', '--chamber', 'congressional', '--districts', '2',
                '--label', 'ExplicitC', '--version', 'v1', '--output-dir', str(dest), '--adjacency', str(adjacency),
                '--metis-engine', 'c-ffi', '--structure', 'nway', '--search', 'single', '--seed', '42',
                '--balance-tolerance', '0.5', '--force', '--manifest']
        run = subprocess.run([cli, *args], cwd=folder, env=dict(os.environ, BISECT_MANIFEST=str(manifest_file)),
                             capture_output=True, timeout=120)
        (folder / 'native.stdout').write_bytes(run.stdout)
        (folder / 'native.stderr').write_bytes(run.stderr)
        assert run.returncode == 0, run.stderr.decode(errors='replace')
        package = dest / '2020/plans/ExplicitC'
        metadata = json.loads((package / 'manifest.json').read_bytes())
        recipe = metadata['native_replay_recipe']
        assert recipe['recursive_metis_backend'] == 'c-ffi'
        algorithm = recipe['state']['algorithm']
        assert algorithm['metis']['seed'] == '42'
        plan = json.loads((package / 'plan.rplan').read_bytes())['plan']
        assert plan['units']['unit_ids'] == graph['geoids']
        assignments = dict(zip(graph['geoids'], (d + 1 for d in plan['assignment']), strict=True))
        # Independent graph connectivity and population observations.
        district_populations = []
        for district in (1, 2):
            members = {i for i, geoid in enumerate(graph['geoids']) if assignments[geoid] == district}
            stack = [min(members)]
            seen = set(stack)
            for vertex in stack:
                for neighbor in graph['adjacency'][vertex]:
                    if neighbor in members and neighbor not in seen:
                        seen.add(neighbor)
                        stack.append(neighbor)
            assert seen == members
            district_populations.append(sum(graph['population'][i] for i in members))
        records[state] = {'native_args': args, 'units': units, 'districts': 2, 'seed': '42',
                          'graph_sha256': sha(raw), 'radj_sha256': sha(binary), 'assignments': assignments,
                          'district_populations': district_populations, 'native_algorithm': algorithm,
                          'effective_backend': recipe['recursive_metis_backend']}
    result = {'status': 'passed-actual-c-native-state-on-retained-batch-graphs',
              'native_exe_sha256': sha(cli.read_bytes()), 'states': records}
    (out / 'native-evidence.json').write_text(json.dumps(result, indent=2), encoding='utf8')


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('assets', type=Path)
    p.add_argument('cli', type=Path)
    p.add_argument('manifest', type=Path)
    p.add_argument('output', type=Path)
    args = p.parse_args()
    native_batch(args.assets, args.cli, args.manifest, args.output)

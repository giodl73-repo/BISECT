"""Recheck the scoped C sidecar matrix against the public native State command."""
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess
import sys


def read(p):
    return json.loads(p.read_bytes())


def digest(b):
    return hashlib.sha256(b).hexdigest()


def radj(graph):
    n = len(graph['geoids'])
    edges = graph['edges']
    data = bytearray(b'RADJ' + struct.pack('<III', 3, n, len(edges)))
    data += struct.pack(f'<{n}q', *graph['population'])
    for row in graph['adjacency']:
        data += struct.pack(f'<I{len(row)}I', len(row), *row)
    data += struct.pack('<I', len(edges))
    for edge in edges:
        data += struct.pack('<IId', *edge)
    data += struct.pack('<I', 1)
    for key in ('exterior_perimeters', 'areas'):
        data += struct.pack(f'<{n}d', *graph[key])
    return data


def write_algorithm_tiger(work, graph):
    # The public CLI's AreaSection needs a TIGER ALAND input. These synthetic
    # polygons supply exactly the GEOID/area pairs used by the graph oracle.
    import shapefile

    stem = 'tl_2020_44_tract'
    base = work / 'data/2020/tiger/tracts' / stem / stem
    base.parent.mkdir(parents=True)
    with shapefile.Writer(str(base), shapeType=shapefile.POLYGON) as out:
        out.field('GEOID', 'C', size=11)
        out.field('ALAND', 'N', size=12)
        for geoid, area, (x, y) in zip(graph['geoids'], graph['areas'], graph['centroids'], strict=True):
            assert area == int(area)
            d = 0.0001
            out.poly([[(x-d, y-d), (x-d, y+d), (x+d, y+d), (x+d, y-d), (x-d, y-d)]])
            out.record(geoid, int(area))
    dbf = base.with_suffix('.dbf')
    raw = bytearray(dbf.read_bytes())
    raw[1:4] = bytes((120, 1, 1))  # Pin DBF header date to the fixture vintage.
    dbf.write_bytes(raw)
    base.with_suffix('.prj').write_text('GEOGCS["NAD83",DATUM["North_American_Datum_1983",SPHEROID["GRS 1980",6378137,298.257222101]],PRIMEM["Greenwich",0],UNIT["Degree",0.0174532925199433]]', encoding='ascii')
    return {p.suffix: digest(p.read_bytes()) for p in sorted(base.parent.iterdir())}


def compare(index_row, cli, root, env):
    idx, row = index_row
    work = root / f'{idx:02}'
    work.mkdir()
    graph, opts = (row['request'][key] for key in ('graph', 'options'))
    adj = work / 'source.adj.bin'
    adj.write_bytes(radj(graph))
    (work / 'source_geoids.json').write_text(json.dumps(dict(enumerate(graph['geoids']))), encoding='utf8')
    (work / 'source_centroids.json').write_text(json.dumps({'centroids': graph['centroids']}), encoding='utf8')
    tiger = write_algorithm_tiger(work, graph) if opts['structure'] == 'ratio-optimal-area' else None
    args = ['state', *row['original_native_arguments'], '--year', '2020', '--chamber', 'congressional',
            '--label', 'MatrixC', '--version', 'v1', '--output-dir', str(work / 'v1'),
            '--adjacency', str(adj), '--manifest', '--force']
    proc = subprocess.run([cli, *args], cwd=work, env=env, capture_output=True, timeout=120)
    (work / 'native.stdout').write_bytes(proc.stdout)
    (work / 'native.stderr').write_bytes(proc.stderr)
    record = {'index': idx, 'cli_args': args, 'options': opts, 'radj_sha256': digest(adj.read_bytes()),
              'exit_code': proc.returncode, 'synthetic_algorithm_tiger': tiger}
    package = work / 'v1/2020/plans/MatrixC'
    if row['native_exit_code']:
        reason = json.loads(row['native_stderr'].strip().split('Error: ', 1)[1])
        stderr = proc.stderr.decode(errors='replace')
        assert proc.returncode != 0 and reason in stderr and not (package / 'plan.rplan').exists(), (idx, stderr)
        return dict(record, outcome='expected-native-failure', reason=reason)
    assert proc.returncode == 0, (idx, proc.stderr.decode(errors='replace'))
    recipe = read(package / 'manifest.json')['native_replay_recipe']
    assert recipe['recursive_metis_backend'] == 'c-ffi'
    assert recipe['state']['algorithm']['metis']['seed'] == opts['seed']
    raw_plan = (package / 'plan.rplan').read_bytes()
    plan = json.loads(raw_plan)['plan']
    assert plan['units']['unit_ids'] == graph['geoids']
    assignments = dict(zip(graph['geoids'], (d + 1 for d in plan['assignment']), strict=True))
    assert assignments == row['native']['assignments'] == row['result']['assignments'], idx
    cut = sum((1 if opts['weights'] == 'unweighted' else w) for u, v, w in graph['edges']
              if assignments[graph['geoids'][u]] != assignments[graph['geoids'][v]])
    assert cut == row['native']['weighted_boundary'], idx
    return dict(record, outcome='accepted-output', assignments=assignments, weighted_boundary=cut,
                plan_sha256=digest(raw_plan), native_algorithm=recipe['state']['algorithm'])


def main(matrix, cli, source_manifest, root):
    cli, root = cli.resolve(strict=True), root.resolve()
    rows = read(matrix)
    assert len(rows) == 56
    root.mkdir()
    manifest = read(source_manifest)
    manifest.update(local_outputs_dir=str(root / 'inputs'), local_data_dir=str(root / 'data'))
    manifest_file = root / 'source-manifest.json'
    manifest_file.write_text(json.dumps(manifest), encoding='utf8')
    env = dict(os.environ, BISECT_MANIFEST=str(manifest_file))
    with ThreadPoolExecutor(max_workers=2) as pool:
        results = list(pool.map(lambda row: compare(row, cli, root, env), enumerate(rows)))
    successes = sum(row['outcome'] == 'accepted-output' for row in results)
    assert successes == 52
    with (root / 'evidence.json').open('x', encoding='utf8') as f:
        json.dump({'status': 'passed-actual-state-cli-c-sidecar-matrix', 'matches': successes,
                   'expected_native_failures': len(rows) - successes, 'native_exe_sha256': digest(cli.read_bytes()),
                   'engine_matrix_sha256': digest(matrix.read_bytes()), 'results': results}, f, indent=2)


if __name__ == '__main__':
    main(*map(Path, sys.argv[1:]))
